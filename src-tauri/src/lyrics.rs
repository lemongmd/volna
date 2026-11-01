//! Тексты песен.
//! 1. LRCLIB (https://lrclib.net) — открытая база синхронизированных текстов
//!    (LRC с таймкодами), её пополняют сами слушатели. Сервер иногда отвечает
//!    500 — тогда повторяем запрос и перебираем другие варианты поиска.
//! 2. Запасной вариант — текст со страницы трека в YouTube Music (без таймкодов).
//! Genius не используем.

use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::search::{collect, text_of, YTM_VERSION};
use crate::AppState;

const LRCLIB_UA: &str = "Volna/0.1 (https://github.com/lemongmd/volna)";

#[derive(Serialize, Clone)]
pub struct Lyrics {
    synced: Option<String>,
    plain: Option<String>,
    instrumental: bool,
    source: String,
}

/// Убирает из названия «(Official Video)», «[Lyrics]», «| Премьера клипа» и т.п.
fn clean_title(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' | '[' | '【' => depth += 1,
            ')' | ']' | '】' => depth = (depth - 1).max(0),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    let out = out.split('|').next().unwrap_or("").to_string();
    let lower = out.to_lowercase();
    let mut cut = out.len();
    for marker in [" official", " lyric", " клип", " премьера", " prod.", " prod by", " visualizer", " текст"] {
        if let Some(p) = lower.find(marker) {
            cut = cut.min(p);
        }
    }
    out[..cut].trim().trim_matches(|c| c == '-' || c == '—' || c == ' ').trim().to_string()
}

fn clean_artist(s: &str) -> String {
    let s = s.trim_end_matches(" - Topic").trim_end_matches("VEVO");
    let lower = s.to_lowercase();
    let mut cut = s.len();
    for marker in [" feat", " ft.", " ft ", " x ", ", ", " & "] {
        if let Some(p) = lower.find(marker) {
            cut = cut.min(p);
        }
    }
    s[..cut].trim().to_string()
}

fn video_id(url: &str) -> Option<String> {
    let id = if let Some(p) = url.find("v=") {
        &url[p + 2..]
    } else if let Some(p) = url.find("youtu.be/") {
        &url[p + 9..]
    } else {
        return None;
    };
    let id: String = id.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
    (id.len() == 11).then_some(id)
}

fn pick(items: &[Value], duration: f64) -> Option<Value> {
    let dur_ok = |v: &Value, tol: f64| {
        duration <= 0.0 || (v["duration"].as_f64().unwrap_or(0.0) - duration).abs() <= tol
    };
    let has = |v: &Value, k: &str| v[k].as_str().map(|s| !s.trim().is_empty()).unwrap_or(false);
    items
        .iter()
        .find(|v| has(v, "syncedLyrics") && dur_ok(v, 4.0))
        .or_else(|| items.iter().find(|v| has(v, "syncedLyrics") && dur_ok(v, 12.0)))
        .or_else(|| items.iter().find(|v| has(v, "plainLyrics") && dur_ok(v, 12.0)))
        .or_else(|| items.iter().find(|v| v["instrumental"].as_bool() == Some(true) && dur_ok(v, 6.0)))
        .cloned()
}

fn to_lyrics(v: &Value, source: &str) -> Lyrics {
    let s = |k: &str| v[k].as_str().filter(|s| !s.trim().is_empty()).map(String::from);
    Lyrics {
        synced: s("syncedLyrics"),
        plain: s("plainLyrics"),
        instrumental: v["instrumental"].as_bool().unwrap_or(false),
        source: source.into(),
    }
}

enum Fail {
    /// Сервер ответил, но такого нет.
    NotFound,
    /// Сервер недоступен / 5xx / таймаут.
    Down(String),
}

/// GET к LRCLIB с повторами при 5xx и сетевых ошибках.
async fn lrclib(state: &AppState, path: &str, params: &[(&str, String)]) -> Result<Value, Fail> {
    let mut last = String::new();
    for attempt in 0..3u64 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(400 * attempt)).await;
        }
        let resp = state
            .http()
            .get(format!("https://lrclib.net{path}"))
            .query(params)
            .header("User-Agent", LRCLIB_UA)
            .header("Lrclib-Client", LRCLIB_UA)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;
        match resp {
            Ok(r) if r.status().is_success() => return r.json().await.map_err(|e| Fail::Down(e.to_string())),
            Ok(r) if r.status().as_u16() == 404 || r.status().is_client_error() => return Err(Fail::NotFound),
            Ok(r) => last = format!("сервер ответил {}", r.status()),
            Err(e) => last = e.to_string(),
        }
    }
    Err(Fail::Down(last))
}

async fn from_lrclib(state: &AppState, title: &str, artist: &str, duration: f64) -> Result<Option<Lyrics>, String> {
    let t = clean_title(title);
    let a = clean_artist(artist);

    // Варианты «исполнитель / название»: YouTube часто пишет «Артист - Трек» в названии.
    let mut variants: Vec<(String, String)> = Vec::new();
    for sep in [" - ", " — ", " – "] {
        if let Some((l, r)) = t.split_once(sep) {
            variants.push((clean_artist(l), clean_title(r)));
            break;
        }
    }
    if !t.is_empty() {
        variants.push((a, t));
    }

    let mut any_ok = false;
    let mut last_err = String::new();
    for (artist, track) in variants.iter().filter(|(_, t)| !t.is_empty()) {
        // 1) точное совпадение по длительности
        if !artist.is_empty() && duration > 0.0 {
            let params = [
                ("track_name", track.clone()),
                ("artist_name", artist.clone()),
                ("duration", (duration.round() as i64).to_string()),
            ];
            match lrclib(state, "/api/get", &params).await {
                Ok(v) => {
                    any_ok = true;
                    if pick(std::slice::from_ref(&v), duration).is_some() {
                        return Ok(Some(to_lyrics(&v, "LRCLIB")));
                    }
                }
                Err(Fail::NotFound) => any_ok = true,
                Err(Fail::Down(e)) => last_err = e,
            }
        }
        // 2) поиск по полям и свободный поиск
        let mut searches = vec![vec![("track_name", track.clone())]];
        if !artist.is_empty() {
            searches[0].push(("artist_name", artist.clone()));
        }
        searches.push(vec![("q", format!("{artist} {track}").trim().to_string())]);
        for params in searches {
            match lrclib(state, "/api/search", &params).await {
                Ok(v) => {
                    any_ok = true;
                    if let Some(hit) = pick(v.as_array().map(|a| a.as_slice()).unwrap_or(&[]), duration) {
                        return Ok(Some(to_lyrics(&hit, "LRCLIB")));
                    }
                }
                Err(Fail::NotFound) => any_ok = true,
                Err(Fail::Down(e)) => last_err = e,
            }
        }
    }
    if any_ok {
        Ok(None)
    } else {
        Err(last_err)
    }
}

async fn ytm_post(state: &AppState, endpoint: &str, body: Value) -> Result<Value, String> {
    state
        .http()
        .post(format!("https://music.youtube.com/youtubei/v1/{endpoint}?prettyPrint=false"))
        .header("Origin", "https://music.youtube.com")
        .header("Referer", "https://music.youtube.com/")
        .header("X-Youtube-Client-Name", "67")
        .header("X-Youtube-Client-Version", YTM_VERSION)
        .timeout(std::time::Duration::from_secs(10))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Текст со страницы трека в YouTube Music (вкладка «Текст»).
async fn from_ytm(state: &AppState, video_id: &str) -> Result<Option<Lyrics>, String> {
    let ctx = json!({ "client": { "clientName": "WEB_REMIX", "clientVersion": YTM_VERSION, "hl": "ru" } });
    let next = ytm_post(state, "next", json!({ "context": ctx, "videoId": video_id, "isAudioOnly": true })).await?;
    let mut eps = Vec::new();
    collect(&next, "browseEndpoint", &mut eps);
    let Some(browse_id) = eps
        .iter()
        .filter_map(|e| e["browseId"].as_str())
        .find(|id| id.starts_with("MPLYt"))
        .map(String::from)
    else {
        return Ok(None);
    };
    let page = ytm_post(state, "browse", json!({ "context": ctx, "browseId": browse_id })).await?;
    let mut shelves = Vec::new();
    collect(&page, "musicDescriptionShelfRenderer", &mut shelves);
    let text = shelves
        .iter()
        .map(|s| text_of(&s["description"]))
        .find(|t| !t.trim().is_empty());
    Ok(text.map(|t| Lyrics {
        synced: None,
        plain: Some(t),
        instrumental: false,
        source: "YouTube Music".into(),
    }))
}

#[tauri::command]
pub async fn lyrics(
    state: State<'_, AppState>,
    title: String,
    artist: String,
    duration: f64,
    url: Option<String>,
) -> Result<Option<Lyrics>, String> {
    let state = state.inner().clone();
    let lrc = from_lrclib(&state, &title, &artist, duration).await;
    if let Ok(Some(l)) = &lrc {
        return Ok(Some(l.clone()));
    }
    if let Some(id) = url.as_deref().and_then(video_id) {
        if let Ok(Some(l)) = from_ytm(&state, &id).await {
            return Ok(Some(l));
        }
    }
    match lrc {
        Err(e) => Err(format!("LRCLIB сейчас недоступен ({e}). Попробуй позже или загрузи свой .lrc")),
        Ok(_) => Ok(None),
    }
}


// ───────── поиск по одному названию (для «Оно?») ─────────

#[derive(Serialize, Clone)]
pub struct Candidate {
    id: i64,
    title: String,
    artist: String,
    album: String,
    duration: f64,
    synced: Option<String>,
    plain: Option<String>,
}

/// Для сравнения названий: только буквы и цифры в нижнем регистре.
fn norm(s: &str) -> String {
    s.chars().filter(|c| c.is_alphanumeric()).flat_map(|c| c.to_lowercase()).collect()
}

/// Когда точный поиск не нашёл текст (например, на SoundCloud исполнитель
/// указан иначе), ищем только по названию и отдаём несколько вариантов —
/// пользователь сам выберет нужный.
#[tauri::command]
pub async fn lyrics_candidates(
    state: State<'_, AppState>,
    title: String,
    duration: f64,
) -> Result<Vec<Candidate>, String> {
    let state = state.inner().clone();
    let full = clean_title(&title);
    let mut titles: Vec<String> = Vec::new();
    for sep in [" - ", " — ", " – "] {
        if let Some((_, r)) = full.split_once(sep) {
            titles.push(clean_title(r));
            break;
        }
    }
    titles.push(full);
    titles.retain(|t| !t.is_empty());
    titles.dedup();
    let want: Vec<String> = titles.iter().map(|t| norm(t)).collect();

    let mut found: Vec<Value> = Vec::new();
    let mut any_ok = false;
    let mut last_err = String::new();
    for t in &titles {
        for key in ["track_name", "q"] {
            match lrclib(&state, "/api/search", &[(key, t.clone())]).await {
                Ok(v) => {
                    any_ok = true;
                    for item in v.as_array().cloned().unwrap_or_default() {
                        let id = item["id"].as_i64().unwrap_or(0);
                        if id != 0 && !found.iter().any(|f| f["id"].as_i64() == Some(id)) {
                            found.push(item);
                        }
                    }
                }
                Err(Fail::NotFound) => any_ok = true,
                Err(Fail::Down(e)) => last_err = e,
            }
        }
    }
    if !any_ok {
        return Err(format!("LRCLIB сейчас недоступен ({last_err})"));
    }

    let has = |v: &Value, k: &str| v[k].as_str().map(|s| !s.trim().is_empty()).unwrap_or(false);
    let score = |v: &Value| -> i32 {
        let mut s = 0;
        if has(v, "syncedLyrics") {
            s += 3;
        }
        let name = norm(v["trackName"].as_str().unwrap_or(""));
        if want.iter().any(|w| *w == name) {
            s += 4;
        } else if want.iter().any(|w| !w.is_empty() && (name.contains(w.as_str()) || w.contains(name.as_str()))) {
            s += 1;
        }
        if duration > 0.0 {
            let d = (v["duration"].as_f64().unwrap_or(0.0) - duration).abs();
            s += if d <= 3.0 {
                4
            } else if d <= 10.0 {
                2
            } else if d > 40.0 {
                -2
            } else {
                0
            };
        }
        s
    };
    let mut items: Vec<Value> = found
        .into_iter()
        .filter(|v| has(v, "syncedLyrics") || has(v, "plainLyrics"))
        .collect();
    items.sort_by_key(|v| -score(v));
    Ok(items
        .into_iter()
        .take(4)
        .map(|v| {
            let s = |k: &str| v[k].as_str().filter(|s| !s.trim().is_empty()).map(String::from);
            Candidate {
                id: v["id"].as_i64().unwrap_or(0),
                title: s("trackName").unwrap_or_default(),
                artist: s("artistName").unwrap_or_default(),
                album: s("albumName").unwrap_or_default(),
                duration: v["duration"].as_f64().unwrap_or(0.0),
                synced: s("syncedLyrics"),
                plain: s("plainLyrics"),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_youtube_titles() {
        assert_eq!(clean_title("Артист - Песня (Official Video)"), "Артист - Песня");
        assert_eq!(clean_title("Песня [Премьера клипа, 2026]"), "Песня");
        assert_eq!(clean_title("Song | Lyric video"), "Song");
        assert_eq!(clean_title("Song Official Audio"), "Song");
        assert_eq!(clean_artist("Artist feat. Other"), "Artist");
        assert_eq!(clean_artist("Artist - Topic"), "Artist");
    }

    #[test]
    fn normalizes_titles() {
        assert_eq!(norm("Время на Веселье!"), norm("время на веселье"));
        assert_ne!(norm("Плывут"), norm("Плыву"));
    }

    #[test]
    fn extracts_video_id() {
        assert_eq!(video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").as_deref(), Some("dQw4w9WgXcQ"));
        assert_eq!(video_id("https://youtu.be/dQw4w9WgXcQ?t=3").as_deref(), Some("dQw4w9WgXcQ"));
        assert_eq!(video_id("https://soundcloud.com/a/b"), None);
    }
}
