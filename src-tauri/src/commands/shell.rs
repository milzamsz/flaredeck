use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::error::AppResult;

#[tauri::command]
pub async fn shell_open_external(app: AppHandle, url: String) -> AppResult<()> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string().into())
}

#[tauri::command]
pub async fn shell_open_path(app: AppHandle, path: String) -> AppResult<()> {
    let target = std::path::Path::new(&path);
    let to_open = if target.is_file() {
        target
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path.clone())
    } else {
        path.clone()
    };
    app.opener()
        .open_path(to_open, None::<&str>)
        .map_err(|e| e.to_string().into())
}
