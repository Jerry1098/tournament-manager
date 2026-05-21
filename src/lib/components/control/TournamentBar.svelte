<script lang="ts">
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { emit } from '@tauri-apps/api/event';
  import {
    closeTournament,
    loadTournament,
    openProjector,
    renameTournament,
    saveTournamentAs,
  } from '$lib/ipc/commands';
  import { dirtyStore, tournamentStore } from '$lib/stores/tournament.svelte';
  import { themeStore } from '$lib/stores/theme.svelte';

  let { onNewTournament }: { onNewTournament: () => void } = $props();

  let editingName = $state(false);
  let nameInput = $state('');

  async function handleLoad() {
    const selected = await open({ filters: [{ name: 'Tournament', extensions: ['json'] }] });
    if (typeof selected === 'string') {
      await loadTournament(selected).catch(console.error);
    }
  }

  async function handleSaveAs() {
    const path = await save({ filters: [{ name: 'Tournament', extensions: ['json'] }] });
    if (path) await saveTournamentAs(path).catch(console.error);
  }

  async function handleClose() {
    if (dirtyStore.value) {
      const ok = confirm('You have unsaved changes. Close without saving?');
      if (!ok) return;
    }
    await closeTournament().catch(console.error);
  }

  function startEditName() {
    nameInput = tournamentStore.value?.name ?? '';
    editingName = true;
  }

  async function commitName() {
    editingName = false;
    if (nameInput && nameInput !== tournamentStore.value?.name) {
      await renameTournament(nameInput).catch(console.error);
    }
  }

  function toggleTheme() {
    const next: 'dark' | 'light' = themeStore.value === 'dark' ? 'light' : 'dark';
    themeStore.value = next;
    localStorage.setItem('theme', next);
    emit('theme-changed', next).catch(() => {});
  }
</script>

<header class="bar">
  <div class="left">
    {#if tournamentStore.value}
      {#if editingName}
        <input
          class="name-input"
          bind:value={nameInput}
          onblur={commitName}
          onkeydown={(e) => e.key === 'Enter' && commitName()}
          autofocus
        />
      {:else}
        <button class="name ghost" onclick={startEditName}>
          {tournamentStore.value.name}
          {#if dirtyStore.value}<span class="dot" title="Unsaved changes">●</span>{/if}
        </button>
      {/if}
    {:else}
      <span class="app-name">Tournament Manager</span>
    {/if}
  </div>

  <nav class="actions">
    <button class="theme-btn" onclick={toggleTheme} title={themeStore.value === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}>
      {themeStore.value === 'dark' ? '☀' : '🌙'}
    </button>
    <button onclick={onNewTournament}>New</button>
    <button onclick={handleLoad}>Open…</button>
    {#if tournamentStore.value}
      <button onclick={handleSaveAs}>Save As…</button>
      <button onclick={handleClose} class="danger">Close</button>
      <button class="accent" onclick={openProjector}>Open Projector</button>
    {/if}
  </nav>
</header>

<style>
  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 1.25rem;
    background: #13151a;
    border-bottom: 1px solid #2a2d36;
    gap: 1rem;
    min-height: 3rem;
  }

  .app-name {
    font-weight: 600;
    font-size: 1rem;
    opacity: 0.8;
  }

  .name {
    font-weight: 600;
    font-size: 1rem;
    cursor: pointer;
    background: none;
    border: none;
    color: inherit;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
  }

  .name:hover { background: #2a2d36; }

  .dot {
    color: #f59e0b;
    margin-left: 0.3rem;
    font-size: 0.5rem;
    vertical-align: super;
  }

  .name-input {
    font-weight: 600;
    font-size: 1rem;
    background: #2a2d36;
    border: 1px solid #4a5568;
    border-radius: 4px;
    color: inherit;
    padding: 0.2rem 0.5rem;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  button {
    background: #2a2d36;
    border: 1px solid #3a3d4a;
    border-radius: 6px;
    color: #e9ecef;
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0.35rem 0.85rem;
    transition: background 0.15s;
  }

  button:hover { background: #363a48; }
  button.ghost { background: transparent; border-color: transparent; }
  button.accent { background: #2563eb; border-color: #1d4ed8; }
  button.accent:hover { background: #1d4ed8; }
  button.danger { color: #f87171; }

  .theme-btn {
    font-size: 1rem;
    padding: 0.3rem 0.6rem;
    line-height: 1;
  }

  /* Light mode overrides */
  :global(body.light) .bar {
    background: #ffffff;
    border-bottom-color: #d1d5db;
  }

  :global(body.light) .actions button {
    background: #e8eaed;
    border-color: #d1d5db;
    color: #111827;
  }

  :global(body.light) .actions button:hover {
    background: #d1d5db;
  }

  :global(body.light) .actions button.ghost {
    background: transparent;
    border-color: transparent;
  }

  :global(body.light) .actions button.accent {
    background: #2563eb;
    border-color: #1d4ed8;
    color: #fff;
  }

  :global(body.light) .actions button.accent:hover {
    background: #1d4ed8;
  }

  :global(body.light) .actions button.danger {
    background: #fee2e2;
    color: #dc2626;
    border-color: #fecaca;
  }

  :global(body.light) .name:hover { background: #e8eaed; }

  :global(body.light) .name-input {
    background: #f0f2f5;
    border-color: #d1d5db;
    color: #111827;
  }
</style>
