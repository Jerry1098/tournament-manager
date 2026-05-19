use chrono::Utc;
use tauri::{AppHandle, State};
use crate::domain::lifecycle::require_phase;
use crate::domain::model::Phase;
use crate::domain::swiss;
use crate::error::AppError;
use crate::events;
use crate::state::SharedState;
use crate::storage;

/// Generates the entire group-phase schedule in one shot (all scheduling rounds
/// pre-calculated and batched into slots of n_tables matches each).
/// Can only be called once — when rounds are empty and phase is Setup.
#[tauri::command]
pub async fn generate_next_round(
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;

    require_phase(t, &Phase::Setup, "generate_next_round")?;

    if !t.rounds.is_empty() {
        return Err(AppError::InvalidArgument("Schedule is already generated".into()));
    }
    if t.teams.len() < 2 {
        return Err(AppError::InvalidArgument("Need at least 2 teams to generate a schedule".into()));
    }
    if t.config.tables.is_empty() {
        return Err(AppError::InvalidArgument("Need at least 1 table to generate a schedule".into()));
    }

    // Generates all rounds and stores them in t.rounds; sets phase = Group.
    swiss::generate_full_schedule(t);
    t.updated_at = Utc::now();

    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}

#[tauri::command]
pub async fn reassign_table(
    match_id: String,
    table_id: Option<String>,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;

    if let Some(ref tid) = table_id {
        if !t.config.tables.iter().any(|tbl| &tbl.id == tid) {
            return Err(AppError::NotFound(format!("table {tid}")));
        }
    }

    let mut found = false;
    for round in &mut t.rounds {
        for m in &mut round.matches {
            if m.id == match_id {
                m.table_id = table_id.clone();
                found = true;
                break;
            }
        }
        if found { break; }
    }

    if !found {
        return Err(AppError::NotFound(format!("match {match_id}")));
    }

    t.updated_at = Utc::now();
    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}
