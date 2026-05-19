use chrono::Utc;
use tauri::{AppHandle, State};
use crate::domain::model::{Phase, Team};
use crate::domain::lifecycle::require_phase;
use crate::error::AppError;
use crate::events;
use crate::state::SharedState;
use crate::storage;

fn new_id() -> String {
    ulid::Ulid::new().to_string()
}

#[tauri::command]
pub async fn add_team(
    name: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<Team, AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    require_phase(t, &Phase::Setup, "add_team")?;

    let team = Team { id: new_id(), name };
    t.teams.push(team.clone());
    t.updated_at = Utc::now();

    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(&app, &t_clone);
    Ok(team)
}

#[tauri::command]
pub async fn rename_team(
    id: String,
    name: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    let team = t.teams.iter_mut().find(|x| x.id == id).ok_or_else(|| AppError::NotFound(format!("team {id}")))?;
    team.name = name;
    t.updated_at = Utc::now();

    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}

#[tauri::command]
pub async fn remove_team(
    id: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    require_phase(t, &Phase::Setup, "remove_team")?;
    let before = t.teams.len();
    t.teams.retain(|x| x.id != id);
    if t.teams.len() == before {
        return Err(AppError::NotFound(format!("team {id}")));
    }
    t.updated_at = Utc::now();

    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}
