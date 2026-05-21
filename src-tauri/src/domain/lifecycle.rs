use crate::domain::model::{Match, MatchStatus, Phase, Tournament};
use crate::error::AppError;

/// Returns the currently active round's matches, if in group phase.
pub fn current_round_matches(t: &Tournament) -> Option<&[Match]> {
    t.rounds.last().map(|r| r.matches.as_slice())
}

/// Returns true when every match in the most recent round is terminal.
pub fn round_is_complete(t: &Tournament) -> bool {
    match current_round_matches(t) {
        Some(matches) => matches.iter().all(|m| m.is_terminal()),
        None => true,
    }
}

/// Returns how many matches in the current round are InProgress.
pub fn in_progress_count(t: &Tournament) -> usize {
    current_round_matches(t)
        .map(|ms| ms.iter().filter(|m| m.status == MatchStatus::InProgress).count())
        .unwrap_or(0)
}

pub fn require_phase(t: &Tournament, phase: &Phase, op: &str) -> Result<(), AppError> {
    if &t.phase != phase {
        Err(AppError::WrongPhase(format!(
            "{op} requires phase {phase:?}, current is {:?}",
            t.phase
        )))
    } else {
        Ok(())
    }
}

pub fn require_any_phase(t: &Tournament, phases: &[Phase], op: &str) -> Result<(), AppError> {
    if !phases.contains(&t.phase) {
        Err(AppError::WrongPhase(format!(
            "{op} not allowed in phase {:?}",
            t.phase
        )))
    } else {
        Ok(())
    }
}

/// Find a match by ID across all group rounds, returning mutable ref.
pub fn find_group_match<'a>(t: &'a mut Tournament, match_id: &str) -> Result<&'a mut Match, AppError> {
    for round in &mut t.rounds {
        if let Some(m) = round.matches.iter_mut().find(|m| m.id == match_id) {
            return Ok(m);
        }
    }
    Err(AppError::NotFound(format!("match {match_id}")))
}

/// Find a match by ID in either group rounds or playoffs.
pub fn find_any_match<'a>(t: &'a mut Tournament, match_id: &str) -> Result<&'a mut Match, AppError> {
    for round in &mut t.rounds {
        if let Some(m) = round.matches.iter_mut().find(|m| m.id == match_id) {
            return Ok(m);
        }
    }
    if let Some(playoffs) = t.playoffs.as_mut() {
        if let Some(m) = playoffs.third_place_match.as_mut().filter(|m| m.id == match_id) {
            return Ok(m);
        }
        if let Some(m) = playoffs.matches.iter_mut().find(|m| m.id == match_id) {
            return Ok(m);
        }
    }
    Err(AppError::NotFound(format!("match {match_id}")))
}

/// Find a match by ID in playoffs matches.
pub fn find_playoff_match<'a>(t: &'a mut Tournament, match_id: &str) -> Result<&'a mut Match, AppError> {
    let playoffs = t
        .playoffs
        .as_mut()
        .ok_or_else(|| AppError::NotFound("playoffs".into()))?;

    if let Some(m) = playoffs.third_place_match.as_mut().filter(|m| m.id == match_id) {
        return Ok(m);
    }
    playoffs
        .matches
        .iter_mut()
        .find(|m| m.id == match_id)
        .ok_or_else(|| AppError::NotFound(format!("playoff match {match_id}")))
}
