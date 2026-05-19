use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use crate::error::AppError;

#[tauri::command]
pub async fn open_projector(app: AppHandle) -> Result<(), AppError> {
    if let Some(w) = app.get_webview_window("projector") {
        w.set_focus().ok();
        return Ok(());
    }

    WebviewWindowBuilder::new(&app, "projector", WebviewUrl::App("projector".into()))
        .title("Tournament — Projector")
        .inner_size(1280.0, 720.0)
        .resizable(true)
        .build()
        .map_err(|e| AppError::Tauri(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_fullscreen_projector(app: AppHandle) -> Result<(), AppError> {
    let w = app
        .get_webview_window("projector")
        .ok_or_else(|| AppError::NotFound("projector window".into()))?;
    let current = w.is_fullscreen().map_err(AppError::from)?;
    w.set_fullscreen(!current).map_err(AppError::from)?;
    Ok(())
}
