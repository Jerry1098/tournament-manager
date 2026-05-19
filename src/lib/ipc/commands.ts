import { invoke } from '@tauri-apps/api/core';
import type {
  Match,
  Playoffs,
  Round,
  Team,
  TeamBalance,
  TeamStanding,
  Tournament,
  TournamentConfig,
} from './types';

// Tournament
export const createTournament = (name: string, config: TournamentConfig) =>
  invoke<Tournament>('create_tournament', { name, config });

export const loadTournament = (path?: string) =>
  invoke<Tournament>('load_tournament', { path: path ?? null });

export const saveTournamentAs = (path: string) =>
  invoke<void>('save_tournament_as', { path });

export const closeTournament = () => invoke<void>('close_tournament');

export const getTournament = () => invoke<Tournament | null>('get_tournament');

export const renameTournament = (name: string) =>
  invoke<Tournament>('rename_tournament', { name });

// Teams
export const addTeam = (name: string) => invoke<Team>('add_team', { name });

export const renameTeam = (id: string, name: string) =>
  invoke<void>('rename_team', { id, name });

export const removeTeam = (id: string) => invoke<void>('remove_team', { id });

// Rounds
/** Generates the entire group-phase schedule in one shot. Returns void; use the
 *  tournament-updated event to hydrate state. */
export const generateNextRound = () => invoke<void>('generate_next_round');

export const reassignTable = (matchId: string, tableId: string | null) =>
  invoke<void>('reassign_table', { matchId, tableId });

// Matches
export const startMatch = (matchId: string) =>
  invoke<void>('start_match', { matchId });

export const startAllAssignedMatches = () =>
  invoke<number>('start_all_assigned_matches');

export const submitResult = (matchId: string, cupsA: number, cupsB: number) =>
  invoke<void>('submit_result', { matchId, cupsA, cupsB });

export const editResult = (matchId: string, cupsA: number, cupsB: number) =>
  invoke<void>('edit_result', { matchId, cupsA, cupsB });

export const cancelMatch = (matchId: string) =>
  invoke<void>('cancel_match', { matchId });

export const pauseMatchTimer = (matchId: string) =>
  invoke<void>('pause_match_timer', { matchId });

export const resumeMatchTimer = (matchId: string) =>
  invoke<void>('resume_match_timer', { matchId });

export const setMatchTimeLimit = (matchId: string, seconds: number | null) =>
  invoke<void>('set_match_time_limit', { matchId, seconds });

// Playoffs
export const startPlayoffs = () => invoke<Playoffs>('start_playoffs');

export const submitPlayoffResult = (matchId: string, cupsA: number, cupsB: number) =>
  invoke<void>('submit_playoff_result', { matchId, cupsA, cupsB });

// Standings
export const getStandings = () => invoke<TeamStanding[]>('get_standings');

export const getBalanceReport = () => invoke<TeamBalance[]>('get_balance_report');

// Window
export const openProjector = () => invoke<void>('open_projector');

export const toggleFullscreenProjector = () =>
  invoke<void>('toggle_fullscreen_projector');
