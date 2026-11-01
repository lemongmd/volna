//! Тексты песен из LRCLIB (https://lrclib.net) — открытой базы, которую
//! пополняют сами слушатели. Там есть синхронизированные тексты (LRC) с
//! таймкодами, как в Apple Music. Genius не используем.

use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::AppState;

const UA: &str = "Volna/0.1 (https://github.com/lemongmd/volna)";

#[derive(Serialize, Clone)]
pub struct Lyrics {
    synced: Option<String>,
    plain: Option<String>,
    instrumental: bool,
    title: String,
    artist: String,
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

fn pick(items: &[Value], duration: f64) -> Option<Value> {
    let dur_ok = |v: &Value, tol: f64| {
        duration <= 0.0 || (v["duration"].as_f64().unwrap_or(0.0) - duration).abs() <= tol
    };
    let has_synced = |v: &Value| v["syncedLyrics"].as_str().map(|s| !s.trim().is_empty()).unwrap_or(false);
    let has_plain = |v: &Value| v["plainLyrics"].as_str().map(|s| !s.trim().is_empty()).unwrap_or(false);
    items
        .iter()
        .find(|v| has_synced(v) && dur_ok(v, 4.0))
        .or_else(|| items.iter().find(|v| has_synced(v) && dur_ok(v, 12.0)))
        .or_else(|| items.iter().find(|v| has_plain(v) && dur_ok(v, 12.0)))
        .or_else(|| items.iter().find(|v| v["instrumental"].as_bool() == Some(true) && dur_ok(v, 6.0)))
        .cloned()
}

async fn search(state: &AppState, params: &[(&str, &str)]) -> Result<Vec<Value>, String> {
    let resp = state
        .http()
        .get("https://lrclib.net/api/search")
        .query(params)
        .header("User-Agent", UA)
        .header("Lrclib-Client", UA)
        .timeout(std::time::Duration::from_secs(12))
        .send()
        .await
        .map_err(|e| format!("LRCLIB недоступен: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("LRCLIB ответил {}", resp.status()));
    }
    let v: Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(v.as_array().cloned().unwrap_or_default())
}

#[tauri::command]
pub async fn lyrics(
    state: State<'_, AppState>,
    title: String,
    artist: String,
    duration: f64,
) -> Result<Option<Lyrics>, String> {
    let state = state.inner().clone();
    let t = clean_title(&title);
    let a = clean_artist(&artist);

    // Варианты «исполнитель / название»: YouTube часто пишет «Артист - Трек» в названии.
    let mut variants: Vec<(String, String)> = Vec::new();
    for sep in [" - ", " — ", " – "] {
        if let Some((l, r)) = t.split_once(sep) {
            variants.push((clean_artist(l), clean_title(r)));
            break;
        }
    }
    if !t.is_empty() {
        variants.push((a.clone(), t.clone()));
    }

    let mut last_err = None;
    for (artist, track) in &variants {
        if track.is_empty() {
            continue;
        }
        let mut params = vec![("track_name", track.as_str())];
        if !artist.is_empty() {
            params.push(("artist_name", artist.as_str()));
        }
        let q = format!("{artist} {track}");
        for p in [params, vec![("q", q.as_str())]] {
            match search(&state, &p).await {
                Ok(items) => {
                    if let Some(v) = pick(&items, duration) {
                        let s = |k: &str| v[k].as_str().filter(|s| !s.trim().is_empty()).map(String::from);
                        return Ok(Some(Lyrics {
                            synced: s("syncedLyrics"),
                            plain: s("plainLyrics"),
                            instrumental: v["instrumental"].as_bool().unwrap_or(false),
                            title: s("trackName").unwrap_or_default(),
                            artist: s("artistName").unwrap_or_default(),
                        }));
                    }
                }
                Err(e) => last_err = Some(e),
            }
        }
    }
    match last_err {
        Some(e) => Err(e),
        None => Ok(None),
    }
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
}
