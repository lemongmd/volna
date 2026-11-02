//! Протокол `stream://<ключ>` — локальный прокси для <audio>.
//! Отдаёт аудио кусками (HTTP Range), подставляя заголовки, которые требует
//! YouTube/SoundCloud, и сам обновляет протухшие ссылки.

use tauri::http::{header, Request, Response, StatusCode};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::{ytdlp, AppState, StreamKind};

/// Размер куска. <audio> сам догружает следующие.
const CHUNK: u64 = 1024 * 1024 + 512 * 1024;

fn base(status: StatusCode) -> tauri::http::response::Builder {
    Response::builder()
        .status(status)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, "Range")
        .header(
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            "Content-Range, Content-Length, Accept-Ranges",
        )
        .header(header::CACHE_CONTROL, "no-store")
}

fn error(status: StatusCode, msg: &str) -> Response<Vec<u8>> {
    base(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(msg.as_bytes().to_vec())
        .unwrap()
}

/// `bytes=START-END` → (start, Some(end))
fn parse_range(v: &str) -> Option<(u64, Option<u64>)> {
    let r = v.trim().strip_prefix("bytes=")?;
    let r = r.split(',').next()?;
    let (a, b) = r.split_once('-')?;
    let start = a.trim().parse::<u64>().ok()?;
    let end = b.trim().parse::<u64>().ok();
    Some((start, end))
}

fn total_from_content_range(v: &str) -> Option<u64> {
    v.rsplit('/').next()?.trim().parse().ok()
}

pub async fn serve(state: AppState, req: Request<Vec<u8>>) -> Response<Vec<u8>> {
    if req.method() == tauri::http::Method::OPTIONS {
        return base(StatusCode::NO_CONTENT).body(Vec::new()).unwrap();
    }
    let key = req
        .uri()
        .path()
        .trim_start_matches('/')
        .split(['/', '?'])
        .next()
        .unwrap_or_default()
        .to_string();
    let range = req
        .headers()
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(parse_range);
    let (start, req_end) = range.unwrap_or((0, None));

    for attempt in 0..2 {
        let entry = state.streams.lock().unwrap().get(&key).cloned();
        let Some(entry) = entry else {
            return error(StatusCode::NOT_FOUND, "stream not found");
        };

        match &entry.kind {
            StreamKind::File(path) => {
                return serve_file(path, &entry.mime, start, req_end).await;
            }
            StreamKind::Remote { url, headers } => {
                let mut end = start + CHUNK - 1;
                if let Some(e) = req_end {
                    end = end.min(e);
                }
                let mut rb = state.http().get(url);
                for (k, v) in headers {
                    // Accept-Encoding не пробрасываем — нам нужны сырые байты.
                    if k.eq_ignore_ascii_case("accept-encoding") {
                        continue;
                    }
                    rb = rb.header(k.as_str(), v.as_str());
                }
                rb = rb.header(header::RANGE, format!("bytes={start}-{end}"));
                let resp = match rb.send().await {
                    Ok(r) => r,
                    Err(e) => return error(StatusCode::BAD_GATEWAY, &e.to_string()),
                };
                let status = resp.status();
                if (status == reqwest::StatusCode::FORBIDDEN
                    || status == reqwest::StatusCode::GONE
                    || status == reqwest::StatusCode::NOT_FOUND)
                    && attempt == 0
                {
                    // Ссылка протухла — достаём свежую и пробуем ещё раз.
                    if ytdlp::refresh(&state, &key).await.is_ok() {
                        continue;
                    }
                }
                if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                    return base(StatusCode::RANGE_NOT_SATISFIABLE)
                        .body(Vec::new())
                        .unwrap();
                }
                if !status.is_success() {
                    return error(StatusCode::BAD_GATEWAY, &format!("upstream {status}"));
                }
                let content_range = resp
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|v| v.to_str().ok())
                    .map(String::from);
                let body = match resp.bytes().await {
                    Ok(b) => b.to_vec(),
                    Err(e) => return error(StatusCode::BAD_GATEWAY, &e.to_string()),
                };

                if status == reqwest::StatusCode::PARTIAL_CONTENT {
                    let total = content_range
                        .as_deref()
                        .and_then(total_from_content_range)
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "*".into());
                    let last = start + body.len() as u64 - 1;
                    return base(StatusCode::PARTIAL_CONTENT)
                        .header(header::CONTENT_TYPE, entry.mime.as_str())
                        .header(header::ACCEPT_RANGES, "bytes")
                        .header(header::CONTENT_RANGE, format!("bytes {start}-{last}/{total}"))
                        .header(header::CONTENT_LENGTH, body.len())
                        .body(body)
                        .unwrap();
                }

                // Сервер проигнорировал Range и прислал файл целиком.
                let total = body.len() as u64;
                if range.is_some() && start < total {
                    let last = req_end.unwrap_or(total - 1).min(total - 1);
                    let slice = body[start as usize..=last as usize].to_vec();
                    return base(StatusCode::PARTIAL_CONTENT)
                        .header(header::CONTENT_TYPE, entry.mime.as_str())
                        .header(header::ACCEPT_RANGES, "bytes")
                        .header(header::CONTENT_RANGE, format!("bytes {start}-{last}/{total}"))
                        .header(header::CONTENT_LENGTH, slice.len())
                        .body(slice)
                        .unwrap();
                }
                return base(StatusCode::OK)
                    .header(header::CONTENT_TYPE, entry.mime.as_str())
                    .header(header::ACCEPT_RANGES, "bytes")
                    .header(header::CONTENT_LENGTH, body.len())
                    .body(body)
                    .unwrap();
            }
        }
    }
    error(StatusCode::BAD_GATEWAY, "stream unavailable")
}

async fn serve_file(path: &std::path::Path, mime: &str, start: u64, req_end: Option<u64>) -> Response<Vec<u8>> {
    let mut file = match tokio::fs::File::open(path).await {
        Ok(f) => f,
        Err(e) => return error(StatusCode::NOT_FOUND, &e.to_string()),
    };
    let total = match file.metadata().await {
        Ok(m) => m.len(),
        Err(e) => return error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    };
    if total == 0 || start >= total {
        return base(StatusCode::RANGE_NOT_SATISFIABLE)
            .header(header::CONTENT_RANGE, format!("bytes */{total}"))
            .body(Vec::new())
            .unwrap();
    }
    let mut end = (start + CHUNK - 1).min(total - 1);
    if let Some(e) = req_end {
        end = end.min(e);
    }
    let len = (end - start + 1) as usize;
    let mut buf = vec![0u8; len];
    if file.seek(std::io::SeekFrom::Start(start)).await.is_err() || file.read_exact(&mut buf).await.is_err() {
        return error(StatusCode::INTERNAL_SERVER_ERROR, "read failed");
    }
    base(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, mime)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{total}"))
        .header(header::CONTENT_LENGTH, len)
        .body(buf)
        .unwrap()
}
