use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use crate::domain::model::Tournament;
use crate::error::AppError;

pub fn default_tournament_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Tauri(e.to_string()))?;
    let dir = base.join("tournaments");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn default_path(app: &AppHandle, tournament_id: &str) -> Result<PathBuf, AppError> {
    Ok(default_tournament_dir(app)?.join(format!("{tournament_id}.json")))
}

/// Atomically write the tournament JSON: write to a .tmp file then rename.
pub fn save(tournament: &Tournament, path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    let json = serde_json::to_string_pretty(tournament)?;
    std::fs::write(&tmp, &json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn load(path: &Path) -> Result<Tournament, AppError> {
    let data = std::fs::read(path)?;
    let t: Tournament = serde_json::from_slice(&data)?;
    Ok(t)
}
