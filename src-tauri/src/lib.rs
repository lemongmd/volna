mod deps;
mod search;
mod stream;
mod ytdlp;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
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
    pub page_url: String,
    pub kind: StreamKind,
    pub mime: String,
    pub duration: f64,
    pub created: std::time::Instant,
}

pub struct Inner {
    pub bin_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub ytdlp_cache: PathBuf,
    http: RwLock<reqwest::Client>,
    pub proxy: RwLock<Option<String>>,
    pub streams: Mutex<HashMap<String, StreamEntry>>,
    pub locks: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
    pub sc_client_id: Mutex<Option<String>>,
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
    /// HTTP-клиент (с прокси, если он задан в настройках). Клонировать дёшево.
    pub fn http(&self) -> reqwest::Client {
        self.http.read().unwrap().clone()
    }
}

pub fn build_client(proxy: Option<&str>) -> Result<reqwest::Client, String> {
    let mut b = reqwest::Client::builder()
        .user_agent(UA)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(60));
    if let Some(p) = proxy {
        b = b.proxy(reqwest::Proxy::all(p).map_err(|e| format!("Неверный адрес прокси: {e}"))?);
    }
    b.build().map_err(|e| e.to_string())
}

/// Прокси для поиска, yt-dlp и стриминга (например, если YouTube замедлен).
#[tauri::command]
fn set_proxy(state: tauri::State<'_, AppState>, proxy: String) -> Result<(), String> {
    let p = proxy.trim();
    let p = if p.is_empty() { None } else { Some(p.to_string()) };
    let client = build_client(p.as_deref())?;
    *state.http.write().unwrap() = client;
    *state.proxy.write().unwrap() = p;
    // ссылки на потоки привязаны к IP — после смены прокси их нужно получить заново
    state.streams.lock().unwrap().clear();
    Ok(())
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

            let http = build_client(None)?;

            app.manage(AppState(Arc::new(Inner {
                bin_dir,
                cache_dir,
                ytdlp_cache,
                http: RwLock::new(http),
                proxy: RwLock::new(None),
                streams: Mutex::new(HashMap::new()),
                locks: Mutex::new(HashMap::new()),
                sc_client_id: Mutex::new(None),
            })));
            Ok(())
        })
        .register_asynchronous_uri_scheme_protocol("stream", |ctx, request, responder| {
            let state = ctx.app_handle().state::<AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                responder.respond(stream::serve(state, request).await);
            });
        })
        .invoke_handler(tauri::generate_handler![
            set_proxy,
            deps::deps_status,
            deps::install_deps,
            deps::update_engine,
            search::search,
            ytdlp::resolve,
            ytdlp::resolve_link,
        ])
        .run(tauri::generate_context!())
        .expect("не удалось запустить Волну");
}
