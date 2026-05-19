<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { getTournament } from '$lib/ipc/commands';
  import {
    onTournamentUpdated,
    onTournamentClosed,
    onStandingsUpdated,
    onDirtyChanged,
  } from '$lib/ipc/events';
  import { tournamentStore, standingsStore, dirtyStore } from '$lib/stores/tournament.svelte';

  let { children } = $props();

  onMount(() => {
    // Hydrate from Rust state on window load
    getTournament().then((t) => {
      if (t) tournamentStore.value = t;
    }).catch(() => {});

    // Subscribe to Rust events (all return Promise<() => void>)
    const unsubs: Promise<() => void>[] = [
      onTournamentUpdated((t) => { tournamentStore.value = t; }),
      onTournamentClosed(() => { tournamentStore.value = null; standingsStore.value = []; }),
      onStandingsUpdated((s) => { standingsStore.value = s; }),
      onDirtyChanged((dirty, path) => { dirtyStore.value = dirty; dirtyStore.path = path; }),
    ];

    return () => {
      unsubs.forEach((p) => p.then((u) => u()).catch(() => {}));
    };
  });
</script>

{@render children()}
