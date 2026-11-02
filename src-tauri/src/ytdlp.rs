//! Получение аудиопотока через yt-dlp и разбор ссылок (YouTube, SoundCloud,
//! плейлисты, Spotify → поиск на YouTube).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::deps::ytdlp_command;
use crate::search::{self, Track};
use crate::{AppState, StreamEntry, StreamKind};

/// Ссылки на YouTube живут ~6 часов, перестрахуемся.
const STREAM_TTL: Duration = Duration::from_secs(4 * 60 * 60);

/// Предпочитаем прямые (progressive) аудиопотоки — их умеет <audio>.
const FORMAT: &str = "bestaudio[protocol^=http][protocol!*=dash][ext=webm]/bestaudio[protocol^=http][protocol!*=dash][ext=m4a]/bestaudio[protocol^=http][protocol!*=dash]/bestaudio/best";

pub fn key_for(url: &str) -> String {
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn mime_for(ext: &str) -> &'static str {
    match ext {
        "m4a" | "mp4" | "m4b" => "audio/mp4",
        "webm" | "weba" => "audio/webm",
        "mp3" => "audio/mpeg",
        "ogg" | "oga" => "audio/ogg",
        "opus" => "audio/ogg",
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        _ => "application/octet-stream",
    }
}

/// Запускает yt-dlp и возвращает stdout.
pub async fn run(state: &AppState, args: &[&str], timeout_s: u64) -> Result<String, String> {
    let mut cmd = ytdlp_command(state)?;
    cmd.arg("--ignore-config")
        .arg("--no-warnings")
        .arg("--no-progress")
        .arg("--encoding")
        .arg("utf-8")
        .arg("--cache-dir")
        .arg(&state.ytdlp_cache);
    cmd.args(args);
    let out = tokio::time::timeout(Duration::from_secs(timeout_s), cmd.output())
        .await
        .map_err(|_| "yt-dlp не ответил вовремя".to_string())?
        .map_err(|e| format!("Не удалось запустить yt-dlp: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if !out.status.success() && stdout.trim().is_empty() {
        let err = String::from_utf8_lossy(&out.stderr);
        let line = err
            .lines()
            .rev()
            .find(|l| l.contains("ERROR"))
            .or_else(|| err.lines().rev().find(|l| !l.trim().is_empty()))
            .unwrap_or("неизвестная ошибка")
            .replace("ERROR: ", "");
        return Err(line.chars().take(300).collect());
    }
    Ok(stdout)
}

fn lock_for(state: &AppState, key: &str) -> Arc<tokio::sync::Mutex<()>> {
    let mut locks = state.locks.lock().unwrap();
    locks
        .entry(key.to_string())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

fn is_youtube(url: &str) -> bool {
    url.starts_with("ytsearch") || url.contains("youtube.com/") || url.contains("youtu.be/")
}

/// Один запрос к yt-dlp: прямая ссылка на аудио + заголовки.
async fn probe_with(state: &AppState, url: &str, extra: &[&str], timeout: u64) -> Result<Value, String> {
    let mut args: Vec<&str> = vec![
        "--no-playlist",
        "--playlist-items",
        "1",
        "--socket-timeout",
        "10",
        "-f",
        FORMAT,
        "--print",
        "%(.{url,http_headers,duration,ext,protocol,format_id})j",
    ];
    args.extend_from_slice(extra);
    args.push("--");
    args.push(url);
    let out = run(state, &args, timeout).await?;
    let line = out
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .ok_or("yt-dlp не вернул поток")?;
    let info: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
    if info["url"].as_str().unwrap_or_default().is_empty() {
        return Err("yt-dlp не вернул ссылку на поток".into());
    }
    Ok(info)
}

/// Для YouTube сначала пробуем клиент android_vr: он отдаёт прямые ссылки без
/// JS-проверки, поэтому yt-dlp не скачивает плеер и не запускает Deno —
/// это экономит несколько секунд. Не вышло — обычный путь.
async fn probe(state: &AppState, url: &str) -> Result<Value, String> {
    if is_youtube(url) {
        let fast = probe_with(
            state,
            url,
            &["--extractor-args", "youtube:player_client=android_vr;player_skip=webpage,configs"],
            25,
        )
        .await;
        if let Ok(v) = fast {
            return Ok(v);
        }
    }
    probe_with(state, url, &[], 90).await
}

/// Название и исполнитель трека — чтобы найти замену, если оригинал недоступен.
#[derive(Deserialize, Clone, Default)]
pub struct AltHint {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub duration: f64,
}

/// Ошибки «видео недоступно / заблокировано в вашей стране / удалено».
fn is_unavailable(err: &str) -> bool {
    let e = err.to_lowercase();
    [
        "unavailable",
        "not available",
        "blocked",
        "country",
        "region",
        "copyright",
        "has been removed",
        "private video",
        "terminated",
        "who has blocked",
    ]
    .iter()
    .any(|m| e.contains(m))
}

/// Оригинал недоступен (чаще всего — заблокирован правообладателем в регионе).
/// Ищем другую копию того же трека: сначала другие загрузки на YouTube,
/// потом SoundCloud. Берём только варианты с близкой длительностью.
async fn find_substitute(state: &AppState, orig_url: &str, hint: &AltHint) -> Result<(String, Value, &'static str), String> {
    let title = crate::lyrics::clean_title(&hint.title);
    let query = format!("{} {}", hint.artist.trim(), title).trim().to_string();
    if query.is_empty() {
        return Err("нечего искать".into());
    }
    let close = |d: f64| hint.duration <= 0.0 || d <= 0.0 || (d - hint.duration).abs() <= 20.0;

    // 1) другие загрузки на YouTube
    let search = format!("ytsearch8:{query}");
    if let Ok(out) = run(state, &["--flat-playlist", "-J", "--", &search], 40).await {
        if let Ok(v) = serde_json::from_str::<Value>(out.trim()) {
            let mut tried = 0;
            for e in v["entries"].as_array().cloned().unwrap_or_default() {
                let Some(id) = e["id"].as_str() else { continue };
                if orig_url.contains(id) || !close(e["duration"].as_f64().unwrap_or(0.0)) {
                    continue;
                }
                let url = format!("https://www.youtube.com/watch?v={id}");
                if let Ok(info) = probe(state, &url).await {
                    return Ok((url, info, "YouTube"));
                }
                tried += 1;
                if tried >= 3 {
                    break;
                }
            }
        }
    }

    // 2) SoundCloud
    if let Ok(tracks) = crate::search::sc_search(state, &query).await {
        for t in tracks.into_iter().filter(|t| close(t.duration)).take(2) {
            if let Ok(info) = probe_with(state, &t.url, &[], 60).await {
                return Ok((t.url, info, "SoundCloud"));
            }
        }
    }
    Err("Трек недоступен в твоём регионе, и другую копию найти не удалось".into())
}

/// Из ответа yt-dlp — откуда и как отдавать звук.
async fn make_stream(state: &AppState, key: &str, page_url: &str, info: &Value) -> Result<(StreamKind, String, f64), String> {
    let stream_url = info["url"].as_str().unwrap_or_default().to_string();
    let protocol = info["protocol"].as_str().unwrap_or_default().to_string();
    let ext = info["ext"].as_str().unwrap_or("webm").to_string();
    let duration = info["duration"].as_f64().unwrap_or(0.0);
    let format_id = info["format_id"].as_str().unwrap_or_default().to_string();

    let kind = if !stream_url.is_empty()
        && protocol.starts_with("http")
        && !protocol.contains("dash")
        && !protocol.contains("m3u8")
    {
        let headers = info["http_headers"]
            .as_object()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        StreamKind::Remote {
            url: stream_url,
            headers,
        }
    } else {
        // HLS/DASH: скачиваем трек целиком в кэш (обычно 3–8 МБ, пара секунд).
        let template = state.cache_dir.join(format!("{key}.%(ext)s"));
        let template = template.to_string_lossy().to_string();
        let fmt = if format_id.is_empty() { FORMAT.to_string() } else { format_id };
        let printed = run(
            state,
            &[
                "--no-playlist",
                "--playlist-items",
                "1",
                "-f",
                &fmt,
                "--no-part",
                "--force-overwrites",
                "-o",
                &template,
                "--print",
                "after_move:filepath",
                "--",
                page_url,
            ],
            180,
        )
        .await?;
        let path = printed
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .map(|l| std::path::PathBuf::from(l.trim()))
            .filter(|p| p.is_file())
            .ok_or("Не удалось скачать трек")?;
        StreamKind::File(path)
    };

    let mime = match &kind {
        StreamKind::File(p) => mime_for(
            &p.extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default(),
        ),
        StreamKind::Remote { .. } => mime_for(&ext),
    }
    .to_string();
    Ok((kind, mime, duration))
}

/// Достаёт поток для страницы трека и кладёт в кэш (ключ — по исходной ссылке,
/// даже если звук взят с замены).
pub async fn resolve_inner(state: &AppState, url: &str, force: bool, hint: Option<&AltHint>) -> Result<StreamEntry, String> {
    let key = key_for(url);
    let lock = lock_for(state, &key);
    let _guard = lock.lock().await;

    if !force {
        if let Some(e) = state.streams.lock().unwrap().get(&key) {
            let alive = match &e.kind {
                StreamKind::File(p) => p.is_file(),
                StreamKind::Remote { .. } => e.created.elapsed() < STREAM_TTL,
            };
            if alive {
                return Ok(e.clone());
            }
        }
    }

    let (page_url, info, substitute) = match probe(state, url).await {
        Ok(info) => (url.to_string(), info, None),
        Err(e) if is_unavailable(&e) && hint.is_some() => {
            let (alt, info, src) = find_substitute(state, url, hint.unwrap()).await?;
            (alt, info, Some(src.to_string()))
        }
        Err(e) => return Err(e),
    };

    let (kind, mime, duration) = make_stream(state, &key, &page_url, &info).await?;
    let entry = StreamEntry {
        page_url,
        substitute,
        kind,
        mime,
        duration,
        created: Instant::now(),
    };
    state.streams.lock().unwrap().insert(key, entry.clone());
    Ok(entry)
}

/// Ссылка на поток протухла — получаем свежую с той же страницы (оригинала
/// или замены) и кладём под тем же ключом.
pub async fn refresh(state: &AppState, key: &str) -> Result<(), String> {
    let old = state.streams.lock().unwrap().get(key).cloned().ok_or("нет в кэше")?;
    let lock = lock_for(state, key);
    let _guard = lock.lock().await;
    let info = probe(state, &old.page_url).await?;
    let (kind, mime, duration) = make_stream(state, key, &old.page_url, &info).await?;
    let entry = StreamEntry {
        kind,
        mime,
        duration,
        created: Instant::now(),
        ..old
    };
    state.streams.lock().unwrap().insert(key.to_string(), entry);
    Ok(())
}

#[derive(Serialize)]
pub struct Resolved {
    key: String,
    duration: f64,
    /// Если оригинал недоступен — откуда взята замена.
    substitute: Option<String>,
}

#[tauri::command]
pub async fn resolve(
    state: State<'_, AppState>,
    url: String,
    force: Option<bool>,
    hint: Option<AltHint>,
) -> Result<Resolved, String> {
    let state = state.inner().clone();
    let entry = resolve_inner(&state, &url, force.unwrap_or(false), hint.as_ref()).await?;
    Ok(Resolved {
        key: key_for(&url),
        duration: entry.duration,
        substitute: entry.substitute,
    })
}

// ---------- Ссылки ----------

fn best_thumb(v: &Value) -> String {
    if let Some(t) = v["thumbnail"].as_str() {
        return t.to_string();
    }
    v["thumbnails"]
        .as_array()
        .and_then(|a| a.last())
        .and_then(|t| t["url"].as_str())
        .unwrap_or_default()
        .to_string()
}

fn source_of(url: &str) -> &'static str {
    if url.contains("soundcloud.com") {
        "sc"
    } else if url.contains("music.youtube.com") {
        "ytm"
    } else if url.contains("youtube.com") || url.contains("youtu.be") {
        "yt"
    } else {
        "link"
    }
}

fn entry_to_track(v: &Value, fallback_thumb: &str) -> Option<Track> {
    let mut url = v["webpage_url"]
        .as_str()
        .or_else(|| v["url"].as_str())
        .unwrap_or_default()
        .to_string();
    let id = v["id"].as_str().unwrap_or_default().to_string();
    let ie = v["ie_key"].as_str().or_else(|| v["extractor_key"].as_str()).unwrap_or("");
    if (url.is_empty() || !url.starts_with("http")) && ie.starts_with("Youtube") && !id.is_empty() {
        url = format!("https://www.youtube.com/watch?v={id}");
    }
    if url.is_empty() {
        return None;
    }
    let title = v["track"]
        .as_str()
        .or_else(|| v["title"].as_str())
        .unwrap_or("Без названия")
        .to_string();
    let artist = v["artist"]
        .as_str()
        .or_else(|| v["creator"].as_str())
        .or_else(|| v["uploader"].as_str())
        .or_else(|| v["channel"].as_str())
        .unwrap_or("")
        .to_string();
    let mut thumb = best_thumb(v);
    if thumb.is_empty() {
        if ie.starts_with("Youtube") && !id.is_empty() {
            thumb = format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg");
        } else {
            thumb = fallback_thumb.to_string();
        }
    }
    let src = source_of(&url);
    Some(Track {
        id: format!("{src}:{}", if id.is_empty() { key_for(&url) } else { id }),
        url,
        title,
        artist,
        duration: v["duration"].as_f64().unwrap_or(0.0),
        thumb,
        source: src.to_string(),
        explicit: false,
    })
}

/// Разбирает вставленную ссылку: трек, плейлист, альбом, Spotify и т.д.
#[tauri::command]
pub async fn resolve_link(state: State<'_, AppState>, url: String) -> Result<Vec<Track>, String> {
    let state = state.inner().clone();
    let url = url.trim().to_string();

    if url.contains("open.spotify.com") || url.starts_with("spotify:") {
        return spotify(&state, &url).await;
    }

    let out = run(
        &state,
        &["--flat-playlist", "-J", "--playlist-end", "200", "--", &url],
        90,
    )
    .await?;
    let v: Value = serde_json::from_str(out.trim()).map_err(|e| e.to_string())?;
    let mut tracks = Vec::new();
    if v["_type"].as_str() == Some("playlist") {
        let cover = best_thumb(&v);
        if let Some(entries) = v["entries"].as_array() {
            for e in entries {
                if let Some(t) = entry_to_track(e, &cover) {
                    tracks.push(t);
                }
            }
        }
    } else if let Some(t) = entry_to_track(&v, "") {
        tracks.push(t);
    }
    if tracks.is_empty() {
        return Err("По ссылке ничего не найдено".into());
    }
    Ok(tracks)
}

// ---------- Spotify ----------
// Сам Spotify отдаёт аудио только с DRM, поэтому берём из ссылки название и
// исполнителя и находим этот трек на YouTube Music.

fn spotify_embed_url(url: &str) -> Option<String> {
    let cleaned = url.split('?').next().unwrap_or(url);
    let (kind, id) = if let Some(rest) = cleaned.strip_prefix("spotify:") {
        let mut it = rest.split(':');
        (it.next()?.to_string(), it.next()?.to_string())
    } else {
        let parts: Vec<&str> = cleaned
            .split('/')
            .skip_while(|p| !p.contains("open.spotify.com"))
            .skip(1)
            .filter(|p| !p.is_empty() && !p.starts_with("intl-"))
            .collect();
        if parts.len() < 2 {
            return None;
        }
        (parts[0].to_string(), parts[1].to_string())
    };
    Some(format!("https://open.spotify.com/embed/{kind}/{id}"))
}

fn next_data(html: &str) -> Option<Value> {
    let start_tag = "<script id=\"__NEXT_DATA__\" type=\"application/json\">";
    let s = html.find(start_tag)? + start_tag.len();
    let e = html[s..].find("</script>")? + s;
    serde_json::from_str(&html[s..e]).ok()
}

async fn spotify(state: &AppState, url: &str) -> Result<Vec<Track>, String> {
    let embed = spotify_embed_url(url).ok_or("Не понимаю эту ссылку Spotify")?;
    let html = state
        .http()
        .get(&embed)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let data = next_data(&html).ok_or("Spotify не отдал данные")?;
    let entity = &data["props"]["pageProps"]["state"]["data"]["entity"];
    let cover = entity["visualIdentity"]["image"]
        .as_array()
        .and_then(|a| a.iter().max_by_key(|i| i["maxWidth"].as_u64().unwrap_or(0)))
        .and_then(|i| i["url"].as_str())
        .or_else(|| {
            entity["coverArt"]["sources"]
                .as_array()
                .and_then(|a| a.last())
                .and_then(|i| i["url"].as_str())
        })
        .unwrap_or_default()
        .to_string();

    let mut pairs: Vec<(String, String, f64)> = Vec::new();
    if let Some(list) = entity["trackList"].as_array() {
        for t in list {
            let title = t["title"].as_str().unwrap_or_default().to_string();
            let artist = t["subtitle"].as_str().unwrap_or_default().replace('\u{a0}', " ");
            let dur = t["duration"].as_f64().unwrap_or(0.0) / 1000.0;
            if !title.is_empty() {
                pairs.push((title, artist, dur));
            }
        }
    } else {
        let title = entity["name"]
            .as_str()
            .or_else(|| entity["title"].as_str())
            .unwrap_or_default()
            .to_string();
        let artist = entity["artists"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x["name"].as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let dur = entity["duration"].as_f64().unwrap_or(0.0) / 1000.0;
        if !title.is_empty() {
            pairs.push((title, artist, dur));
        }
    }
    if pairs.is_empty() {
        return Err("В ссылке Spotify не нашлось треков".into());
    }

    // Один трек — сразу ищем лучшее совпадение на YouTube Music.
    if pairs.len() == 1 {
        let (title, artist, _) = &pairs[0];
        let q = format!("{artist} {title}");
        if let Ok(found) = search::ytm_search(state, &q).await {
            if let Some(first) = found.into_iter().next() {
                return Ok(vec![first]);
            }
        }
    }

    // Плейлист — «ленивые» треки: yt-dlp сам найдёт каждый при воспроизведении.
    Ok(pairs
        .into_iter()
        .map(|(title, artist, duration)| {
            let q = format!("ytsearch1:{artist} - {title}");
            Track {
                id: format!("sp:{}", key_for(&q)),
                url: q,
                title,
                artist,
                duration,
                thumb: cover.clone(),
                source: "link".into(),
                explicit: false,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::is_unavailable;

    #[test]
    fn detects_region_blocks() {
        assert!(is_unavailable("[youtube] mYNmpS3a1Ag: Video unavailable. This video is not available"));
        assert!(is_unavailable("The uploader has not made this video available in your country"));
        assert!(is_unavailable("Video unavailable. This video contains content from X, who has blocked it in your country on copyright grounds"));
        assert!(!is_unavailable("yt-dlp не ответил вовремя"));
        assert!(!is_unavailable("Unable to download webpage: timed out"));
    }
}
