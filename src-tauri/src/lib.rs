mod deps;
mod images;
mod lyrics;
mod search;
mod stream;
mod ytdlp;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::Manager;

/// Откуда брать аудио для конкретного трека.
#[derive(Clone, Debug)]
pub enum StreamKind {
    /// Прямая ссылка на аудио (YouTube/SoundCloud progressive) + заголовки,
    /// которые требует сервер.
    Remote {
        url: String,
        headers: Vec<(String, String)>,
    },
    /// Трек, скачанный в кэш (для HLS/DASH, которые <audio> не умеет).
    File(PathBuf),
}

#[derive(Clone, Debug)]
pub struct StreamEntry {
    /// Страница, с которой реально берётся звук (оригинал или найденная замена).
    pub page_url: String,
    /// Откуда взята замена, если оригинал недоступен в регионе («YouTube»/«SoundCloud»).
    pub substitute: Option<String>,
    pub kind: StreamKind,
    pub mime: String,
    pub duration: f64,
    pub created: std::time::Instant,
}

pub struct Inner {
    pub bin_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub ytdlp_cache: PathBuf,
    pub cookie_browser: Mutex<String>,
    http: reqwest::Client,
    pub streams: Mutex<HashMap<String, StreamEntry>>,
    pub locks: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
    pub sc_client_id: Mutex<Option<String>>,
    pub images: Mutex<images::ImageCache>,
}

#[derive(Clone)]
pub struct AppState(pub Arc<Inner>);

impl std::ops::Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

impl Inner {
    /// Общий HTTP-клиент. Клонировать дёшево.
    pub fn http(&self) -> reqwest::Client {
        self.http.clone()
    }
}

pub fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(UA)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(8)
        .build()
        .map_err(|e| e.to_string())
}

pub const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let base = app
                .path()
                .app_local_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("volna"));
            let bin_dir = base.join("bin");
            let cache_dir = base.join("cache");
            let ytdlp_cache = base.join("ytdlp-cache");
            // Кэш треков чистим при каждом запуске — он нужен только на сессию.
            let _ = std::fs::remove_dir_all(&cache_dir);
            for d in [&bin_dir, &cache_dir, &ytdlp_cache] {
                let _ = std::fs::create_dir_all(d);
            }

            let http = build_client()?;

            app.manage(AppState(Arc::new(Inner {
                bin_dir,
                cache_dir,
                ytdlp_cache,
                cookie_browser: Mutex::new("none".into()),
                http,
                streams: Mutex::new(HashMap::new()),
                locks: Mutex::new(HashMap::new()),
                sc_client_id: Mutex::new(None),
                images: Mutex::new(images::ImageCache::default()),
            })));
            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("stream", |ctx, request, responder| {
            let state = ctx.app_handle().state::<AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                responder.respond(stream::serve(state, request).await);
            });
        })
        .register_asynchronous_uri_scheme_protocol("img", |ctx, request, responder| {
            let state = ctx.app_handle().state::<AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                responder.respond(images::serve(state, request).await);
            });
        })
        .invoke_handler(tauri::generate_handler![
            deps::deps_status,
            deps::install_deps,
            deps::update_engine,
            search::search,
            ytdlp::resolve,
            ytdlp::resolve_link,
            ytdlp::set_cookie_browser,
            lyrics::lyrics,
            lyrics::lyrics_candidates,
        ])
        .run(tauri::generate_context!())
        .expect("не удалось запустить Волну");
}
