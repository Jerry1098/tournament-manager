use std::collections::{HashMap, HashSet};
use chrono::Utc;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

use crate::domain::model::{BYE, Match, MatchStatus, Phase, Round, TableConfig, Tournament};

fn new_id() -> String {
    ulid::Ulid::new().to_string()
}

fn make_match(round_index: u32, a: &str, b: &str, table_id: Option<String>) -> Match {
    Match {
        id: new_id(),
        round_index: round_index as i32,
        team_a: a.to_string(),
        team_b: b.to_string(),
        cups_a: 0,
        cups_b: 0,
        status: if b == BYE { MatchStatus::Bye } else { MatchStatus::Scheduled },
        table_id,
        started_at: None,
        completed_at: None,
        time_limit_seconds: None,
        paused_at: None,
        paused_elapsed_seconds: 0,
    }
}

fn pair_key(a: &str, b: &str) -> (String, String) {
    if a <= b { (a.to_string(), b.to_string()) } else { (b.to_string(), a.to_string()) }
}

/// Per-team target games per category for a team that plays `total_games` non-bye matches.
/// Uses the largest-remainder method so that `sum(targets) == total_games`.
pub fn calculate_category_targets(total_games: u32, tables: &[TableConfig]) -> HashMap<String, u32> {
    if tables.is_empty() { return HashMap::new(); }

    let mut cat_counts: HashMap<String, u32> = HashMap::new();
    for t in tables {
        *cat_counts.entry(t.category.clone()).or_default() += 1;
    }

    let total_tables = tables.len() as f64;

    // Exact proportional quota per category (sorted for determinism)
    let mut entries: Vec<(String, f64)> = cat_counts.iter()
        .map(|(cat, &cnt)| (cat.clone(), total_games as f64 * cnt as f64 / total_tables))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Floor allocations
    let mut result: HashMap<String, u32> = entries.iter()
        .map(|(cat, val)| (cat.clone(), val.floor() as u32))
        .collect();

    let sum_floors: u32 = result.values().sum();
    let to_distribute = total_games.saturating_sub(sum_floors);

    // Distribute remainder by largest fractional parts (deterministic tie-break by name)
    let mut by_remainder: Vec<(String, f64)> = entries.iter()
        .map(|(cat, val)| (cat.clone(), val - val.floor()))
        .collect();
    by_remainder.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });

    for i in 0..to_distribute as usize {
        if i < by_remainder.len() {
            *result.entry(by_remainder[i].0.clone()).or_default() += 1;
        }
    }

    result
}

/// Generate one pairing pass: each team plays once (or gets a bye for odd counts).
fn generate_pass(
    team_ids: &[String],
    history: &HashSet<(String, String)>,
    bye_counts: &HashMap<String, usize>,
    rng: &mut ChaCha8Rng,
) -> Vec<(String, String)> {
    let mut pool: Vec<String> = team_ids.to_vec();
    pool.shuffle(rng);

    let bye_team: Option<String> = if pool.len() % 2 == 1 {
        let candidate = pool.iter()
            .min_by_key(|id| bye_counts.get(*id).copied().unwrap_or(0))
            .cloned();
        if let Some(ref id) = candidate { pool.retain(|x| x != id); }
        candidate
    } else {
        None
    };

    let mut paired: HashSet<String> = HashSet::new();
    let mut pairs: Vec<(String, String)> = Vec::new();

    for i in 0..pool.len() {
        if paired.contains(&pool[i]) { continue; }
        let a = &pool[i];

        let partner = pool[i + 1..]
            .iter()
            .filter(|b| !paired.contains(*b))
            .find(|b| !history.contains(&pair_key(a, b)))
            .or_else(|| pool[i + 1..].iter().find(|b| !paired.contains(*b)))
            .cloned();

        if let Some(b) = partner {
            paired.insert(a.clone());
            paired.insert(b.clone());
            pairs.push((a.clone(), b.clone()));
        }
    }

    if let Some(bye_id) = bye_team {
        pairs.push((bye_id, BYE.to_string()));
    }

    pairs
}

/// Try to assign a category to every non-bye match such that each team's
/// per-category count exactly matches their proportional target.
///
/// Algorithm: most-constrained-first greedy.
///   1. Find the unassigned match with the fewest valid categories.
///   2. Among valid categories for that match, pick the one with the smallest
///      global remaining quota (most globally constrained).
///   3. Assign, decrement quotas, repeat.
///
/// Returns None if no valid complete assignment exists.
fn try_assign_categories(
    all_pairs: &[(String, String)],
    tables: &[TableConfig],
    rng: &mut ChaCha8Rng,
) -> Option<Vec<Option<String>>> {
    // Ordered unique categories
    let cats: Vec<String> = {
        let mut seen = HashSet::new();
        tables.iter()
            .filter_map(|t| if seen.insert(t.category.clone()) { Some(t.category.clone()) } else { None })
            .collect()
    };

    if cats.is_empty() {
        return Some(vec![None; all_pairs.len()]);
    }

    // Actual non-bye game count per team
    let mut game_counts: HashMap<String, u32> = HashMap::new();
    for (a, b) in all_pairs {
        if b != BYE {
            *game_counts.entry(a.clone()).or_default() += 1;
            *game_counts.entry(b.clone()).or_default() += 1;
        }
    }

    // Per-team remaining quota (based on actual game count)
    let mut remaining: HashMap<String, HashMap<String, i32>> = game_counts.iter()
        .map(|(team, &count)| {
            let targets = calculate_category_targets(count, tables);
            (team.clone(), targets.into_iter().map(|(c, v)| (c, v as i32)).collect())
        })
        .collect();

    let mut assignments: Vec<Option<String>> = vec![None; all_pairs.len()];

    // Collect non-bye match indices and shuffle for randomized tie-breaking
    let mut unassigned: Vec<usize> = all_pairs.iter().enumerate()
        .filter(|(_, (_, b))| b != BYE)
        .map(|(i, _)| i)
        .collect();
    unassigned.shuffle(rng);

    while !unassigned.is_empty() {
        // Most-constrained match: fewest valid categories
        let pos = unassigned.iter().enumerate()
            .min_by_key(|(_, &idx)| {
                let (a, b) = &all_pairs[idx];
                cats.iter()
                    .filter(|cat| {
                        remaining.get(a).and_then(|m| m.get(*cat)).copied().unwrap_or(0) > 0
                        && remaining.get(b).and_then(|m| m.get(*cat)).copied().unwrap_or(0) > 0
                    })
                    .count()
            })
            .map(|(pos, _)| pos)?;

        let match_idx = unassigned.remove(pos);
        let (a, b) = &all_pairs[match_idx];

        let valid: Vec<String> = cats.iter()
            .filter(|cat| {
                remaining.get(a).and_then(|m| m.get(*cat)).copied().unwrap_or(0) > 0
                && remaining.get(b).and_then(|m| m.get(*cat)).copied().unwrap_or(0) > 0
            })
            .cloned()
            .collect();

        if valid.is_empty() { return None; }

        // Among valid choices, prefer the globally most-constrained category
        // (smallest total remaining across all teams)
        let chosen = valid.iter()
            .min_by_key(|cat| {
                remaining.values()
                    .map(|m| m.get(*cat).copied().unwrap_or(0))
                    .sum::<i32>()
            })
            .cloned()
            .unwrap();

        *remaining.get_mut(a).unwrap().get_mut(&chosen).unwrap() -= 1;
        *remaining.get_mut(b).unwrap().get_mut(&chosen).unwrap() -= 1;
        assignments[match_idx] = Some(chosen);
    }

    // All quotas must be fully consumed
    for cat_map in remaining.values() {
        if cat_map.values().any(|&v| v != 0) { return None; }
    }

    Some(assignments)
}

/// Batch pairs (with pre-assigned categories) into scheduling rounds.
/// Each round uses at most one instance of each physical table.
/// No team appears twice in a round.
fn batch_into_rounds(
    all_pairs: &[(String, String)],
    assignments: &[Option<String>],
    tables: &[TableConfig],
) -> Vec<Round> {
    let n_tables = tables.len().max(1);

    let mut pending: Vec<(String, String, Option<String>)> = all_pairs.iter().zip(assignments)
        .map(|((a, b), cat)| (a.clone(), b.clone(), cat.clone()))
        .collect();

    let mut rounds: Vec<Round> = Vec::new();
    let mut round_index = 0u32;

    while !pending.is_empty() {
        let mut slot: Vec<(String, String, Option<String>)> = Vec::new();
        let mut used_teams: HashSet<String> = HashSet::new();
        let mut used_tables: HashSet<String> = HashSet::new();
        let mut deferred: Vec<(String, String, Option<String>)> = Vec::new();

        for (a, b, cat) in pending {
            if b == BYE {
                if !used_teams.contains(&a) {
                    used_teams.insert(a.clone());
                    slot.push((a, b, None));
                } else {
                    deferred.push((a, b, cat));
                }
                continue;
            }

            if used_teams.contains(&a) || used_teams.contains(&b) || slot.len() >= n_tables {
                deferred.push((a, b, cat));
                continue;
            }

            let table = match &cat {
                Some(c) => tables.iter().find(|tbl| &tbl.category == c && !used_tables.contains(&tbl.id)),
                None    => tables.iter().find(|tbl| !used_tables.contains(&tbl.id)),
            };

            if let Some(tbl) = table {
                used_teams.insert(a.clone());
                used_teams.insert(b.clone());
                used_tables.insert(tbl.id.clone());
                slot.push((a, b, Some(tbl.id.clone())));
            } else {
                deferred.push((a, b, cat));
            }
        }

        // Safety: force progress if nothing was placed (shouldn't happen with valid input)
        if slot.is_empty() {
            if let Some((a, b, cat)) = deferred.first().cloned() {
                deferred.remove(0);
                let tid = cat.as_deref()
                    .and_then(|c| tables.iter().find(|tbl| tbl.category == c))
                    .map(|tbl| tbl.id.clone());
                slot.push((a, b, tid));
            } else {
                break;
            }
        }

        pending = deferred;

        let matches: Vec<Match> = slot.into_iter().map(|(a, b, tid)| {
            make_match(round_index, &a, &b, if b == BYE { None } else { tid })
        }).collect();

        rounds.push(Round {
            index: round_index,
            generated_at: Utc::now(),
            matches,
        });

        round_index += 1;
    }

    rounds
}

/// One attempt to generate a complete balanced schedule for `total_games` matches per team.
fn try_generate_schedule(
    t: &Tournament,
    total_games: u32,
    rng: &mut ChaCha8Rng,
) -> Option<Vec<Round>> {
    let team_ids: Vec<String> = t.teams.iter().map(|tm| tm.id.clone()).collect();

    let mut all_pairs: Vec<(String, String)> = Vec::new();
    let mut history: HashSet<(String, String)> = HashSet::new();
    let mut bye_counts: HashMap<String, usize> = HashMap::new();

    for _ in 0..total_games {
        let pass = generate_pass(&team_ids, &history, &bye_counts, rng);
        for (a, b) in &pass {
            history.insert(pair_key(a, b));
            if b == BYE { *bye_counts.entry(a.clone()).or_insert(0) += 1; }
        }
        all_pairs.extend(pass);
    }

    let assignments = if t.config.tables.is_empty() {
        vec![None; all_pairs.len()]
    } else {
        try_assign_categories(&all_pairs, &t.config.tables, rng)?
    };

    Some(batch_into_rounds(&all_pairs, &assignments, &t.config.tables))
}

/// Generate the complete group-phase schedule with exact per-category balance.
///
/// Strategy:
///   For each `total_games` in `[swiss_rounds, swiss_rounds + max_round_extension]`:
///     Try 10 attempts with varied seeds. Return the first success.
///   If all budgets exhausted, fall back to a schedule without category balance.
pub fn generate_full_schedule(t: &mut Tournament) {
    let base = t.config.swiss_rounds;
    let max_extra = t.config.max_round_extension;

    for extra in 0..=max_extra {
        let total_games = base + extra;
        for attempt in 0u64..10 {
            let seed = t.config.random_seed
                .wrapping_add(attempt.wrapping_mul(0x9e37_79b9_7f4a_7c15))
                .wrapping_add((extra as u64).wrapping_mul(0x6c62_272e_07bb_0142));
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            if let Some(rounds) = try_generate_schedule(t, total_games, &mut rng) {
                t.rounds = rounds;
                t.phase = Phase::Group;
                return;
            }
        }
    }

    // Fallback: generate without category balance (should rarely be reached)
    let mut rng = ChaCha8Rng::seed_from_u64(t.config.random_seed);
    let rounds = try_generate_schedule(t, base, &mut rng)
        .unwrap_or_else(|| generate_fallback(t, base, &mut rng));
    t.rounds = rounds;
    t.phase = Phase::Group;
}

/// Last-resort fallback: generate schedule ignoring category balance.
fn generate_fallback(t: &Tournament, total_games: u32, rng: &mut ChaCha8Rng) -> Vec<Round> {
    let team_ids: Vec<String> = t.teams.iter().map(|tm| tm.id.clone()).collect();

    let mut all_pairs: Vec<(String, String)> = Vec::new();
    let mut history: HashSet<(String, String)> = HashSet::new();
    let mut bye_counts: HashMap<String, usize> = HashMap::new();

    for _ in 0..total_games {
        let pass = generate_pass(&team_ids, &history, &bye_counts, rng);
        for (a, b) in &pass {
            history.insert(pair_key(a, b));
            if b == BYE { *bye_counts.entry(a.clone()).or_insert(0) += 1; }
        }
        all_pairs.extend(pass);
    }

    let assignments = vec![None; all_pairs.len()];
    batch_into_rounds(&all_pairs, &assignments, &t.config.tables)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::*;
    use chrono::Utc;

    fn make_tournament(n: usize, seed: u64, n_tables: usize, matches_per_team: u32) -> Tournament {
        let teams: Vec<Team> = (0..n)
            .map(|i| Team { id: format!("T{i}"), name: format!("Team{i}") })
            .collect();
        let tables: Vec<TableConfig> = (0..n_tables)
            .map(|i| TableConfig {
                id: format!("tbl{i}"),
                name: format!("Table {i}"),
                category: if i < n_tables / 2 { "indoor" } else { "outdoor" }.into(),
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
                swiss_rounds: matches_per_team,
                playoff_team_count: 4,
                allow_byes: true,
                random_seed: seed,
                default_match_minutes: 0,
                max_round_extension: 3,
            },
            teams,
            phase: Phase::Setup,
            rounds: vec![],
            playoffs: None,
        }
    }

    #[test]
    fn schedule_is_deterministic() {
        let mut t1 = make_tournament(8, 42, 2, 4);
        let mut t2 = make_tournament(8, 42, 2, 4);
        generate_full_schedule(&mut t1);
        generate_full_schedule(&mut t2);
        assert_eq!(t1.rounds.len(), t2.rounds.len());
        let pairs1: Vec<_> = t1.rounds.iter().flat_map(|r| r.matches.iter().map(|m| (m.team_a.clone(), m.team_b.clone()))).collect();
        let pairs2: Vec<_> = t2.rounds.iter().flat_map(|r| r.matches.iter().map(|m| (m.team_a.clone(), m.team_b.clone()))).collect();
        assert_eq!(pairs1, pairs2);
    }

    #[test]
    fn no_team_appears_twice_in_same_round() {
        let mut t = make_tournament(28, 42, 6, 3);
        generate_full_schedule(&mut t);
        for round in &t.rounds {
            let mut seen = HashSet::new();
            for m in &round.matches {
                assert!(seen.insert(m.team_a.clone()), "duplicate {} in round {}", m.team_a, round.index);
                if m.team_b != BYE {
                    assert!(seen.insert(m.team_b.clone()), "duplicate {} in round {}", m.team_b, round.index);
                }
            }
        }
    }

    #[test]
    fn each_round_has_at_most_n_tables_matches() {
        let n_tables = 6;
        let mut t = make_tournament(28, 0, n_tables, 3);
        generate_full_schedule(&mut t);
        for round in &t.rounds {
            let playable = round.matches.iter().filter(|m| m.team_b != BYE).count();
            assert!(playable <= n_tables, "round {} has {} matches > {} tables", round.index, playable, n_tables);
        }
    }

    #[test]
    fn each_team_plays_at_least_configured_games() {
        let matches_per_team = 3u32;
        let max_ext = 3u32;
        let mut t = make_tournament(28, 1, 6, matches_per_team);
        t.config.max_round_extension = max_ext;
        generate_full_schedule(&mut t);

        for team in &t.teams.clone() {
            let played = t.rounds.iter()
                .flat_map(|r| r.matches.iter())
                .filter(|m| m.team_b != BYE && (m.team_a == team.id || m.team_b == team.id))
                .count() as u32;
            assert!(
                played >= matches_per_team,
                "Team {} played {played} < {matches_per_team}", team.id
            );
            assert!(
                played <= matches_per_team + max_ext,
                "Team {} played {played} > {} (configured + extension)", team.id, matches_per_team + max_ext
            );
        }
    }

    #[test]
    fn odd_team_count_produces_byes() {
        let mut t = make_tournament(7, 0, 2, 4);
        generate_full_schedule(&mut t);
        let bye_count = t.rounds.iter()
            .flat_map(|r| r.matches.iter())
            .filter(|m| m.status == MatchStatus::Bye)
            .count();
        // 7 teams with 4 passes → 4 byes
        assert_eq!(bye_count, 4);
    }

    #[test]
    fn phase_changes_to_group() {
        let mut t = make_tournament(4, 1, 2, 2);
        assert_eq!(t.phase, Phase::Setup);
        generate_full_schedule(&mut t);
        assert_eq!(t.phase, Phase::Group);
    }

    /// Every team should play the same number of games in each category.
    ///
    /// The solver may extend swiss_rounds by up to max_round_extension to achieve
    /// an integer target. For cases where extension makes targets exact integers,
    /// we require max - min == 0 (perfect balance).
    #[test]
    fn category_balance_exact_for_integer_targets() {
        // Cases where an even mpt (or extension) yields integer per-cat targets
        // Tables: n_tables/2 indoor + n_tables/2 outdoor
        let cases: &[(usize, usize, u32)] = &[
            (16, 4, 4),  // target = 2 indoor, 2 outdoor (exact with mpt=4)
            (10, 6, 4),  // target = 2, 2 (exact)
            (8,  4, 4),  // target = 2, 2 (exact)
            (28, 6, 4),  // target = 2, 2 (exact)
        ];

        for &(n_teams, n_tables, mpt) in cases {
            let mut t = make_tournament(n_teams, 77, n_tables, mpt);
            t.config.max_round_extension = 0; // no extension needed when target already exact
            generate_full_schedule(&mut t);

            for team in &t.teams.clone() {
                let mut counts: HashMap<String, u32> = HashMap::new();
                for round in &t.rounds {
                    for m in &round.matches {
                        let is_participant = m.team_a == team.id || m.team_b == team.id;
                        if !is_participant || m.team_b == BYE { continue; }
                        if let Some(ref tid) = m.table_id {
                            let cat = t.config.tables.iter()
                                .find(|tbl| &tbl.id == tid)
                                .map(|tbl| tbl.category.as_str())
                                .unwrap_or("unknown");
                            *counts.entry(cat.to_string()).or_insert(0) += 1;
                        }
                    }
                }

                if counts.len() > 1 {
                    let min_val = *counts.values().min().unwrap();
                    let max_val = *counts.values().max().unwrap();
                    assert_eq!(
                        max_val, min_val,
                        "Team {} (n_teams={n_teams}, n_tables={n_tables}, mpt={mpt}): \
                         category imbalance {counts:?}",
                        team.id
                    );
                }
            }
        }
    }

    /// For odd mpt with equal categories the solver extends by 1 round to reach
    /// an even (integer) target. Every team's cat counts should then be exact.
    #[test]
    fn category_balance_after_extension() {
        let cases: &[(usize, usize, u32)] = &[
            (28, 6, 3),  // 3 games, equal cats → extend to 4
            (8,  2, 3),  // 3 games, equal cats → extend to 4
        ];

        for &(n_teams, n_tables, mpt) in cases {
            let mut t = make_tournament(n_teams, 77, n_tables, mpt);
            t.config.max_round_extension = 3;
            generate_full_schedule(&mut t);

            // After extension, every non-bye team should have equal cat counts
            for team in &t.teams.clone() {
                let mut counts: HashMap<String, u32> = HashMap::new();
                for round in &t.rounds {
                    for m in &round.matches {
                        let is_participant = m.team_a == team.id || m.team_b == team.id;
                        if !is_participant || m.team_b == BYE { continue; }
                        if let Some(ref tid) = m.table_id {
                            let cat = t.config.tables.iter()
                                .find(|tbl| &tbl.id == tid)
                                .map(|tbl| tbl.category.as_str())
                                .unwrap_or("unknown");
                            *counts.entry(cat.to_string()).or_insert(0) += 1;
                        }
                    }
                }

                if counts.len() > 1 {
                    let total: u32 = counts.values().sum();
                    // Only teams that actually played an even number of games can be exactly balanced
                    if total % 2 == 0 {
                        let min_val = *counts.values().min().unwrap();
                        let max_val = *counts.values().max().unwrap();
                        assert_eq!(
                            max_val, min_val,
                            "Team {} (n_teams={n_teams}, n_tables={n_tables}, mpt={mpt}, total={total}): \
                             category imbalance {counts:?}",
                            team.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn single_category_always_balanced() {
        // All tables in one category → every match in that category, trivially balanced
        let n = 10usize;
        let mut t = Tournament {
            schema_version: 1,
            id: "t".into(),
            name: "T".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            config: TournamentConfig {
                format: FormatKind::Swiss,
                tables: (0..4).map(|i| TableConfig { id: format!("t{i}"), name: format!("T{i}"), category: "indoor".into() }).collect(),
                swiss_rounds: 3,
                playoff_team_count: 4,
                allow_byes: true,
                random_seed: 5,
                default_match_minutes: 0,
                max_round_extension: 3,
            },
            teams: (0..n).map(|i| Team { id: format!("T{i}"), name: format!("Team{i}") }).collect(),
            phase: Phase::Setup,
            rounds: vec![],
            playoffs: None,
        };
        generate_full_schedule(&mut t);
        for round in &t.rounds {
            for m in &round.matches {
                if m.team_b != BYE {
                    assert!(m.table_id.is_some(), "match without table");
                    let cat = t.config.tables.iter().find(|tbl| Some(&tbl.id) == m.table_id.as_ref()).map(|tbl| tbl.category.as_str()).unwrap_or("");
                    assert_eq!(cat, "indoor");
                }
            }
        }
    }

    #[test]
    fn calculate_category_targets_sums_to_total() {
        let tables: Vec<TableConfig> = vec![
            TableConfig { id: "1".into(), name: "A".into(), category: "indoor".into() },
            TableConfig { id: "2".into(), name: "B".into(), category: "indoor".into() },
            TableConfig { id: "3".into(), name: "C".into(), category: "outdoor".into() },
        ];
        for total in 0u32..10 {
            let targets = calculate_category_targets(total, &tables);
            let sum: u32 = targets.values().sum();
            assert_eq!(sum, total, "targets sum {sum} != {total} for {tables:?}");
        }
    }
}
