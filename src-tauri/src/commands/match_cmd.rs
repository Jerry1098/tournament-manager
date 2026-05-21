use chrono::Utc;
use tauri::{AppHandle, State};
use crate::domain::lifecycle::{find_group_match, in_progress_count};
use crate::domain::model::MatchStatus;
use crate::domain::standings::compute_standings;
use crate::error::AppError;
use crate::events;
use crate::state::SharedState;
use crate::storage;

#[tauri::command]
pub async fn start_match(
    match_id: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;

    // Find which round contains this match and enforce ordering.
    let round_idx = t.rounds.iter()
        .position(|r| r.matches.iter().any(|m| m.id == match_id))
        .ok_or_else(|| AppError::NotFound(format!("match {match_id}")))?;

    for i in 0..round_idx {
        if !t.rounds[i].matches.iter().all(|m| m.is_terminal()) {
            return Err(AppError::InvalidArgument(format!(
                "Round {} must be fully completed before starting round {} matches",
                i + 1, round_idx + 1
            )));
        }
    }

    let table_count = t.config.tables.len();
    let current_in_progress = in_progress_count(t);

    let m = find_group_match(t, &match_id)?;

    if m.status != MatchStatus::Scheduled {
        return Err(AppError::InvalidArgument(format!(
            "Match {} is not Scheduled (status: {:?})",
            match_id, m.status
        )));
    }
    if m.table_id.is_none() {
        return Err(AppError::InvalidArgument("Match must be assigned to a table before starting".into()));
    }
    if current_in_progress >= table_count {
        return Err(AppError::InvalidArgument(format!(
            "All {table_count} tables are already in use"
        )));
    }

    m.status = MatchStatus::InProgress;
    m.started_at = Some(Utc::now());
    m.paused_at = None;
    m.paused_elapsed_seconds = 0;
    let m_clone = m.clone();

    t.updated_at = Utc::now();
    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_match_updated(&app, &m_clone);
    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}

/// Start every Scheduled match in the given round that has a table assigned.
#[tauri::command]
pub async fn start_all_assigned_matches(
    round_index: usize,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<usize, AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;

    let round_idx = round_index;
    if round_idx >= t.rounds.len() {
        return Ok(0);
    }

    // Collect IDs of all scheduled matches that have a table assigned.
    let to_start: Vec<String> = t.rounds[round_idx].matches.iter()
        .filter(|m| m.status == MatchStatus::Scheduled && m.table_id.is_some())
        .map(|m| m.id.clone())
        .collect();

    // Enforce ordering: the previous round must be complete.
    if round_idx > 0 && !t.rounds[round_idx - 1].matches.iter().all(|m| m.is_terminal()) {
        return Ok(0); // silently skip — previous round not done
    }

    let count = to_start.len();
    if count == 0 {
        return Ok(0);
    }

    let now = Utc::now();
    for m in &mut t.rounds[round_idx].matches {
        if to_start.contains(&m.id) {
            m.status = MatchStatus::InProgress;
            m.started_at = Some(now);
            m.paused_at = None;
            m.paused_elapsed_seconds = 0;
        }
    }

    t.updated_at = now;
    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_tournament_updated(&app, &t_clone);
    Ok(count)
}

#[tauri::command]
pub async fn submit_result(
    match_id: String,
    cups_a: u32,
    cups_b: u32,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    if cups_a == cups_b {
        return Err(AppError::InvalidArgument("Cup counts cannot be equal (no ties in beerpong)".into()));
    }

    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    let m = find_group_match(t, &match_id)?;

    if m.status != MatchStatus::InProgress {
        return Err(AppError::InvalidArgument(format!(
            "Match {} is not InProgress", match_id
        )));
    }

    m.cups_a = cups_a;
    m.cups_b = cups_b;
    m.status = MatchStatus::Completed;
    m.completed_at = Some(Utc::now());
    m.paused_at = None;
    let m_clone = m.clone();

    t.updated_at = Utc::now();
    storage::save(t, &path)?;
    let standings = compute_standings(t);
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_match_updated(&app, &m_clone);
    events::emit_tournament_updated(&app, &t_clone);
    events::emit_standings_updated(&app, &standings);
    Ok(())
}

#[tauri::command]
pub async fn edit_result(
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
    let m = find_group_match(t, &match_id)?;

    if m.status != MatchStatus::Completed {
        return Err(AppError::InvalidArgument(format!(
            "Match {} is not Completed", match_id
        )));
    }

    m.cups_a = cups_a;
    m.cups_b = cups_b;
    m.completed_at = Some(Utc::now());
    let m_clone = m.clone();

    t.updated_at = Utc::now();
    storage::save(t, &path)?;
    let standings = compute_standings(t);
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_match_updated(&app, &m_clone);
    events::emit_tournament_updated(&app, &t_clone);
    events::emit_standings_updated(&app, &standings);
    Ok(())
}

#[tauri::command]
pub async fn cancel_match(
    match_id: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    let m = find_group_match(t, &match_id)?;

    if m.status != MatchStatus::InProgress {
        return Err(AppError::InvalidArgument(format!(
            "Match {} is not InProgress", match_id
        )));
    }

    m.status = MatchStatus::Scheduled;
    m.started_at = None;
    m.paused_at = None;
    m.paused_elapsed_seconds = 0;
    let m_clone = m.clone();

    t.updated_at = Utc::now();
    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_match_updated(&app, &m_clone);
    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}

#[tauri::command]
pub async fn pause_match_timer(
    match_id: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    let m = find_group_match(t, &match_id)?;

    if m.status != MatchStatus::InProgress {
        return Err(AppError::InvalidArgument("Can only pause InProgress matches".into()));
    }
    if m.paused_at.is_some() {
        return Err(AppError::InvalidArgument("Timer is already paused".into()));
    }

    m.paused_at = Some(Utc::now());
    let m_clone = m.clone();

    t.updated_at = Utc::now();
    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_match_updated(&app, &m_clone);
    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}

#[tauri::command]
pub async fn resume_match_timer(
    match_id: String,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    let m = find_group_match(t, &match_id)?;

    if m.status != MatchStatus::InProgress {
        return Err(AppError::InvalidArgument("Can only resume InProgress matches".into()));
    }
    let paused_at = m.paused_at.ok_or_else(|| AppError::InvalidArgument("Timer is not paused".into()))?;

    let paused_duration = (Utc::now() - paused_at).num_seconds().max(0) as u32;
    m.paused_elapsed_seconds = m.paused_elapsed_seconds.saturating_add(paused_duration);
    m.paused_at = None;
    let m_clone = m.clone();

    t.updated_at = Utc::now();
    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_match_updated(&app, &m_clone);
    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}

#[tauri::command]
pub async fn set_match_time_limit(
    match_id: String,
    seconds: Option<u32>,
    state: State<'_, SharedState>,
    app: AppHandle,
) -> Result<(), AppError> {
    let mut guard = state.lock().unwrap();
    let path = guard.current_path.clone().ok_or_else(|| AppError::Tauri("no save path".into()))?;
    let t = guard.current.as_mut().ok_or(AppError::NoTournament)?;
    let m = find_group_match(t, &match_id)?;

    m.time_limit_seconds = seconds;
    let m_clone = m.clone();

    t.updated_at = Utc::now();
    storage::save(t, &path)?;
    let t_clone = guard.current.clone().unwrap();
    drop(guard);

    events::emit_match_updated(&app, &m_clone);
    events::emit_tournament_updated(&app, &t_clone);
    Ok(())
}
