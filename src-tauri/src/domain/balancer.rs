use std::collections::{HashMap, HashSet};
use pathfinding::kuhn_munkres::kuhn_munkres_min;
use pathfinding::matrix::Matrix;
use crate::domain::model::{BYE, Match, MatchStatus, Tournament};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamBalance {
    pub team_id: String,
    pub team_name: String,
    pub counts: HashMap<String, u32>,
}

/// Returns map of MatchId → TableId for the non-bye matches in `new_matches`.
///
/// Uses the Hungarian algorithm (optimal 1-1 assignment) to minimise total
/// imbalance over all teams in the round at once. The cost of assigning match
/// (A, B) to a table with category C equals the sum of penalty(A, C) and
/// penalty(B, C), where the penalty reflects how much over the per-category
/// ideal that team would be after the assignment.
pub fn assign_round_to_tables(
    new_matches: &[Match],
    t: &Tournament,
) -> HashMap<String, String> {
    let playable: Vec<&Match> = new_matches.iter().filter(|m| m.team_b != BYE).collect();
    let tables = &t.config.tables;

    if tables.is_empty() || playable.is_empty() {
        return HashMap::new();
    }

    // Distinct categories declared in the tournament config (stable order).
    let all_cats: Vec<String> = {
        let mut seen = HashSet::new();
        tables.iter()
            .filter_map(|tbl| {
                if seen.insert(tbl.category.clone()) { Some(tbl.category.clone()) } else { None }
            })
            .collect()
    };

    // Per-team category counts accumulated from already-committed rounds.
    let counts = build_counts(t);

    let n_m = playable.len();
    let n_t = tables.len();
    let dim = n_m.max(n_t);

    // Build a square cost matrix (dim × dim).  Dummy cells get cost 0.
    let mut data = vec![0i64; dim * dim];
    for (i, m) in playable.iter().enumerate() {
        for (j, tbl) in tables.iter().enumerate() {
            let cost = penalty(&counts, &m.team_a, &tbl.category, &all_cats)
                     + penalty(&counts, &m.team_b, &tbl.category, &all_cats);
            data[i * dim + j] = cost;
        }
        // columns j >= n_t remain 0 (dummy)
    }
    // rows i >= n_m: already 0 (dummy)

    let matrix = Matrix::from_vec(dim, dim, data).expect("square matrix");
    let (_, assignment) = kuhn_munkres_min(&matrix);

    let mut result = HashMap::new();
    for (i, &j) in assignment.iter().enumerate() {
        if i < n_m && j < n_t {
            result.insert(playable[i].id.clone(), tables[j].id.clone());
        }
    }
    result
}

/// Imbalance penalty for assigning one more game of `category` to `team_id`.
///
/// Excess = `(current + 1) * n - (total + 1)` where n = number of categories.
/// When excess ≤ 0 the category is under-represented — cost is 0.
/// When excess > 0 the cost is **quadratic** (`excess²`) so the optimizer
/// strongly avoids creating or worsening any imbalance.
fn penalty(
    counts: &HashMap<String, HashMap<String, u32>>,
    team_id: &str,
    category: &str,
    all_cats: &[String],
) -> i64 {
    if team_id == BYE { return 0; }
    let n = all_cats.len() as i64;
    if n == 0 { return 0; }

    let cat_map = counts.get(team_id);
    let current = cat_map.and_then(|c| c.get(category)).copied().unwrap_or(0) as i64;
    let total: i64 = cat_map
        .map(|c| c.values().map(|&v| v as i64).sum())
        .unwrap_or(0);

    let excess = (current + 1) * n - (total + 1);
    if excess <= 0 { 0 } else { excess * excess }
}

/// Build per-team per-category play counts from already-committed rounds.
fn build_counts(t: &Tournament) -> HashMap<String, HashMap<String, u32>> {
    let mut counts: HashMap<String, HashMap<String, u32>> = t
        .teams
        .iter()
        .map(|tm| (tm.id.clone(), HashMap::new()))
        .collect();

    for round in &t.rounds {
        for m in &round.matches {
            if m.status == MatchStatus::Bye || m.table_id.is_none() {
                continue;
            }
            if let Some(ref tid) = m.table_id {
                let cat = t.config.tables.iter()
                    .find(|tbl| &tbl.id == tid)
                    .map(|tbl| tbl.category.as_str())
                    .unwrap_or("unknown");
                *counts.entry(m.team_a.clone()).or_default().entry(cat.to_string()).or_default() += 1;
                *counts.entry(m.team_b.clone()).or_default().entry(cat.to_string()).or_default() += 1;
            }
        }
    }

    counts
}

/// Returns per-team per-category play counts (for the balance report command).
pub fn category_counts(t: &Tournament) -> HashMap<String, HashMap<String, u32>> {
    build_counts(t)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::*;
    use chrono::Utc;

    fn make_tournament_with_tables(n_teams: usize, n_indoor: usize, n_outdoor: usize) -> Tournament {
        let teams: Vec<Team> = (0..n_teams).map(|i| Team { id: format!("T{i}"), name: format!("Team{i}") }).collect();
        let mut tables: Vec<TableConfig> = Vec::new();
        for i in 0..n_indoor  { tables.push(TableConfig { id: format!("IN{i}"),  name: format!("Indoor{i}"),  category: "indoor".into() }); }
        for i in 0..n_outdoor { tables.push(TableConfig { id: format!("OUT{i}"), name: format!("Outdoor{i}"), category: "outdoor".into() }); }
        Tournament {
            schema_version: 1, id: "t".into(), name: "T".into(),
            created_at: Utc::now(), updated_at: Utc::now(),
            config: TournamentConfig {
                format: FormatKind::Swiss, tables, swiss_rounds: 4,
                playoff_team_count: 4, allow_byes: true, random_seed: 0, default_match_minutes: 0,
            },
            teams, phase: Phase::Group, rounds: vec![], playoffs: None,
        }
    }

    fn scheduled_match(id: &str, a: &str, b: &str) -> Match {
        Match {
            id: id.into(), round_index: 0, team_a: a.into(), team_b: b.into(),
            cups_a: 0, cups_b: 0, status: MatchStatus::Scheduled, table_id: None,
            started_at: None, completed_at: None, time_limit_seconds: None,
            paused_at: None, paused_elapsed_seconds: 0,
        }
    }

    #[test]
    fn assigns_all_non_bye_matches() {
        let t = make_tournament_with_tables(8, 2, 2);
        let matches = vec![
            scheduled_match("m1", "T0", "T1"), scheduled_match("m2", "T2", "T3"),
            scheduled_match("m3", "T4", "T5"), scheduled_match("m4", "T6", "T7"),
        ];
        let result = assign_round_to_tables(&matches, &t);
        assert_eq!(result.len(), 4);
        for table_id in result.values() {
            assert!(t.config.tables.iter().any(|tbl| &tbl.id == table_id));
        }
    }

    #[test]
    fn no_two_matches_share_a_table() {
        let t = make_tournament_with_tables(8, 2, 2);
        let matches = vec![
            scheduled_match("m1", "T0", "T1"), scheduled_match("m2", "T2", "T3"),
            scheduled_match("m3", "T4", "T5"), scheduled_match("m4", "T6", "T7"),
        ];
        let result = assign_round_to_tables(&matches, &t);
        let mut used: HashSet<&str> = HashSet::new();
        for table_id in result.values() {
            assert!(used.insert(table_id.as_str()), "table {table_id} used twice");
        }
    }
}
