use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use crate::domain::model::Tournament;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TournamentSummary {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub path: String,
}

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

pub fn list_all(app: &AppHandle) -> Result<Vec<TournamentSummary>, AppError> {
    let dir = default_tournament_dir(app)?;
    let mut summaries = Vec::new();

    for entry in std::fs::read_dir(&dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Ok(t) = load(&path) {
            summaries.push(TournamentSummary {
                id: t.id,
                name: t.name,
                created_at: t.created_at,
                updated_at: t.updated_at,
                path: path.to_string_lossy().into_owned(),
            });
        }
    }

    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(summaries)
}
