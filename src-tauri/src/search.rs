//! Поиск: YouTube Music и YouTube через InnerTube (быстро, без ключей),
//! SoundCloud через api-v2. Если быстрый путь сломался — запасной через yt-dlp.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use crate::{ytdlp, AppState};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Track {
    pub id: String,
    pub url: String,
    pub title: String,
    pub artist: String,
    pub duration: f64,
    pub thumb: String,
    pub source: String,
    pub explicit: bool,
}

pub(crate) const YTM_VERSION: &str = "1.20250929.03.00";
const YT_VERSION: &str = "2.20250929.01.00";

#[tauri::command]
pub async fn search(state: State<'_, AppState>, query: String, source: String) -> Result<Vec<Track>, String> {
    let state = state.inner().clone();
    let q = query.trim();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let fast = match source.as_str() {
        "yt" => yt_search(&state, q).await,
        "sc" => sc_search(&state, q).await,
        _ => ytm_search(&state, q).await,
    };
    match fast {
        Ok(v) if !v.is_empty() => Ok(v),
        _ => ytdlp_search(&state, q, &source).await,
    }
}

// ---------- общие помощники ----------

pub(crate) fn text_of(v: &Value) -> String {
    if let Some(s) = v["simpleText"].as_str() {
        return s.to_string();
    }
    v["runs"]
        .as_array()
        .map(|r| r.iter().filter_map(|x| x["text"].as_str()).collect::<String>())
        .unwrap_or_default()
}

fn parse_duration(s: &str) -> f64 {
    let parts: Vec<&str> = s.trim().split(':').collect();
    if parts.is_empty() || parts.len() > 3 {
        return 0.0;
    }
    let mut total = 0.0;
    for p in parts {
        match p.parse::<f64>() {
            Ok(n) => total = total * 60.0 + n,
            Err(_) => return 0.0,
        }
    }
    total
}

fn is_duration(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty() && s.contains(':') && s.split(':').all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// Рекурсивно собирает все объекты с заданным ключом — так парсер не ломается,
/// когда YouTube переставляет обёртки в ответе.
pub(crate) fn collect<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(m) => {
            for (k, val) in m {
                if k == key {
                    out.push(val);
                } else {
                    collect(val, key, out);
                }
            }
        }
        Value::Array(a) => {
            for x in a {
                collect(x, key, out);
            }
        }
        _ => {}
    }
}

fn upscale_ytm_thumb(url: &str) -> String {
    // https://lh3.googleusercontent.com/...=w120-h120-l90-rj → 544x544
    if let Some(pos) = url.rfind("=w") {
        let tail = &url[pos + 2..];
        if tail.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return format!("{}=w544-h544-l90-rj", &url[..pos]);
        }
    }
    url.to_string()
}

// ---------- YouTube Music ----------

pub async fn ytm_search(state: &AppState, q: &str) -> Result<Vec<Track>, String> {
    let body = json!({
        "context": { "client": { "clientName": "WEB_REMIX", "clientVersion": YTM_VERSION, "hl": "ru" } },
        "query": q,
        // фильтр «Песни»
        "params": "EgWKAQIIAWoMEA4QChADEAQQCRAF"
    });
    let resp: Value = state
        .http()
        .post("https://music.youtube.com/youtubei/v1/search?prettyPrint=false")
        .header("Origin", "https://music.youtube.com")
        .header("Referer", "https://music.youtube.com/")
        .header("X-Youtube-Client-Name", "67")
        .header("X-Youtube-Client-Version", YTM_VERSION)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    collect(&resp, "musicResponsiveListItemRenderer", &mut items);
    let mut out: Vec<Track> = Vec::new();
    for it in items {
        let cols = it["flexColumns"].as_array().cloned().unwrap_or_default();
        let col = |i: usize| -> &Value {
            cols.get(i)
                .map(|c| &c["musicResponsiveListItemFlexColumnRenderer"]["text"])
                .unwrap_or(&Value::Null)
        };
        let first = col(0);
        let title = text_of(first);
        let video_id = it["playlistItemData"]["videoId"]
            .as_str()
            .or_else(|| first["runs"][0]["navigationEndpoint"]["watchEndpoint"]["videoId"].as_str())
            .or_else(|| {
                it["overlay"]["musicItemThumbnailOverlayRenderer"]["content"]["musicPlayButtonRenderer"]
                    ["playNavigationEndpoint"]["watchEndpoint"]["videoId"]
                    .as_str()
            });
        let Some(video_id) = video_id else { continue };
        if title.is_empty() || out.iter().any(|t| t.id.ends_with(video_id)) {
            continue;
        }
        let second = text_of(col(1));
        let parts: Vec<&str> = second.split(" • ").collect();
        let mut artist = parts.first().copied().unwrap_or("").to_string();
        // Для «Песен» первая часть иногда — слово «Песня»
        if (artist == "Песня" || artist == "Song") && parts.len() > 1 {
            artist = parts[1].to_string();
        }
        let mut duration = parts
            .iter()
            .rev()
            .find(|p| is_duration(p))
            .map(|p| parse_duration(p))
            .unwrap_or(0.0);
        if duration == 0.0 {
            for i in 2..cols.len() {
                let t = text_of(col(i));
                if is_duration(&t) {
                    duration = parse_duration(&t);
                }
            }
        }
        let thumb = it["thumbnail"]["musicThumbnailRenderer"]["thumbnail"]["thumbnails"]
            .as_array()
            .and_then(|a| a.last())
            .and_then(|t| t["url"].as_str())
            .map(upscale_ytm_thumb)
            .unwrap_or_else(|| format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg"));
        let explicit = it["badges"]
            .as_array()
            .map(|b| {
                b.iter().any(|x| {
                    x["musicInlineBadgeRenderer"]["icon"]["iconType"].as_str() == Some("MUSIC_EXPLICIT_BADGE")
                })
            })
            .unwrap_or(false);
        out.push(Track {
            id: format!("ytm:{video_id}"),
            url: format!("https://www.youtube.com/watch?v={video_id}"),
            title,
            artist,
            duration,
            thumb,
            source: "ytm".into(),
            explicit,
        });
    }
    Ok(out)
}

// ---------- YouTube ----------

async fn yt_search(state: &AppState, q: &str) -> Result<Vec<Track>, String> {
    let body = json!({
        "context": { "client": { "clientName": "WEB", "clientVersion": YT_VERSION, "hl": "ru" } },
        "query": q,
        // только видео
        "params": "EgIQAQ=="
    });
    let resp: Value = state
        .http()
        .post("https://www.youtube.com/youtubei/v1/search?prettyPrint=false")
        .header("Origin", "https://www.youtube.com")
        .header("Referer", "https://www.youtube.com/")
        .header("X-Youtube-Client-Name", "1")
        .header("X-Youtube-Client-Version", YT_VERSION)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    collect(&resp, "videoRenderer", &mut items);
    let mut out: Vec<Track> = Vec::new();
    for v in items {
        let Some(id) = v["videoId"].as_str() else { continue };
        if out.iter().any(|t| t.id.ends_with(id)) {
            continue;
        }
        let duration = parse_duration(&text_of(&v["lengthText"]));
        if duration == 0.0 {
            continue; // стримы и премьеры
        }
        let mut artist = text_of(&v["ownerText"]);
        if artist.is_empty() {
            artist = text_of(&v["longBylineText"]);
        }
        out.push(Track {
            id: format!("yt:{id}"),
            url: format!("https://www.youtube.com/watch?v={id}"),
            title: text_of(&v["title"]),
            artist: artist.trim_end_matches(" - Topic").to_string(),
            duration,
            thumb: format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg"),
            source: "yt".into(),
            explicit: false,
        });
    }
    Ok(out)
}

// ---------- SoundCloud ----------

async fn sc_client_id(state: &AppState, refresh: bool) -> Result<String, String> {
    if !refresh {
        if let Some(id) = state.sc_client_id.lock().unwrap().clone() {
            return Ok(id);
        }
    }
    let html = state
        .http()
        .get("https://soundcloud.com/")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let mut scripts: Vec<String> = html
        .split("<script crossorigin src=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .filter(|s| s.contains("sndcdn.com"))
        .map(String::from)
        .collect();
    scripts.reverse();
    for src in scripts {
        let Ok(resp) = state.http().get(&src).send().await else { continue };
        let Ok(js) = resp.text().await else { continue };
        for marker in ["client_id:\"", "client_id=\"", "?client_id="] {
            if let Some(pos) = js.find(marker) {
                let id: String = js[pos + marker.len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric())
                    .collect();
                if id.len() >= 20 {
                    *state.sc_client_id.lock().unwrap() = Some(id.clone());
                    return Ok(id);
                }
            }
        }
    }
    Err("Не удалось получить ключ SoundCloud".into())
}

pub(crate) async fn sc_search(state: &AppState, q: &str) -> Result<Vec<Track>, String> {
    let mut last_err = String::new();
    for attempt in 0..2 {
        let cid = sc_client_id(state, attempt > 0).await?;
        let url = format!(
            "https://api-v2.soundcloud.com/search/tracks?q={}&client_id={}&limit=30&linked_partitioning=1",
            urlencoding::encode(q),
            cid
        );
        let resp = state.http().get(&url).send().await.map_err(|e| e.to_string())?;
        if resp.status().as_u16() == 401 || resp.status().as_u16() == 403 {
            last_err = "SoundCloud отклонил ключ".into();
            continue;
        }
        let v: Value = resp.json().await.map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for t in v["collection"].as_array().cloned().unwrap_or_default() {
            let Some(link) = t["permalink_url"].as_str() else { continue };
            let art = t["artwork_url"]
                .as_str()
                .or_else(|| t["user"]["avatar_url"].as_str())
                .unwrap_or_default()
                .replace("-large.", "-t500x500.");
            out.push(Track {
                id: format!("sc:{}", t["id"].as_u64().unwrap_or(0)),
                url: link.to_string(),
                title: t["title"].as_str().unwrap_or("Без названия").to_string(),
                artist: t["user"]["username"].as_str().unwrap_or("").to_string(),
                duration: t["full_duration"]
                    .as_f64()
                    .or_else(|| t["duration"].as_f64())
                    .unwrap_or(0.0)
                    / 1000.0,
                thumb: art,
                source: "sc".into(),
                explicit: false,
            });
        }
        return Ok(out);
    }
    Err(last_err)
}

// ---------- запасной путь: yt-dlp ----------

async fn ytdlp_search(state: &AppState, q: &str, source: &str) -> Result<Vec<Track>, String> {
    let prefix = if source == "sc" { "scsearch25" } else { "ytsearch25" };
    let query = format!("{prefix}:{q}");
    let out = ytdlp::run(state, &["--flat-playlist", "-J", "--", &query], 60).await?;
    let v: Value = serde_json::from_str(out.trim()).map_err(|e| e.to_string())?;
    let mut res = Vec::new();
    for e in v["entries"].as_array().cloned().unwrap_or_default() {
        let id = e["id"].as_str().unwrap_or_default().to_string();
        let url = if source == "sc" {
            e["url"].as_str().or_else(|| e["webpage_url"].as_str()).unwrap_or_default().to_string()
        } else {
            format!("https://www.youtube.com/watch?v={id}")
        };
        if url.is_empty() || id.is_empty() {
            continue;
        }
        let thumb = e["thumbnails"]
            .as_array()
            .and_then(|a| a.last())
            .and_then(|t| t["url"].as_str())
            .map(String::from)
            .unwrap_or_else(|| {
                if source == "sc" {
                    String::new()
                } else {
                    format!("https://i.ytimg.com/vi/{id}/hqdefault.jpg")
                }
            });
        let src = if source == "sc" { "sc" } else { source };
        res.push(Track {
            id: format!("{src}:{id}"),
            url,
            title: e["title"].as_str().unwrap_or("Без названия").to_string(),
            artist: e["uploader"]
                .as_str()
                .or_else(|| e["channel"].as_str())
                .unwrap_or("")
                .to_string(),
            duration: e["duration"].as_f64().unwrap_or(0.0),
            thumb,
            source: src.to_string(),
            explicit: false,
        });
    }
    Ok(res)
}
