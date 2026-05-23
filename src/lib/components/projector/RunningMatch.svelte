<script lang="ts">
  import type { Match, Tournament } from '$lib/ipc/types';

  let { match, tournament }: { match: Match; tournament: Tournament } = $props();

  const teamA  = $derived(tournament.teams.find((t) => t.id === match.teamA)?.name ?? match.teamA);
  const teamB  = $derived(tournament.teams.find((t) => t.id === match.teamB)?.name ?? match.teamB);
  const table  = $derived(tournament.config.tables.find((t) => t.id === match.tableId));
  const winner = $derived(
    match.status === 'completed'
      ? (match.cupsA > match.cupsB ? match.teamA : match.cupsB > match.cupsA ? match.teamB : null)
      : null
  );

  // ── Timer ──────────────────────────────────────────────────────────────────
  const timeLimitSec = $derived(
    match.timeLimitSeconds != null
      ? match.timeLimitSeconds
      : (tournament.config.defaultMatchMinutes ?? 0) * 60
  );

  let elapsedSec = $state(0);

  $effect(() => {
    if (match.status !== 'inProgress' || !match.startedAt) {
      elapsedSec = 0;
      return;
    }

    const compute = () => {
      const startMs       = new Date(match.startedAt!).getTime();
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

  const remainingSec  = $derived(timeLimitSec > 0 ? timeLimitSec - elapsedSec : null);
  const isOvertime    = $derived(remainingSec != null && remainingSec < 0);
  const isPaused      = $derived(!!match.pausedAt);

  const timerClass = $derived(
    remainingSec == null       ? 'neutral'  :
    remainingSec < 0           ? 'overtime' :
    remainingSec < 60          ? 'urgent'   :
    remainingSec < 120         ? 'warning'  : 'ok'
  );

  function fmt(sec: number): string {
    const abs = Math.abs(sec);
    const m   = Math.floor(abs / 60).toString().padStart(2, '0');
    const s   = (abs % 60).toString().padStart(2, '0');
    return `${m}:${s}`;
  }
</script>

<div class="card" class:completed={match.status === 'completed'} class:paused={isPaused}>
  <!-- Table name -->
  {#if table}
    <div class="table-label">
      {table.name}
      {#if table.category}<span class="cat">· {table.category}</span>{/if}
    </div>
  {/if}

  <!-- Teams + score -->
  <div class="teams">
    <span class="team" class:winner={winner === match.teamA}>{teamA}</span>
    <div class="center-block">
      {#if match.status === 'completed'}
        <div class="scores">
          <span class="score" class:win={winner === match.teamA}>{match.cupsA}</span>
          <span class="dash">–</span>
          <span class="score" class:win={winner === match.teamB}>{match.cupsB}</span>
        </div>
      {:else}
        <span class="vs">vs</span>
      {/if}

      <!-- Timer (shown while in-progress) -->
      {#if match.status === 'inProgress'}
        <div class="timer {timerClass}" class:blink={isOvertime && !isPaused}>
          {#if isPaused}
            <span class="pause-icon">⏸</span>
          {:else if isOvertime}
            <span class="overtime-label">+{fmt(elapsedSec - timeLimitSec)}</span>
          {:else if remainingSec != null}
            {fmt(remainingSec)}
          {:else}
            {fmt(elapsedSec)}
          {/if}
        </div>
      {/if}
    </div>
    <span class="team right" class:winner={winner === match.teamB}>{teamB}</span>
  </div>
</div>

<style>
  .card {
    background: #141824;
    border: 2px solid #2563eb;
    border-radius: 14px;
    padding: 1rem 1.5rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .card.completed { border-color: #166534; background: #0f1c12; }
  .card.paused    { border-color: #78350f; background: #1a150a; }

  .table-label {
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: #6b7280;
  }

  .cat { font-weight: 400; opacity: 0.7; }

  .teams {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .team {
    flex: 1;
    font-size: 1.6rem;
    font-weight: 800;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #e2e8f0;
    line-height: 1.15;
  }

  .team.right   { text-align: right; }
  .team.winner  { color: #4ade80; }

  /* Center column: vs / scores + timer stacked */
  .center-block {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
    flex-shrink: 0;
  }

  .scores {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .score {
    font-size: 2.75rem;
    font-weight: 900;
    font-variant-numeric: tabular-nums;
    line-height: 1;
    color: #e2e8f0;
  }

  .score.win { color: #4ade80; }
  .dash      { font-size: 1.75rem; opacity: 0.3; }

  .vs {
    font-size: 1rem;
    opacity: 0.35;
    letter-spacing: 0.05em;
  }

  /* ── Timer ── */
  .timer {
    font-size: 1.5rem;
    font-weight: 900;
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.03em;
    line-height: 1;
    min-width: 5rem;
    text-align: center;
    padding: 0.15rem 0.5rem;
    border-radius: 6px;
  }

  .timer.ok       { color: #4ade80; }
  .timer.warning  { color: #fbbf24; }
  .timer.urgent   { color: #fb923c; background: #431a0840; }
  .timer.overtime { color: #f87171; background: #3b0d0d60; }
  .timer.neutral  { color: #64748b; }

  .pause-icon     { font-size: 1.25rem; }
  .overtime-label { font-size: 1.4rem; }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50%       { opacity: 0.35; }
  }

  .blink { animation: blink 1s step-end infinite; }

  /* Light mode */
  :global(body.light) .card { background: #ffffff; }
  :global(body.light) .card.completed { background: #f0fdf4; border-color: #16a34a; }
  :global(body.light) .card.paused { background: #fffbeb; border-color: #d97706; }
  :global(body.light) .team { color: #111827; }
  :global(body.light) .score { color: #111827; }
  :global(body.light) .table-label { color: #6b7280; }
  :global(body.light) .dash { color: #9ca3af; }
</style>
