<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { getTournament } from '$lib/ipc/commands';
  import {
    onTournamentUpdated,
    onTournamentClosed,
    onStandingsUpdated,
    onDirtyChanged,
  } from '$lib/ipc/events';
  import { tournamentStore, standingsStore, dirtyStore } from '$lib/stores/tournament.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';

  let { children } = $props();

  $effect(() => {
    document.body.classList.toggle('light', themeStore.value === 'light');
  });

  onMount(() => {
    const saved = localStorage.getItem('theme');
    if (saved === 'light' || saved === 'dark') themeStore.value = saved;

    getTournament().then((t) => {
      if (t) tournamentStore.value = t;
    }).catch(() => {});

    const unsubs: Promise<() => void>[] = [
      onTournamentUpdated((t) => { tournamentStore.value = t; }),
      onTournamentClosed(() => { tournamentStore.value = null; standingsStore.value = []; }),
      onStandingsUpdated((s) => { standingsStore.value = s; }),
      onDirtyChanged((dirty, path) => { dirtyStore.value = dirty; dirtyStore.path = path; }),
      listen<'dark' | 'light'>('theme-changed', (e) => {
        themeStore.value = e.payload;
        localStorage.setItem('theme', e.payload);
      }),
    ];

    return () => {
      unsubs.forEach((p) => p.then((u) => u()).catch(() => {}));
    };
  });
</script>

{@render children()}
