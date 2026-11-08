//! Browser cookie source configuration for yt-dlp.

use std::path::Path;
use tauri::State;

use crate::AppState;

const BROWSERS: &[&str] = &["none", "chrome", "firefox", "edge", "brave"];

fn validate_browser(browser: &str) -> Result<&str, String> {
    BROWSERS
        .iter()
        .copied()
        .find(|candidate| *candidate == browser)
        .ok_or_else(|| "Неподдерживаемый браузер для cookies".to_string())
}

pub fn load_cookie_browser(path: &Path) -> String {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .and_then(|value| validate_browser(&value).ok().map(str::to_string))
        .unwrap_or_else(|| "none".to_string())
}

#[tauri::command]
pub fn set_cookie_browser(state: State<'_, AppState>, browser: String) -> Result<(), String> {
    let browser = validate_browser(&browser)?;
    if let Some(parent) = state.cookie_browser_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Не удалось создать каталог настроек: {e}"))?;
    }
    std::fs::write(&state.cookie_browser_path, browser)
        .map_err(|e| format!("Не удалось сохранить настройку cookies: {e}"))?;
    *state
        .cookie_browser
        .lock()
        .map_err(|_| "Не удалось обновить настройку cookies".to_string())? = browser.to_string();
    Ok(())
}
