use serde::Serialize;
use tauri::{AppHandle, Emitter};
use crate::domain::model::{Match, Phase, Round, Tournament};
use crate::domain::standings::TeamStanding;

pub const TOURNAMENT_LOADED: &str = "tournament-loaded";
pub const TOURNAMENT_CLOSED: &str = "tournament-closed";
pub const TOURNAMENT_UPDATED: &str = "tournament-updated";
pub const MATCH_UPDATED: &str = "match-updated";
pub const ROUND_GENERATED: &str = "round-generated";
pub const PHASE_CHANGED: &str = "phase-changed";
pub const STANDINGS_UPDATED: &str = "standings-updated";
pub const DIRTY_CHANGED: &str = "dirty-changed";

#[derive(Serialize, Clone)]
pub struct PhaseChangedPayload {
    pub from: Phase,
    pub to: Phase,
}

#[derive(Serialize, Clone)]
pub struct DirtyChangedPayload {
    pub dirty: bool,
    pub path: Option<String>,
}

pub fn emit_tournament_updated(app: &AppHandle, t: &Tournament) {
    let _ = app.emit(TOURNAMENT_UPDATED, t);
    let _ = app.emit(TOURNAMENT_LOADED, t);
}

pub fn emit_tournament_closed(app: &AppHandle) {
    let _ = app.emit(TOURNAMENT_CLOSED, ());
}

pub fn emit_match_updated(app: &AppHandle, m: &Match) {
    let _ = app.emit(MATCH_UPDATED, m);
}

pub fn emit_round_generated(app: &AppHandle, r: &Round) {
    let _ = app.emit(ROUND_GENERATED, r);
}

pub fn emit_phase_changed(app: &AppHandle, from: Phase, to: Phase) {
    let _ = app.emit(PHASE_CHANGED, PhaseChangedPayload { from, to });
}

pub fn emit_standings_updated(app: &AppHandle, standings: &[TeamStanding]) {
    let _ = app.emit(STANDINGS_UPDATED, standings);
}

pub fn emit_dirty_changed(app: &AppHandle, dirty: bool, path: Option<String>) {
    let _ = app.emit(DIRTY_CHANGED, DirtyChangedPayload { dirty, path });
}
