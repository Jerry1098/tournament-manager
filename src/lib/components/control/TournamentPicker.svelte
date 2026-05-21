<script lang="ts">
  import { onMount } from 'svelte';
  import { deleteTournament, listTournaments, loadTournament } from '$lib/ipc/commands';
  import type { TournamentSummary } from '$lib/ipc/types';

  let { onNewTournament }: { onNewTournament: () => void } = $props();

  let tournaments = $state<TournamentSummary[]>([]);
  let loading = $state(true);
  let deletingPath = $state<string | null>(null);

  onMount(async () => {
    await refresh();
  });

  async function refresh() {
    loading = true;
    tournaments = await listTournaments().catch(() => []);
    loading = false;
  }

  async function handleOpen(path: string) {
    await loadTournament(path).catch(console.error);
  }

  async function handleDelete(t: TournamentSummary) {
    const ok = confirm(`Delete "${t.name}"? This cannot be undone.`);
    if (!ok) return;
    deletingPath = t.path;
    await deleteTournament(t.path).catch(console.error);
    deletingPath = null;
    await refresh();
  }

  function formatDate(iso: string) {
    return new Date(iso).toLocaleString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
</script>

<div class="picker">
  <div class="picker-header">
    <h2>Tournament Manager</h2>
    <button class="primary" onclick={onNewTournament}>+ New Tournament</button>
  </div>

  {#if loading}
    <p class="hint">Loading…</p>
  {:else if tournaments.length === 0}
    <div class="empty">
      <p>No saved tournaments yet.</p>
      <p class="sub">Create one to get started.</p>
    </div>
  {:else}
    <ul class="list">
      {#each tournaments as t (t.id)}
        <li class="item">
          <button class="item-main" onclick={() => handleOpen(t.path)}>
            <span class="name">{t.name}</span>
            <span class="date">Last updated {formatDate(t.updatedAt)}</span>
          </button>
          <button
            class="delete-btn"
            title="Delete tournament"
            disabled={deletingPath === t.path}
            onclick={() => handleDelete(t)}
          >
            ✕
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    max-width: 36rem;
    margin: 6vh auto 0;
  }

  .picker-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .picker-header h2 {
    margin: 0;
    font-size: 1.4rem;
  }

  .hint {
    opacity: 0.5;
    text-align: center;
    padding: 2rem;
  }

  .empty {
    text-align: center;
    padding: 2.5rem 1rem;
    border: 1px dashed #3a3d4a;
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .empty p { margin: 0; opacity: 0.6; }
  .empty .sub { font-size: 0.85rem; opacity: 0.4; }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .item {
    display: flex;
    align-items: stretch;
    background: #1e2028;
    border: 1px solid #2a2d36;
    border-radius: 8px;
    overflow: hidden;
    transition: border-color 0.15s;
  }

  .item:hover { border-color: #4a5068; }

  .item-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
    padding: 0.75rem 1rem;
    background: transparent;
    border: none;
    color: inherit;
    cursor: pointer;
    text-align: left;
  }

  .item-main:hover { background: #252830; }

  .name {
    font-weight: 600;
    font-size: 0.95rem;
  }

  .date {
    font-size: 0.78rem;
    opacity: 0.45;
  }

  .delete-btn {
    padding: 0 0.85rem;
    background: transparent;
    border: none;
    border-left: 1px solid #2a2d36;
    color: #6b7280;
    cursor: pointer;
    font-size: 0.8rem;
    transition: background 0.15s, color 0.15s;
  }

  .delete-btn:hover {
    background: #3b1414;
    color: #f87171;
  }

  .delete-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  button.primary {
    background: #2563eb;
    border: none;
    border-radius: 7px;
    color: #fff;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 600;
    padding: 0.5rem 1.1rem;
    white-space: nowrap;
  }

  button.primary:hover { background: #1d4ed8; }

  /* Light mode */
  :global(body.light) .item {
    background: #f8f9fb;
    border-color: #d1d5db;
  }

  :global(body.light) .item:hover { border-color: #9ca3af; }

  :global(body.light) .item-main:hover { background: #f0f2f5; }

  :global(body.light) .delete-btn {
    border-left-color: #d1d5db;
    color: #9ca3af;
  }

  :global(body.light) .delete-btn:hover {
    background: #fee2e2;
    color: #dc2626;
  }

  :global(body.light) .empty {
    border-color: #d1d5db;
  }
</style>
