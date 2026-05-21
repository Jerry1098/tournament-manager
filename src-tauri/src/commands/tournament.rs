use std::path::PathBuf;
use std::collections::HashSet;
use chrono::Utc;
use tauri::{AppHandle, State};
use crate::domain::model::{Phase, Tournament, TournamentConfig};
use crate::error::AppError;
use crate::events;
use crate::state::SharedState;
use crate::storage::{self, TournamentSummary};

fn new_id() -> String {
    ulid::Ulid::new().to_string()
}

fn mutate_and_persist<F>(
    state: &State<SharedState>,
    app: &AppHandle,
    f: F,
) -> Result<Tournament, AppError>
where
    F: FnOnce(&mut Tournament),
{
    let mut guard = state.lock().unwrap();
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    let prev_phase = t.phase.clone();
    t.updated_at = Utc::now();
    f(t);
    let new_phase = t.phase.clone();

    let path = guard
        .current_path
        .clone()
        .ok_or_else(|| AppError::Tauri("no save path".into()))?;

    storage::save(&guard.current.as_ref().unwrap(), &path)?;
    guard.dirty = false;

    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(app, &t_clone);
    if prev_phase != new_phase {
        events::emit_phase_changed(app, prev_phase, new_phase);
    }

    Ok(t_clone)
}

#[tauri::command]
pub async fn create_tournament(
    name: String,
    config: TournamentConfig,
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<Tournament, AppError> {
    let id = new_id();
    let now = Utc::now();

    // Regenerate table IDs on the Rust side so they are valid ULIDs
    let mut config = config;
    for tbl in &mut config.tables {
        tbl.id = new_id();
    }

    // Deduplicate name: if a tournament with this name already exists, append a number.
    let existing_names: HashSet<String> = storage::list_all(&app)
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.name)
        .collect();
    let name = if existing_names.contains(&name) {
        let mut n = 2u32;
        loop {
            let candidate = format!("{name} {n}");
            if !existing_names.contains(&candidate) {
                break candidate;
            }
            n += 1;
        }
    } else {
        name
    };

    let t = Tournament {
        schema_version: 1,
        id: id.clone(),
        name,
        created_at: now,
        updated_at: now,
        config,
        teams: vec![],
        phase: Phase::Setup,
        rounds: vec![],
        playoffs: None,
    };

    let path = storage::default_path(&app, &id)?;
    storage::save(&t, &path)?;

    let mut guard = state.lock().unwrap();
    guard.current = Some(t.clone());
    guard.current_path = Some(path);
    guard.dirty = false;
    drop(guard);

    events::emit_tournament_updated(&app, &t);
    Ok(t)
}

#[tauri::command]
pub async fn load_tournament(
    path: Option<PathBuf>,
    app: AppHandle,
    state: State<'_, SharedState>,
) -> Result<Tournament, AppError> {
    let resolved_path = match path {
        Some(p) => p,
        None => {
            // No path provided: return error; frontend will open dialog and re-call with path
            return Err(AppError::InvalidArgument("path required".into()));
        }
    };

    let t = storage::load(&resolved_path)?;
    let mut guard = state.lock().unwrap();
    guard.current = Some(t.clone());
    guard.current_path = Some(resolved_path);
    guard.dirty = false;
    drop(guard);

    events::emit_tournament_updated(&app, &t);
    Ok(t)
}

#[tauri::command]
pub async fn save_tournament_as(
    path: PathBuf,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let guard = state.lock().unwrap();
    let t = guard.current.as_ref().ok_or(AppError::NoTournament)?;
    storage::save(t, &path)?;
    drop(guard);

    let mut guard = state.lock().unwrap();
    guard.current_path = Some(path.clone());
    guard.dirty = false;
    events::emit_dirty_changed(&app, false, path.to_str().map(|s| s.to_string()));
    Ok(())
}

#[tauri::command]
pub async fn close_tournament(
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    guard.current = None;
    guard.current_path = None;
    guard.dirty = false;
    drop(guard);
    events::emit_tournament_closed(&app);
    Ok(())
}

#[tauri::command]
pub async fn get_tournament(state: State<'_, SharedState>) -> Result<Option<Tournament>, AppError> {
    let guard = state.lock().unwrap();
    Ok(guard.current.clone())
}

#[tauri::command]
pub async fn rename_tournament(
    name: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<Tournament, AppError> {
    mutate_and_persist(&state, &app, |t| {
        t.name = name;
    })
}

#[tauri::command]
pub async fn list_tournaments(app: AppHandle) -> Result<Vec<TournamentSummary>, AppError> {
    storage::list_all(&app)
}

#[tauri::command]
pub async fn delete_tournament(
    path: PathBuf,
    state: State<'_, SharedState>,
) -> Result<(), AppError> {
    let guard = state.lock().unwrap();
    if let Some(current_path) = &guard.current_path {
        if *current_path == path {
            return Err(AppError::InvalidArgument(
                "Cannot delete the currently open tournament".into(),
            ));
        }
    }
    drop(guard);
    std::fs::remove_file(&path)?;
    Ok(())
}
