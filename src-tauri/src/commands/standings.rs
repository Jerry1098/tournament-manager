use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use crate::domain::balancer::category_counts;
use crate::domain::standings::{compute_standings, TeamStanding};
use crate::error::AppError;
use crate::state::SharedState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamBalance {
    pub team_id: String,
    pub team_name: String,
    pub counts: HashMap<String, u32>,
}

#[tauri::command]
pub async fn get_standings(state: State<'_, SharedState>) -> Result<Vec<TeamStanding>, AppError> {
    let guard = state.lock().unwrap();
    let t = guard.current.as_ref().ok_or(AppError::NoTournament)?;
    Ok(compute_standings(t))
}

#[tauri::command]
pub async fn get_balance_report(state: State<'_, SharedState>) -> Result<Vec<TeamBalance>, AppError> {
    let guard = state.lock().unwrap();
    let t = guard.current.as_ref().ok_or(AppError::NoTournament)?;
    let counts = category_counts(t);
    let report: Vec<TeamBalance> = t
        .teams
        .iter()
        .map(|team| TeamBalance {
            team_id: team.id.clone(),
            team_name: team.name.clone(),
            counts: counts.get(&team.id).cloned().unwrap_or_default(),
        })
        .collect();
    Ok(report)
}
