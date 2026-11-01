//! Протокол `img://<url>` — обложки грузятся через Rust: идут через прокси из
//! настроек (в РФ i.ytimg.com замедлен вместе с YouTube) и кэшируются в памяти.

use std::collections::VecDeque;
use std::sync::Arc;

use tauri::http::{header, Request, Response, StatusCode};

use crate::AppState;

const MAX_ITEMS: usize = 400;
const MAX_BYTES: usize = 60 * 1024 * 1024;

#[derive(Default)]
pub struct ImageCache {
    order: VecDeque<String>,
    map: std::collections::HashMap<String, (String, Arc<Vec<u8>>)>,
    bytes: usize,
}

impl ImageCache {
    fn get(&self, url: &str) -> Option<(String, Arc<Vec<u8>>)> {
        self.map.get(url).cloned()
    }

    fn put(&mut self, url: String, mime: String, data: Arc<Vec<u8>>) {
        if self.map.contains_key(&url) {
            return;
        }
        self.bytes += data.len();
        self.order.push_back(url.clone());
        self.map.insert(url, (mime, data));
        while self.order.len() > MAX_ITEMS || self.bytes > MAX_BYTES {
            let Some(old) = self.order.pop_front() else { break };
            if let Some((_, d)) = self.map.remove(&old) {
                self.bytes -= d.len();
            }
        }
    }
}

fn respond(status: StatusCode, mime: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, mime)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::CACHE_CONTROL, "max-age=86400")
        .body(body)
        .unwrap()
}

pub async fn serve(state: AppState, req: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let raw = req.uri().path().trim_start_matches('/');
    let url = match urlencoding::decode(raw) {
        Ok(u) => u.into_owned(),
        Err(_) => return respond(StatusCode::BAD_REQUEST, "text/plain", b"bad url".to_vec()),
    };
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return respond(StatusCode::BAD_REQUEST, "text/plain", b"bad url".to_vec());
    }

    if let Some((mime, data)) = state.images.lock().unwrap().get(&url) {
        return respond(StatusCode::OK, &mime, data.as_ref().clone());
    }

    let resp = match state
        .http()
        .get(&url)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => return respond(StatusCode::BAD_GATEWAY, "text/plain", format!("upstream {}", r.status()).into_bytes()),
        Err(e) => return respond(StatusCode::BAD_GATEWAY, "text/plain", e.to_string().into_bytes()),
    };
    let mime = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .filter(|m| m.starts_with("image/"))
        .unwrap_or("image/jpeg")
        .to_string();
    let data = match resp.bytes().await {
        Ok(b) => b.to_vec(),
        Err(e) => return respond(StatusCode::BAD_GATEWAY, "text/plain", e.to_string().into_bytes()),
    };
    let data = Arc::new(data);
    state.images.lock().unwrap().put(url, mime.clone(), data.clone());
    respond(StatusCode::OK, &mime, data.as_ref().clone())
}
