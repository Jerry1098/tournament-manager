<script lang="ts">
  import {
    cancelMatch,
    editResult,
    pauseMatchTimer,
    reassignTable,
    resumeMatchTimer,
    setMatchTimeLimit,
    startMatch,
    submitResult,
  } from '$lib/ipc/commands';
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import type { Match } from '$lib/ipc/types';
  import ResultDialog from './ResultDialog.svelte';

  let { match }: { match: Match } = $props();

  const t = $derived(tournamentStore.value!);
  const teamA = $derived(t.teams.find((x) => x.id === match.teamA)?.name ?? match.teamA);
  const teamB = $derived(t.teams.find((x) => x.id === match.teamB)?.name ?? match.teamB);
  const isBye = $derived(match.teamB === 'BYE');
  const isPaused = $derived(match.pausedAt != null);

  // Effective time limit in seconds (per-match override or tournament default)
  const timeLimitSec = $derived(() => {
    if (match.timeLimitSeconds != null) return match.timeLimitSeconds;
    const mins = t.config.defaultMatchMinutes;
    return mins > 0 ? mins * 60 : 0;
  });

  // Live elapsed seconds — updates every second while running
  let elapsedSec = $state(0);

  $effect(() => {
    if (match.status !== 'inProgress' || !match.startedAt) {
      elapsedSec = 0;
      return;
    }

    const compute = () => {
      const startMs = new Date(match.startedAt!).getTime();
      const pausedElapsed = match.pausedElapsedSeconds ?? 0;
      if (match.pausedAt) {
        const pausedMs = new Date(match.pausedAt).getTime();
        elapsedSec = Math.floor((pausedMs - startMs) / 1000) - pausedElapsed;
      } else {
        elapsedSec = Math.floor((Date.now() - startMs) / 1000) - pausedElapsed;
      }
    };

    compute();
    if (!match.pausedAt) {
      const id = setInterval(compute, 1000);
      return () => clearInterval(id);
    }
  });

  const remainingSec = $derived(() => {
    const lim = timeLimitSec();
    if (lim <= 0) return null; // no timer
    return lim - elapsedSec;
  });

  function formatTime(sec: number): string {
    const abs = Math.abs(sec);
    const m = Math.floor(abs / 60).toString().padStart(2, '0');
    const s = (abs % 60).toString().padStart(2, '0');
    return `${sec < 0 ? '+' : ''}${m}:${s}`;
  }

  const timerClass = $derived(() => {
    const r = remainingSec();
    if (r == null) return '';
    if (r < 0) return 'overtime';
    if (r < 60) return 'urgent';
    if (r < 120) return 'warning';
    return 'ok';
  });

  let showDialog = $state<'submit' | 'edit' | null>(null);
  let editingTimeLimit = $state(false);
  let timeLimitInput = $state('');

  // Winner = team with fewer cups
  const winner = $derived(() => {
    if (match.status !== 'completed') return null;
    if (match.cupsA < match.cupsB) return match.teamA;
    if (match.cupsB < match.cupsA) return match.teamB;
    return null;
  });

  const statusLabel: Record<string, string> = {
    scheduled: 'Scheduled',
    inProgress: 'Playing',
    completed: 'Done',
    bye: 'Bye',
  };

  async function handleStart() {
    await startMatch(match.id).catch((e) => alert(e));
  }

  async function handleCancel() {
    await cancelMatch(match.id).catch((e) => alert(e));
  }

  async function handleSubmit(cupsA: number, cupsB: number) {
    await submitResult(match.id, cupsA, cupsB);
  }

  async function handleEdit(cupsA: number, cupsB: number) {
    await editResult(match.id, cupsA, cupsB);
  }

  async function handleTableChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    await reassignTable(match.id, val || null).catch((e) => alert(e));
  }

  async function handlePauseResume() {
    if (match.pausedAt) {
      await resumeMatchTimer(match.id).catch((e) => alert(e));
    } else {
      await pauseMatchTimer(match.id).catch((e) => alert(e));
    }
  }

  function startEditTimeLimit() {
    const lim = timeLimitSec();
    timeLimitInput = lim > 0 ? String(Math.floor(lim / 60)) : '';
    editingTimeLimit = true;
  }

  async function commitTimeLimit() {
    editingTimeLimit = false;
    const mins = parseInt(timeLimitInput, 10);
    const secs = isNaN(mins) || mins <= 0 ? null : mins * 60;
    await setMatchTimeLimit(match.id, secs).catch((e) => alert(e));
  }
</script>

<div
  class="card"
  class:in-progress={match.status === 'inProgress'}
  class:completed={match.status === 'completed'}
  class:bye={isBye}
>
  <!-- Header: status + table selector -->
  <div class="header">
    <span class="status {match.status}">{statusLabel[match.status] ?? match.status}</span>

    {#if !isBye && match.status !== 'completed'}
      <select
        class="table-select"
        value={match.tableId ?? ''}
        onchange={handleTableChange}
        disabled={match.status === 'inProgress'}
      >
        <option value="">— no table —</option>
        {#each t.config.tables as tbl}
          <option value={tbl.id}>{tbl.name} ({tbl.category})</option>
        {/each}
      </select>
    {:else if match.tableId}
      <span class="table-badge">
        {t.config.tables.find((x) => x.id === match.tableId)?.name ?? ''}
      </span>
    {/if}
  </div>

  <!-- Teams + scores -->
  <div class="matchup">
    <span class="team" class:winner={winner() === match.teamA}>{teamA}</span>
    <div class="score">
      {#if match.status === 'completed'}
        <span class="cups" class:win={winner() === match.teamA}>{match.cupsA}</span>
        <span class="dash">–</span>
        <span class="cups" class:win={winner() === match.teamB}>{match.cupsB}</span>
      {:else if isBye}
        <span class="bye-label">free win</span>
      {:else}
        <span class="dash">vs</span>
      {/if}
    </div>
    <span class="team right" class:winner={winner() === match.teamB}>
      {isBye ? '' : teamB}
    </span>
  </div>

  <!-- Timer row (only for in-progress matches with a time limit) -->
  {#if match.status === 'inProgress' && !isBye}
    <div class="timer-row">
      {#if timeLimitSec() > 0}
        <span class="timer {timerClass()}">
          {#if (remainingSec() ?? 1) < 0}⏱ +{formatTime(elapsedSec - timeLimitSec())}
          {:else}⏱ {formatTime(remainingSec()!)}
          {/if}
        </span>
      {:else}
        <span class="timer ok">⏱ {formatTime(elapsedSec)}</span>
      {/if}

      <button class="icon-btn" onclick={handlePauseResume} title={match.pausedAt ? 'Resume' : 'Pause'}>
        {match.pausedAt ? '▶' : '⏸'}
      </button>

      {#if editingTimeLimit}
        <input
          type="number"
          class="time-input"
          bind:value={timeLimitInput}
          placeholder="min"
          min="1"
          max="120"
          onblur={commitTimeLimit}
          onkeydown={(e) => e.key === 'Enter' && commitTimeLimit()}
          autofocus
        />
      {:else}
        <button class="icon-btn muted" onclick={startEditTimeLimit} title="Set time limit">
          {timeLimitSec() > 0 ? `${Math.floor(timeLimitSec() / 60)}min` : 'set limit'}
        </button>
      {/if}
    </div>
  {/if}

  <!-- Actions -->
  {#if !isBye}
    <div class="actions">
      {#if match.status === 'scheduled'}
        <button
          class="small primary"
          onclick={handleStart}
          disabled={!match.tableId}
          title={!match.tableId ? 'Assign a table first' : ''}
        >
          Start
        </button>
      {:else if match.status === 'inProgress'}
        <button class="small" onclick={handleCancel}>Cancel</button>
        <button class="small primary" onclick={() => { showDialog = 'submit'; }}>Submit result</button>
      {:else if match.status === 'completed'}
        <button class="small" onclick={() => { showDialog = 'edit'; }}>Edit result</button>
      {/if}
    </div>
  {/if}
</div>

{#if showDialog === 'submit'}
  <ResultDialog
    {match}
    tournament={t}
    onSubmit={handleSubmit}
    onClose={() => { showDialog = null; }}
  />
{:else if showDialog === 'edit'}
  <ResultDialog
    {match}
    tournament={t}
    onSubmit={handleEdit}
    onClose={() => { showDialog = null; }}
  />
{/if}

<style>
  .card {
    background: #22252e;
    border: 1px solid #3a3d4a;
    border-radius: 10px;
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    transition: border-color 0.15s;
    min-height: 9.5rem;
  }

  .card.in-progress { border-color: #2563eb; background: #1a1f30; }
  .card.completed   { border-color: #166534; background: #141e17; }
  .card.bye         { opacity: 0.5; }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .status {
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    padding: 0.12rem 0.45rem;
    border-radius: 4px;
    flex-shrink: 0;
  }

  .status.scheduled  { background: #2a2d36; color: #9ca3af; }
  .status.inProgress { background: #1e3a8a; color: #93c5fd; }
  .status.completed  { background: #14532d; color: #86efac; }
  .status.bye        { background: #2a2d36; color: #6b7280; }

  /* table-select inherits from global app.css; just size it down */
  .table-select {
    font-size: 0.78rem;
    padding: 0.15rem 1.6rem 0.15rem 0.45rem;
    max-width: 14rem;
  }

  .table-select:disabled { opacity: 0.5; cursor: default; }

  .table-badge {
    font-size: 0.75rem;
    color: #9ca3af;
    background: #2a2d36;
    border-radius: 4px;
    padding: 0.1rem 0.45rem;
  }

  .matchup {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .team {
    flex: 1;
    font-size: 0.95rem;
    font-weight: 600;
    color: #f1f3f7;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .team.right  { text-align: right; }
  .team.winner { color: #4ade80; }

  .score {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 1.05rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .cups { color: #f1f3f7; }
  .cups.win { color: #4ade80; }
  .dash { opacity: 0.35; font-weight: 400; font-size: 0.85rem; }
  .bye-label { opacity: 0.45; font-size: 0.75rem; font-weight: 400; }

  /* Timer */
  .timer-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-top: 0.1rem;
  }

  .timer {
    font-variant-numeric: tabular-nums;
    font-size: 0.9rem;
    font-weight: 700;
    flex: 1;
  }

  .timer.ok      { color: #4ade80; }
  .timer.warning { color: #fbbf24; }
  .timer.urgent  { color: #fb923c; }
  .timer.overtime{ color: #f87171; }

  .icon-btn {
    background: #2a2d36;
    border: 1px solid #3a3d4a;
    border-radius: 5px;
    color: #f1f3f7;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0.2rem 0.5rem;
    flex-shrink: 0;
  }

  .icon-btn.muted { color: #7a7f8e; font-size: 0.72rem; }
  .icon-btn:hover { background: #363a48; }

  .time-input {
    background: #32363f;
    border: 1px solid #2563eb;
    border-radius: 5px;
    color: #f1f3f7;
    font-size: 0.78rem;
    padding: 0.15rem 0.4rem;
    width: 4rem;
  }

  .actions {
    display: flex;
    gap: 0.4rem;
    justify-content: flex-end;
  }

  button.small {
    background: #2a2d36;
    border: 1px solid #3a3d4a;
    border-radius: 5px;
    color: #f1f3f7;
    cursor: pointer;
    font-size: 0.78rem;
    padding: 0.25rem 0.7rem;
  }

  button.small:disabled { opacity: 0.35; cursor: not-allowed; }
  button.small.primary { background: #2563eb; border-color: #1d4ed8; }
  button.small.primary:hover:not(:disabled) { background: #1d4ed8; }

  /* Light mode */
  :global(body.light) .card { background: #ffffff; border-color: #d1d5db; }
  :global(body.light) .card.in-progress { background: #eff6ff; border-color: #2563eb; }
  :global(body.light) .card.completed   { background: #f0fdf4; border-color: #16a34a; }

  :global(body.light) .status.scheduled  { background: #f3f4f6; color: #6b7280; }
  :global(body.light) .status.inProgress { background: #dbeafe; color: #1d4ed8; }
  :global(body.light) .status.completed  { background: #dcfce7; color: #166534; }
  :global(body.light) .status.bye        { background: #f3f4f6; color: #9ca3af; }

  :global(body.light) .table-badge { background: #e8eaed; color: #6b7280; }
  :global(body.light) .team  { color: #111827; }
  :global(body.light) .cups  { color: #111827; }

  :global(body.light) .icon-btn {
    background: #e8eaed;
    border-color: #d1d5db;
    color: #374151;
  }
  :global(body.light) .icon-btn:hover { background: #d1d5db; }
  :global(body.light) .time-input {
    background: var(--bg-input);
    border-color: var(--border-focus);
    color: var(--text-primary);
  }
  :global(body.light) button.small {
    background: #e8eaed;
    border-color: #d1d5db;
    color: #111827;
  }
  :global(body.light) button.small:hover:not(:disabled) { background: #d1d5db; }
  :global(body.light) button.small.primary { background: #2563eb; border-color: #1d4ed8; color: #fff; }
  :global(body.light) button.small.primary:hover:not(:disabled) { background: #1d4ed8; }
</style>
