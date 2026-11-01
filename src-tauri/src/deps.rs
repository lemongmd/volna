//! Движок воспроизведения: yt-dlp (достаёт аудио с YouTube / SoundCloud / ещё
//! тысячи сайтов) и Deno (JS-рантайм, который yt-dlp нужен для YouTube).
//! Оба скачиваются один раз при первом запуске в папку данных приложения.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::io::AsyncWriteExt;

use crate::AppState;

#[cfg(windows)]
const YTDLP_FILE: &str = "yt-dlp.exe";
#[cfg(not(windows))]
const YTDLP_FILE: &str = "yt-dlp";

#[cfg(windows)]
const DENO_FILE: &str = "deno.exe";
#[cfg(not(windows))]
const DENO_FILE: &str = "deno";

/// Папка с распакованной (onedir) сборкой yt-dlp для Windows: она стартует
/// заметно быстрее однофайловой, которая распаковывает себя при каждом запуске.
const YTDLP_DIR: &str = "yt-dlp-win";
const YTDLP_WIN_ZIP: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_win.zip";

fn ytdlp_url() -> &'static str {
    if cfg!(windows) {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else if cfg!(target_os = "macos") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux"
    }
}

fn deno_url() -> &'static str {
    if cfg!(windows) {
        "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "https://github.com/denoland/deno/releases/latest/download/deno-aarch64-apple-darwin.zip"
        } else {
            "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-apple-darwin.zip"
        }
    } else {
        "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-unknown-linux-gnu.zip"
    }
}

/// Ищет исполняемый файл в PATH.
fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|p| p.join(name))
        .find(|p| p.is_file())
}

fn ytdlp_fast_path(state: &AppState) -> Option<PathBuf> {
    let p = state.bin_dir.join(YTDLP_DIR).join(YTDLP_FILE);
    p.is_file().then_some(p)
}

pub fn ytdlp_path(state: &AppState) -> Option<PathBuf> {
    if let Some(p) = ytdlp_fast_path(state) {
        return Some(p);
    }
    let local = state.bin_dir.join(YTDLP_FILE);
    if local.is_file() {
        return Some(local);
    }
    which(YTDLP_FILE)
}

fn deno_path(state: &AppState) -> Option<PathBuf> {
    let local = state.bin_dir.join(DENO_FILE);
    if local.is_file() {
        return Some(local);
    }
    which(DENO_FILE)
}

/// Команда для запуска yt-dlp: без консольного окна, с нашей папкой bin в PATH
/// (чтобы yt-dlp нашёл скачанный Deno).
pub fn ytdlp_command(state: &AppState) -> Result<tokio::process::Command, String> {
    let exe = ytdlp_path(state).ok_or("Движок yt-dlp не установлен")?;
    let mut cmd = tokio::process::Command::new(exe);
    let mut paths = vec![state.bin_dir.clone()];
    if let Some(p) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&p));
    }
    if let Ok(joined) = std::env::join_paths(paths) {
        cmd.env("PATH", joined);
    }
    cmd.env("PYTHONIOENCODING", "utf-8");
    cmd.env("PYTHONUTF8", "1");
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.kill_on_drop(true);
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    Ok(cmd)
}

#[derive(Serialize)]
pub struct DepsStatus {
    ytdlp: bool,
    /// Быстрая (распакованная) сборка yt-dlp. Вне Windows всегда true.
    fast: bool,
    deno: bool,
    version: Option<String>,
}

#[tauri::command]
pub async fn deps_status(state: State<'_, AppState>) -> Result<DepsStatus, String> {
    let state = state.inner().clone();
    let ytdlp = ytdlp_path(&state).is_some();
    let fast = !cfg!(windows) || ytdlp_fast_path(&state).is_some();
    let deno = deno_path(&state).is_some();
    let mut version = None;
    if ytdlp {
        if let Ok(mut cmd) = ytdlp_command(&state) {
            cmd.arg("--version");
            if let Ok(Ok(out)) =
                tokio::time::timeout(std::time::Duration::from_secs(20), cmd.output()).await
            {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !v.is_empty() {
                    version = Some(v);
                }
            }
        }
    }
    Ok(DepsStatus {
        ytdlp,
        fast,
        deno,
        version,
    })
}

#[derive(Serialize, Clone)]
struct Progress {
    stage: &'static str,
    received: u64,
    total: u64,
}

async fn download(
    app: &AppHandle,
    state: &AppState,
    url: &str,
    dest: &Path,
    stage: &'static str,
) -> Result<(), String> {
    let mut resp = state
        .http()
        .get(url)
        .timeout(std::time::Duration::from_secs(600))
        .send()
        .await
        .map_err(|e| format!("Не удалось скачать {stage}: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Не удалось скачать {stage}: HTTP {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(0);
    let tmp = dest.with_extension("part");
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| e.to_string())?;
    let mut received: u64 = 0;
    let mut last_emit: u64 = 0;
    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        received += chunk.len() as u64;
        if received - last_emit > 256 * 1024 || received == total {
            last_emit = received;
            let _ = app.emit(
                "deps-progress",
                Progress {
                    stage,
                    received,
                    total,
                },
            );
        }
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);
    let _ = tokio::fs::remove_file(dest).await;
    tokio::fs::rename(&tmp, dest)
        .await
        .map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755));
    }
    Ok(())
}

fn extract_deno(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut entry = archive
        .by_name(DENO_FILE)
        .map_err(|_| "В архиве нет deno".to_string())?;
    let mut out = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755));
    }
    Ok(())
}

/// Распаковывает весь архив в `dest` (через временную папку, чтобы не оставить
/// полусобранный движок при ошибке).
fn extract_all(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let tmp = dest.with_extension("tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(rel) = entry.enclosed_name() else { continue };
        let out = tmp.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = std::fs::File::create(&out).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
    }
    // В архиве yt-dlp.exe может лежать в корне или во вложенной папке.
    let root = if tmp.join(YTDLP_FILE).is_file() {
        tmp.clone()
    } else {
        std::fs::read_dir(&tmp)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.join(YTDLP_FILE).is_file())
            .ok_or("В архиве нет yt-dlp.exe")?
    };
    let _ = std::fs::remove_dir_all(dest);
    std::fs::rename(&root, dest).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(())
}

/// Ставит быструю сборку yt-dlp (только Windows).
async fn install_fast_ytdlp(app: &AppHandle, state: &AppState) -> Result<(), String> {
    let zip_path = state.bin_dir.join("yt-dlp_win.zip");
    download(app, state, YTDLP_WIN_ZIP, &zip_path, "yt-dlp").await?;
    let dest = state.bin_dir.join(YTDLP_DIR);
    let zp = zip_path.clone();
    let res = tauri::async_runtime::spawn_blocking(move || extract_all(&zp, &dest))
        .await
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(zip_path);
    res
}

/// Скачивает недостающие компоненты. Прогресс шлётся событием `deps-progress`.
#[tauri::command]
pub async fn install_deps(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let state = state.inner().clone();
    std::fs::create_dir_all(&state.bin_dir).map_err(|e| e.to_string())?;

    if cfg!(windows) && ytdlp_fast_path(&state).is_none() {
        // Если быстрая сборка не скачалась — откатываемся на обычный yt-dlp.exe.
        if install_fast_ytdlp(&app, &state).await.is_ok() {
            let _ = std::fs::remove_file(state.bin_dir.join(YTDLP_FILE));
        }
    }
    if ytdlp_path(&state).is_none() {
        let dest = state.bin_dir.join(YTDLP_FILE);
        download(&app, &state, ytdlp_url(), &dest, "yt-dlp").await?;
    }

    if deno_path(&state).is_none() {
        let zip_path = state.bin_dir.join("deno.zip");
        download(&app, &state, deno_url(), &zip_path, "deno").await?;
        let dest = state.bin_dir.join(DENO_FILE);
        let zp = zip_path.clone();
        tauri::async_runtime::spawn_blocking(move || extract_deno(&zp, &dest))
            .await
            .map_err(|e| e.to_string())??;
        let _ = std::fs::remove_file(zip_path);
    }
    Ok(())
}

/// Обновляет yt-dlp до последней версии (YouTube часто ломает старые версии).
#[tauri::command]
pub async fn update_engine(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    let state = state.inner().clone();
    if cfg!(windows) && (ytdlp_fast_path(&state).is_some() || state.bin_dir.join(YTDLP_FILE).is_file()) {
        install_fast_ytdlp(&app, &state).await?;
        let _ = std::fs::remove_file(state.bin_dir.join(YTDLP_FILE));
        return Ok("yt-dlp обновлён".into());
    }
    let local = state.bin_dir.join(YTDLP_FILE);
    if local.is_file() {
        // Перекачиваем свежий бинарник — надёжнее, чем `yt-dlp -U`.
        download(&app, &state, ytdlp_url(), &local, "yt-dlp").await?;
        return Ok("yt-dlp обновлён".into());
    }
    let mut cmd = ytdlp_command(&state)?;
    cmd.arg("-U");
    let out = cmd.output().await.map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(text.lines().last().unwrap_or("Готово").to_string())
}
