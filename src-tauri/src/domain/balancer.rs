use std::collections::HashMap;
use crate::domain::model::{MatchStatus, Tournament};

/// Build per-team per-category play counts from all completed/scheduled rounds.
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

/// Returns per-team per-category play counts (used by the balance report command).
pub fn category_counts(t: &Tournament) -> HashMap<String, HashMap<String, u32>> {
    build_counts(t)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{BYE, *};
    use chrono::Utc;

    fn make_tournament_with_counts(rounds_data: Vec<Vec<(&'static str, &'static str, &'static str)>>) -> Tournament {
        // rounds_data: each inner vec is a round, each tuple is (team_a, team_b, table_id)
        let all_teams: std::collections::HashSet<String> = rounds_data.iter()
            .flat_map(|r| r.iter().flat_map(|(a, b, _)| [a.to_string(), b.to_string()]))
            .filter(|s| s != BYE)
            .collect();

        let all_tables: std::collections::HashSet<String> = rounds_data.iter()
            .flat_map(|r| r.iter().map(|(_, _, t)| t.to_string()))
            .collect();

        let teams: Vec<Team> = all_teams.iter().map(|id| Team { id: id.clone(), name: id.clone() }).collect();
        let tables: Vec<TableConfig> = all_tables.iter().map(|id| TableConfig {
            id: id.clone(),
            name: id.clone(),
            category: if id.starts_with("IN") { "indoor" } else { "outdoor" }.into(),
        }).collect();

        let rounds: Vec<Round> = rounds_data.iter().enumerate()
            .map(|(idx, pairs)| Round {
                index: idx as u32,
                generated_at: Utc::now(),
                matches: pairs.iter().map(|(a, b, tid)| Match {
                    id: format!("m{idx}"),
                    round_index: idx as i32,
                    team_a: a.to_string(),
                    team_b: b.to_string(),
                    cups_a: 0,
                    cups_b: 0,
                    status: if *b == BYE { MatchStatus::Bye } else { MatchStatus::Completed },
                    table_id: if *b == BYE { None } else { Some(tid.to_string()) },
                    started_at: None,
                    completed_at: None,
                    time_limit_seconds: None,
                    paused_at: None,
                    paused_elapsed_seconds: 0,
                }).collect(),
            })
            .collect();

        Tournament {
            schema_version: 1,
            id: "t".into(),
            name: "T".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            config: TournamentConfig {
                format: FormatKind::Swiss,
                tables,
                swiss_rounds: 3,
                playoff_team_count: 4,
                allow_byes: true,
                random_seed: 0,
                default_match_minutes: 0,
                max_round_extension: 3,
                schedule_attempts: 10,
            },
            teams,
            phase: Phase::Group,
            rounds,
            playoffs: None,
        }
    }

    #[test]
    fn category_counts_basic() {
        let t = make_tournament_with_counts(vec![
            vec![("A", "B", "IN0"), ("C", "D", "OUT0")],
            vec![("A", "C", "IN0"), ("B", "D", "OUT0")],
        ]);
        let counts = category_counts(&t);

        assert_eq!(counts["A"].get("indoor").copied().unwrap_or(0), 2);
        assert_eq!(counts["A"].get("outdoor").copied().unwrap_or(0), 0);
        assert_eq!(counts["B"].get("indoor").copied().unwrap_or(0), 1);
        assert_eq!(counts["B"].get("outdoor").copied().unwrap_or(0), 1);
    }

    #[test]
    fn category_counts_ignores_byes() {
        let t = make_tournament_with_counts(vec![
            vec![("A", BYE, ""), ("B", "C", "IN0")],
        ]);
        let counts = category_counts(&t);
        assert!(!counts.get("A").map_or(false, |m| m.get("indoor").copied().unwrap_or(0) > 0));
        assert_eq!(counts["B"]["indoor"], 1);
    }
}
