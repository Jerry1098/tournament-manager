use chrono::Utc;
use tauri::{AppHandle, State};
use crate::domain::bracket::{advance_bracket, build_playoffs};
use crate::domain::lifecycle::{find_playoff_match, require_phase};
use crate::domain::model::{MatchStatus, Phase, Playoffs};
use crate::error::AppError;
use crate::events;
use crate::state::SharedState;
use crate::storage;

#[tauri::command]
pub async fn start_playoffs(
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<Playoffs, AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    require_phase(t, &Phase::Group, "start_playoffs")?;

    let prev_phase = t.phase.clone();
    let playoffs = build_playoffs(t)?;
    t.playoffs = Some(playoffs.clone());
    t.phase = Phase::Playoffs;
    t.updated_at = Utc::now();

    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(&app, &t_clone);
    events::emit_phase_changed(&app, prev_phase, Phase::Playoffs);
    Ok(playoffs)
}

#[tauri::command]
pub async fn submit_playoff_result(
    match_id: String,
    cups_a: u32,
    cups_b: u32,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    if cups_a == cups_b {
        return Err(AppError::InvalidArgument("Cup counts cannot be equal".into()));
    }

    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    require_phase(t, &Phase::Playoffs, "submit_playoff_result")?;

    let m = find_playoff_match(t, &match_id)?;
    if m.status == MatchStatus::Completed {
        return Err(AppError::InvalidArgument("Match already completed".into()));
    }
    m.cups_a = cups_a;
    m.cups_b = cups_b;
    m.status = MatchStatus::Completed;
    m.completed_at = Some(Utc::now());

    advance_bracket(t, &match_id)?;

    let finished = {
        let playoffs = t.playoffs.as_ref().unwrap();
        let final_done = playoffs
            .bracket
            .iter()
            .find(|n| n.round == 0)
            .and_then(|n| playoffs.matches.iter().find(|m| m.id == n.match_id))
            .map(|m| m.status == MatchStatus::Completed)
            .unwrap_or(false);
        let third_done = playoffs
            .third_place_match
            .as_ref()
            .map(|m| m.status == MatchStatus::Completed)
            .unwrap_or(false);
        final_done && third_done
    };

    let prev_phase = t.phase.clone();
    if finished {
        t.phase = Phase::Finished;
    }
    t.updated_at = Utc::now();

    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(&app, &t_clone);
    if finished {
        events::emit_phase_changed(&app, prev_phase, Phase::Finished);
    }
    Ok(())
}
