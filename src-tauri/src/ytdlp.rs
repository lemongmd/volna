//! Получение аудиопотока через yt-dlp и разбор ссылок (YouTube, SoundCloud,
//! плейлисты, Spotify → поиск на YouTube).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
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
    if let Some(p) = state.proxy.read().unwrap().clone() {
        cmd.arg("--proxy").arg(p);
    }
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

/// Достаёт поток для страницы трека и кладёт в кэш. Возвращает ключ для
/// протокола `stream://`.
pub async fn resolve_inner(state: &AppState, url: &str, force: bool) -> Result<StreamEntry, String> {
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

    let out = run(
        state,
        &[
            "--no-playlist",
            "--playlist-items",
            "1",
            "-f",
            FORMAT,
            "--print",
            "%(.{url,http_headers,duration,ext,protocol,format_id})j",
            "--",
            url,
        ],
        90,
    )
    .await?;
    let line = out
        .lines()
        .find(|l| l.trim_start().starts_with('{'))
        .ok_or("yt-dlp не вернул поток")?;
    let info: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;

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
                url,
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

    let entry = StreamEntry {
        page_url: url.to_string(),
        kind,
        mime,
        duration,
        created: Instant::now(),
    };
    state.streams.lock().unwrap().insert(key, entry.clone());
    Ok(entry)
}

#[derive(Serialize)]
pub struct Resolved {
    key: String,
    duration: f64,
}

#[tauri::command]
pub async fn resolve(state: State<'_, AppState>, url: String, force: Option<bool>) -> Result<Resolved, String> {
    let state = state.inner().clone();
    let entry = resolve_inner(&state, &url, force.unwrap_or(false)).await?;
    Ok(Resolved {
        key: key_for(&url),
        duration: entry.duration,
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
