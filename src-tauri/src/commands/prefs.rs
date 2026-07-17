use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppPrefs {
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub tray_hint_shown: bool,
    #[serde(default)]
    pub close_choice_made: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AppPrefs {
    fn default() -> Self {
        Self {
            minimize_to_tray: true,
            tray_hint_shown: false,
            close_choice_made: true,
        }
    }
}

#[derive(Default)]
pub struct PrefsState {
    pub prefs: Mutex<AppPrefs>,
}

fn prefs_path(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::Other(e.to_string()))?;
    std::fs::create_dir_all(&dir).ok();
    Ok(dir.join("prefs.json"))
}

pub fn load_prefs_sync(app: &AppHandle) -> AppPrefs {
    let Ok(path) = prefs_path(app) else {
        return AppPrefs::default();
    };
    if !path.exists() {
        return AppPrefs::default();
    }
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return AppPrefs::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_prefs_sync(app: &AppHandle, prefs: &AppPrefs) -> AppResult<()> {
    let path = prefs_path(app)?;
    let raw = serde_json::to_string_pretty(prefs)?;
    std::fs::write(&path, raw)?;
    Ok(())
}

#[tauri::command]
pub async fn prefs_get(state: State<'_, PrefsState>) -> AppResult<AppPrefs> {
    let guard = state
        .prefs
        .lock()
        .map_err(|_| AppError::Other("prefs lock poisoned".into()))?;
    Ok(guard.clone())
}

#[tauri::command]
pub async fn prefs_set_minimize_to_tray(
    app: AppHandle,
    state: State<'_, PrefsState>,
    minimize_to_tray: bool,
) -> AppResult<AppPrefs> {
    let updated = {
        let mut guard = state
            .prefs
            .lock()
            .map_err(|_| AppError::Other("prefs lock poisoned".into()))?;
        guard.minimize_to_tray = minimize_to_tray;
        guard.clone()
    };
    save_prefs_sync(&app, &updated)?;
    Ok(updated)
}

#[tauri::command]
pub async fn prefs_mark_tray_hint_shown(
    app: AppHandle,
    state: State<'_, PrefsState>,
) -> AppResult<AppPrefs> {
    let updated = {
        let mut guard = state
            .prefs
            .lock()
            .map_err(|_| AppError::Other("prefs lock poisoned".into()))?;
        guard.tray_hint_shown = true;
        guard.clone()
    };
    save_prefs_sync(&app, &updated)?;
    Ok(updated)
}

#[tauri::command]
pub async fn prefs_set_close_choice(
    app: AppHandle,
    state: State<'_, PrefsState>,
    minimize_to_tray: bool,
) -> AppResult<AppPrefs> {
    let updated = {
        let mut guard = state
            .prefs
            .lock()
            .map_err(|_| AppError::Other("prefs lock poisoned".into()))?;
        guard.minimize_to_tray = minimize_to_tray;
        guard.close_choice_made = true;
        guard.clone()
    };
    save_prefs_sync(&app, &updated)?;
    Ok(updated)
}
