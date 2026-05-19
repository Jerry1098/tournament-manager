use crate::domain::model::{BracketNode, Match, MatchStatus, Playoffs, Tournament};
use crate::domain::standings::compute_standings;
use crate::error::AppError;

fn new_id() -> String {
    ulid::Ulid::new().to_string()
}

fn placeholder_match() -> Match {
    Match {
        id: new_id(),
        round_index: -1,
        team_a: String::new(),
        team_b: String::new(),
        cups_a: 0,
        cups_b: 0,
        status: MatchStatus::Scheduled,
        table_id: None,
        started_at: None,
        completed_at: None,
        time_limit_seconds: None,
        paused_at: None,
        paused_elapsed_seconds: 0,
    }
}

/// Depth of node i in a 0-indexed binary heap = floor(log2(i+1)).
fn heap_level(i: usize) -> u32 {
    (usize::BITS - (i + 1).leading_zeros()) - 1
}

/// Generate first-round pairs in heap leaf order so that higher seeds only
/// meet lower seeds in later rounds. For n=4: [(0,3),(1,2)]. For n=8:
/// [(0,7),(1,6),(2,5),(3,4)] placed at leaves in order.
fn first_round_pairs(n: usize) -> Vec<(usize, usize)> {
    (0..n / 2).map(|i| (i, n - 1 - i)).collect()
}

pub fn build_playoffs(t: &mut Tournament) -> Result<Playoffs, AppError> {
    let n = t.config.playoff_team_count as usize;
    if n < 2 || !n.is_power_of_two() {
        return Err(AppError::InvalidArgument(format!(
            "playoffTeamCount must be a power of 2 ≥ 2, got {n}"
        )));
    }

    let standings = compute_standings(t);
    if standings.len() < n {
        return Err(AppError::InvalidArgument(format!(
            "Not enough teams ({}) for {n} playoff spots",
            standings.len()
        )));
    }

    let seeding: Vec<String> = standings.into_iter().take(n).map(|s| s.team_id).collect();

    // Pre-allocate n-1 matches in heap order:
    //   index 0 = final (round 0)
    //   index 1,2 = semis (round 1)
    //   index 3..6 = quarters (round 2), etc.
    let total = n - 1;
    let mut matches: Vec<Match> = (0..total).map(|_| placeholder_match()).collect();

    // Leaf indices in the heap: [n/2-1 .. n-2]
    let first_leaf = n / 2 - 1;
    let pairs = first_round_pairs(n);
    for (i, (a_idx, b_idx)) in pairs.iter().enumerate() {
        matches[first_leaf + i].team_a = seeding[*a_idx].clone();
        matches[first_leaf + i].team_b = seeding[*b_idx].clone();
    }

    // Build bracket nodes: heap order, round = depth from root (0 = final)
    let bracket: Vec<BracketNode> = (0..total)
        .map(|i| {
            let level = heap_level(i);
            let pos = i + 1 - (1_usize << level);
            BracketNode {
                match_id: matches[i].id.clone(),
                round: level,
                position: pos as u32,
            }
        })
        .collect();

    Ok(Playoffs {
        seeding,
        matches,
        bracket,
        third_place_match: None,
    })
}

/// After a playoff match completes, propagate the winner to the parent slot
/// and generate the 3rd-place match when both semis are done.
pub fn advance_bracket(t: &mut Tournament, match_id: &str) -> Result<(), AppError> {
    let playoffs = t
        .playoffs
        .as_mut()
        .ok_or_else(|| AppError::NotFound("playoffs".into()))?;

    // Find heap index of this match
    let match_pos = playoffs
        .matches
        .iter()
        .position(|m| m.id == match_id)
        .ok_or_else(|| AppError::NotFound(format!("match {match_id} in bracket")))?;

    let winner = playoffs.matches[match_pos]
        .winner()
        .ok_or_else(|| AppError::InvalidArgument("tied cup counts not allowed".into()))?
        .to_string();

    // Propagate winner to parent match (heap parent = (i-1)/2)
    if match_pos > 0 {
        let parent = (match_pos - 1) / 2;
        let is_left = match_pos % 2 == 1; // odd index = left child
        if is_left {
            playoffs.matches[parent].team_a = winner.clone();
        } else {
            playoffs.matches[parent].team_b = winner.clone();
        }
    }

    // Generate 3rd-place match when both semis (heap indices 1 & 2) are complete.
    // For n=2 there are no semis; skip.
    let n = t.config.playoff_team_count as usize;
    if n >= 4 {
        let playoffs = t.playoffs.as_mut().unwrap();
        let semi_l_done = playoffs
            .matches
            .get(1)
            .map(|m| m.status == MatchStatus::Completed)
            .unwrap_or(false);
        let semi_r_done = playoffs
            .matches
            .get(2)
            .map(|m| m.status == MatchStatus::Completed)
            .unwrap_or(false);

        if semi_l_done && semi_r_done && playoffs.third_place_match.is_none() {
            let loser = |m: &Match| -> String {
                let w = m.winner().unwrap_or("");
                if m.team_a == w { m.team_b.clone() } else { m.team_a.clone() }
            };
            let loser1 = loser(&playoffs.matches[1]);
            let loser2 = loser(&playoffs.matches[2]);
            let mut tpm = placeholder_match();
            tpm.team_a = loser1;
            tpm.team_b = loser2;
            playoffs.third_place_match = Some(tpm);
        }
    }

    Ok(())
}
