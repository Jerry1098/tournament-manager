use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::domain::model::{Tournament, MatchStatus, BYE};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamStanding {
    pub team_id: String,
    pub team_name: String,
    pub points: u32,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub cup_diff: i32,
    pub buchholz: f32,
    pub rank: u32,
}

pub fn compute_standings(t: &Tournament) -> Vec<TeamStanding> {
    struct Entry {
        wins: u32,
        draws: u32,
        losses: u32,
        cup_diff: i32,
        opponents: Vec<String>,
        name: String,
    }

    fn points(e: &Entry) -> u32 {
        e.wins * 3 + e.draws
    }

    let mut map: HashMap<String, Entry> = t
        .teams
        .iter()
        .map(|team| {
            (
                team.id.clone(),
                Entry {
                    wins: 0,
                    draws: 0,
                    losses: 0,
                    cup_diff: 0,
                    opponents: vec![],
                    name: team.name.clone(),
                },
            )
        })
        .collect();

    for round in &t.rounds {
        for m in &round.matches {
            match m.status {
                MatchStatus::Bye => {
                    if let Some(entry) = map.get_mut(&m.team_a) {
                        entry.wins += 1;
                    }
                }
                MatchStatus::Completed => {
                    // Winner = team with MORE cups (more standing = opponent sank fewer).
                    // cd > 0 means team_a has more cups → team_a wins.
                    // cd == 0 → draw.
                    let cd = m.cups_a as i32 - m.cups_b as i32;
                    if let Some(entry) = map.get_mut(&m.team_a) {
                        entry.cup_diff += cd;
                        entry.opponents.push(m.team_b.clone());
                        if cd > 0 {
                            entry.wins += 1;
                        } else if cd < 0 {
                            entry.losses += 1;
                        } else {
                            entry.draws += 1;
                        }
                    }
                    if let Some(entry) = map.get_mut(&m.team_b) {
                        entry.cup_diff -= cd;
                        entry.opponents.push(m.team_a.clone());
                        if cd < 0 {
                            entry.wins += 1;
                        } else if cd > 0 {
                            entry.losses += 1;
                        } else {
                            entry.draws += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Second pass: Buchholz = sum of opponents' wins
    let wins_snapshot: HashMap<String, u32> =
        map.iter().map(|(id, e)| (id.clone(), e.wins)).collect();

    let mut standings: Vec<TeamStanding> = map
        .into_iter()
        .filter(|(id, _)| id != BYE)
        .map(|(id, entry)| {
            let buchholz: f32 = entry
                .opponents
                .iter()
                .filter(|opp| *opp != BYE)
                .map(|opp| *wins_snapshot.get(opp).unwrap_or(&0) as f32)
                .sum();
            let pts = points(&entry);
            TeamStanding {
                team_id: id,
                team_name: entry.name,
                points: pts,
                wins: entry.wins,
                draws: entry.draws,
                losses: entry.losses,
                cup_diff: entry.cup_diff,
                buchholz,
                rank: 0,
            }
        })
        .collect();

    standings.sort_by(|a, b| {
        b.points
            .cmp(&a.points)
            .then(b.cup_diff.cmp(&a.cup_diff))
            .then(b.buchholz.partial_cmp(&a.buchholz).unwrap_or(std::cmp::Ordering::Equal))
            .then(a.team_name.cmp(&b.team_name))
    });

    for (i, s) in standings.iter_mut().enumerate() {
        s.rank = (i + 1) as u32;
    }

    standings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::*;
    use chrono::Utc;

    fn make_tournament(teams: &[(&str, &str)], matches: Vec<Match>) -> Tournament {
        Tournament {
            schema_version: 1,
            id: "t1".into(),
            name: "Test".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            config: TournamentConfig {
                format: FormatKind::Swiss,
                tables: vec![],
                swiss_rounds: 3,
                playoff_team_count: 4,
                allow_byes: true,
                random_seed: 0,
                default_match_minutes: 0,
                max_round_extension: 3,
                schedule_attempts: 10,
            },
            teams: teams
                .iter()
                .map(|(id, name)| Team { id: id.to_string(), name: name.to_string() })
                .collect(),
            phase: Phase::Group,
            rounds: vec![Round { index: 0, generated_at: Utc::now(), matches }],
            playoffs: None,
        }
    }

    fn completed(id: &str, a: &str, b: &str, ca: u32, cb: u32) -> Match {
        Match {
            id: id.into(),
            round_index: 0,
            team_a: a.into(),
            team_b: b.into(),
            cups_a: ca,
            cups_b: cb,
            status: MatchStatus::Completed,
            table_id: None,
            started_at: None,
            completed_at: None,
            time_limit_seconds: None,
            paused_at: None,
            paused_elapsed_seconds: 0,
        }
    }

    #[test]
    fn basic_wins_and_cup_diff() {
        // A wins with 10 cups remaining (B has 3), cd for A = 10-3 = +7
        // C wins with 7 cups remaining (D has 0), cd for C = 7-0 = +7
        let t = make_tournament(
            &[("A", "Alpha"), ("B", "Beta"), ("C", "Gamma"), ("D", "Delta")],
            vec![
                completed("m1", "A", "B", 10, 3),
                completed("m2", "C", "D", 7, 0),
            ],
        );
        let s = compute_standings(&t);
        let a = s.iter().find(|x| x.team_id == "A").unwrap();
        let c = s.iter().find(|x| x.team_id == "C").unwrap();
        assert_eq!(a.wins, 1);
        assert_eq!(a.cup_diff, 7);
        assert_eq!(c.wins, 1);
        assert_eq!(c.cup_diff, 7);
    }

    #[test]
    fn buchholz_sums_opponent_wins() {
        // Winner = more cups. A(10) beats B(0), C(10) beats A(0), B(10) beats D(0)
        let t = make_tournament(
            &[("A", "A"), ("B", "B"), ("C", "C"), ("D", "D")],
            vec![
                completed("m1", "A", "B", 10, 0),
                completed("m2", "C", "A", 10, 0),
                completed("m3", "B", "D", 10, 0),
            ],
        );
        let s = compute_standings(&t);
        let b_standing = s.iter().find(|x| x.team_id == "B").unwrap();
        // B's opponent is A; A has 1 win → B's Buchholz = 1
        assert_eq!(b_standing.buchholz, 1.0);
    }
}
