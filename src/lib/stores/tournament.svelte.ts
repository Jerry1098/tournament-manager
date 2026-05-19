import type { Tournament, TeamStanding } from '$lib/ipc/types';

export const tournamentStore = $state<{ value: Tournament | null }>({ value: null });
export const standingsStore = $state<{ value: TeamStanding[] }>({ value: [] });
export const dirtyStore = $state<{ value: boolean; path: string | null }>({
  value: false,
  path: null,
});
