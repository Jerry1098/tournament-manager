import { listen } from '@tauri-apps/api/event';
import type { Match, Phase, Round, Tournament, TeamStanding } from './types';

export const onTournamentUpdated = (cb: (t: Tournament) => void) =>
  listen<Tournament>('tournament-updated', (e) => cb(e.payload));

export const onTournamentLoaded = (cb: (t: Tournament) => void) =>
  listen<Tournament>('tournament-loaded', (e) => cb(e.payload));

export const onTournamentClosed = (cb: () => void) =>
  listen('tournament-closed', () => cb());

export const onMatchUpdated = (cb: (m: Match) => void) =>
  listen<Match>('match-updated', (e) => cb(e.payload));

export const onRoundGenerated = (cb: (r: Round) => void) =>
  listen<Round>('round-generated', (e) => cb(e.payload));

export const onPhaseChanged = (cb: (from: Phase, to: Phase) => void) =>
  listen<{ from: Phase; to: Phase }>('phase-changed', (e) =>
    cb(e.payload.from, e.payload.to)
  );

export const onStandingsUpdated = (cb: (s: TeamStanding[]) => void) =>
  listen<TeamStanding[]>('standings-updated', (e) => cb(e.payload));

export const onDirtyChanged = (cb: (dirty: boolean, path: string | null) => void) =>
  listen<{ dirty: boolean; path: string | null }>('dirty-changed', (e) =>
    cb(e.payload.dirty, e.payload.path)
  );
