use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const BYE: &str = "BYE";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tournament {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config: TournamentConfig,
    pub teams: Vec<Team>,
    pub phase: Phase,
    pub rounds: Vec<Round>,
    pub playoffs: Option<Playoffs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TournamentConfig {
    pub format: FormatKind,
    pub tables: Vec<TableConfig>,
    pub swiss_rounds: u32,
    pub playoff_team_count: u32,
    pub allow_byes: bool,
    pub random_seed: u64,
    /// Default match duration in minutes. 0 = no timer.
    #[serde(default)]
    pub default_match_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableConfig {
    pub id: String,
    pub name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatKind {
    Swiss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Round {
    pub index: u32,
    pub generated_at: DateTime<Utc>,
    pub matches: Vec<Match>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Match {
    pub id: String,
    pub round_index: i32,
    pub team_a: String,
    pub team_b: String,
    /// Cups remaining in each team's rack at game end.
    /// The team with FEWER cups wins (they cleared the opponent's rack).
    /// Winner has 0 (sank all opponent cups), loser still has cups standing.
    pub cups_a: u32,
    pub cups_b: u32,
    pub status: MatchStatus,
    pub table_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    /// Per-match time limit override in seconds. None = use tournament default.
    #[serde(default)]
    pub time_limit_seconds: Option<u32>,
    /// When the timer was paused (Some = currently paused).
    #[serde(default)]
    pub paused_at: Option<DateTime<Utc>>,
    /// Accumulated pause duration in seconds.
    #[serde(default)]
    pub paused_elapsed_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MatchStatus {
    Scheduled,
    InProgress,
    Completed,
    Bye,
}

impl Match {
    pub fn is_terminal(&self) -> bool {
        matches!(self.status, MatchStatus::Completed | MatchStatus::Bye)
    }

    /// Team with FEWER cups wins (they cleared the opponent's rack, reaching 0).
    pub fn winner(&self) -> Option<&str> {
        if self.status != MatchStatus::Completed {
            return None;
        }
        if self.cups_a < self.cups_b {
            Some(&self.team_a)
        } else if self.cups_b < self.cups_a {
            Some(&self.team_b)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Setup,
    Group,
    Playoffs,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playoffs {
    pub seeding: Vec<String>,
    pub matches: Vec<Match>,
    pub bracket: Vec<BracketNode>,
    pub third_place_match: Option<Match>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BracketNode {
    pub match_id: String,
    pub round: u32,
    pub position: u32,
}
