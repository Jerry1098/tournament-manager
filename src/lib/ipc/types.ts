export type Phase = 'setup' | 'group' | 'playoffs' | 'finished';
export type MatchStatus = 'scheduled' | 'inProgress' | 'completed' | 'bye';
export type FormatKind = 'swiss';

export interface TableConfig {
  id: string;
  name: string;
  category: string;
}

export interface TournamentConfig {
  format: FormatKind;
  tables: TableConfig[];
  swissRounds: number;
  playoffTeamCount: number;
  allowByes: boolean;
  randomSeed: number;
  /** Default match duration in minutes. 0 = no timer. */
  defaultMatchMinutes: number;
  /**
   * How many extra swiss-rounds the scheduler may add to achieve exact
   * per-category balance. Defaults to 3 when omitted.
   */
  maxRoundExtension?: number;
  /**
   * How many seeded schedule candidates to generate per game-count level.
   * All successes are compared and the one with the fewest scheduling rounds is used.
   * Defaults to 10 when omitted.
   */
  scheduleAttempts?: number;
}

export interface Team {
  id: string;
  name: string;
}

export interface Match {
  id: string;
  roundIndex: number;
  teamA: string;
  teamB: string;
  /** Cups remaining in this team's rack. Fewer = winning. 0 = all sunk = that team won. */
  cupsA: number;
  cupsB: number;
  status: MatchStatus;
  tableId: string | null;
  startedAt: string | null;
  completedAt: string | null;
  /** Per-match override in seconds. null = use tournament default. */
  timeLimitSeconds: number | null;
  /** ISO timestamp when timer was paused. null = running. */
  pausedAt: string | null;
  /** Accumulated pause duration in seconds. */
  pausedElapsedSeconds: number;
}

export interface Round {
  index: number;
  generatedAt: string;
  matches: Match[];
}

export interface BracketNode {
  matchId: string;
  round: number;
  position: number;
}

export interface Playoffs {
  seeding: string[];
  matches: Match[];
  bracket: BracketNode[];
  thirdPlaceMatch: Match | null;
}

export interface Tournament {
  schemaVersion: 1;
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  config: TournamentConfig;
  teams: Team[];
  phase: Phase;
  rounds: Round[];
  playoffs: Playoffs | null;
}

export interface TeamStanding {
  teamId: string;
  teamName: string;
  points: number;
  wins: number;
  draws: number;
  losses: number;
  cupDiff: number;
  buchholz: number;
  rank: number;
}

export interface TeamBalance {
  teamId: string;
  teamName: string;
  counts: Record<string, number>;
}

export interface TournamentSummary {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  path: string;
}
