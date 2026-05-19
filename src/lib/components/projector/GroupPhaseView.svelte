<script lang="ts">
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import RunningMatch from './RunningMatch.svelte';
  import Scoreboard from './Scoreboard.svelte';
  import type { Match } from '$lib/ipc/types';

  const t = $derived(tournamentStore.value!);
  const totalRounds = $derived(t.rounds.length);

  // ── Which round is the projector showing ─────────────────────────────────
  // Tracks the first incomplete round. Advances automatically as rounds
  // complete; the operator can also click "Next Round →" manually.
  let viewIdx = $state(0);

  $effect(() => {
    const first = t.rounds.findIndex(
      (r) => r.matches.some((m) => m.status !== 'completed' && m.status !== 'bye')
    );
    const target = first === -1 ? totalRounds - 1 : first;
    // Only auto-jump if the projector is lagging behind
    if (viewIdx < target) viewIdx = target;
  });

  const round = $derived(t.rounds[viewIdx] ?? null);

  const playing   = $derived(round?.matches.filter((m) => m.status === 'inProgress')  ?? []);
  const nextUp    = $derived(round?.matches.filter((m) => m.status === 'scheduled')    ?? []);
  const finished  = $derived(round?.matches.filter((m) => m.status === 'completed' || m.status === 'bye') ?? []);
  const roundDone = $derived(round?.matches.every((m) => m.status === 'completed' || m.status === 'bye') ?? false);

  function teamName(id: string) {
    if (id === 'BYE') return 'Bye';
    return t.teams.find((x) => x.id === id)?.name ?? id;
  }

  function tableName(id: string | null) {
    if (!id) return null;
    return t.config.tables.find((x) => x.id === id)?.name ?? id;
  }

  function advanceRound() {
    if (viewIdx < totalRounds - 1) viewIdx++;
  }
</script>

<div class="view">

  <!-- ── Header ────────────────────────────────────────────────────────── -->
  <div class="topbar">
    <span class="round-label">
      Round <strong>{viewIdx + 1}</strong>
      <span class="of">of {totalRounds}</span>
    </span>
    {#if roundDone && viewIdx < totalRounds - 1}
      <button class="next-btn" onclick={advanceRound}>
        Next Round →
      </button>
    {:else if roundDone && viewIdx === totalRounds - 1}
      <span class="all-done">All rounds complete — ready for playoffs!</span>
    {/if}
  </div>

  <!-- ── Main content (active round) ──────────────────────────────────── -->
  {#if !roundDone}
    <!-- PLAYING NOW -->
    {#if playing.length > 0}
      <section class="now-playing">
        <h2>Now Playing</h2>
        <div class="match-grid">
          {#each playing as m (m.id)}
            <RunningMatch match={m} tournament={t} />
          {/each}
        </div>
      </section>
    {:else}
      <section class="now-playing idle-section">
        <h2>Now Playing</h2>
        <p class="idle">No matches in progress</p>
      </section>
    {/if}

    <!-- NEXT UP + FINISHED (in a row at the bottom) -->
    <div class="lower">
      {#if nextUp.length > 0}
        <section class="next-up">
          <h3>Next Up</h3>
          <ul>
            {#each nextUp as m (m.id)}
              {@const tbl = tableName(m.tableId)}
              <li>
                {#if tbl}<span class="table-tag">{tbl}</span>{/if}
                <span class="pair">
                  {teamName(m.teamA)}
                  <span class="vs">vs</span>
                  {teamName(m.teamB)}
                </span>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if finished.length > 0}
        <section class="finished-this-round">
          <h3>Completed this round</h3>
          <ul>
            {#each finished as m (m.id)}
              {@const winner = m.cupsA < m.cupsB ? m.teamA : m.teamB}
              <li>
                <span class="pair muted">
                  <span class:won={m.cupsA < m.cupsB}>{teamName(m.teamA)}</span>
                  <span class="score">{m.cupsA}–{m.cupsB}</span>
                  <span class:won={m.cupsB < m.cupsA}>{teamName(m.teamB)}</span>
                </span>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      <section class="score-section">
        <Scoreboard />
      </section>
    </div>

  {:else}
    <!-- ROUND COMPLETE: show all results -->
    <section class="round-complete">
      <h2>Round {viewIdx + 1} Results</h2>
      <div class="results-grid">
        {#each round!.matches.filter(m => m.status !== 'bye') as m (m.id)}
          {@const aWon = m.cupsA < m.cupsB}
          {@const tbl = tableName(m.tableId)}
          <div class="result-card">
            {#if tbl}<div class="tbl-label">{tbl}</div>{/if}
            <div class="result-row">
              <span class="rteam" class:winner={aWon}>{teamName(m.teamA)}</span>
              <span class="rscore">
                <span class:win={aWon}>{m.cupsA}</span>
                <span class="dash">–</span>
                <span class:win={!aWon}>{m.cupsB}</span>
              </span>
              <span class="rteam right" class:winner={!aWon}>{teamName(m.teamB)}</span>
            </div>
          </div>
        {/each}
      </div>

      {#if viewIdx < totalRounds - 1}
        <div class="next-prompt">
          <button class="next-btn large" onclick={advanceRound}>
            Round {viewIdx + 2} →
          </button>
        </div>
      {:else}
        <p class="all-done large">All rounds complete — ready for playoffs!</p>
      {/if}
    </section>
  {/if}

</div>

<style>
  .view {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    padding: 1.5rem 2rem;
    gap: 1.25rem;
  }

  /* ── Top bar ── */
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-shrink: 0;
  }

  .round-label {
    font-size: 1.1rem;
    color: #6b7280;
    letter-spacing: 0.05em;
  }

  .round-label strong { color: #e2e8f0; font-size: 1.3rem; }
  .of { opacity: 0.6; }

  .next-btn {
    background: #2563eb;
    border: none;
    border-radius: 8px;
    color: #fff;
    cursor: pointer;
    font-size: 1rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    padding: 0.5rem 1.5rem;
    transition: background 0.15s;
  }

  .next-btn:hover { background: #1d4ed8; }
  .next-btn.large { font-size: 1.4rem; padding: 0.7rem 2.5rem; }

  .all-done { color: #4ade80; font-size: 0.95rem; font-weight: 600; }
  .all-done.large { font-size: 1.4rem; text-align: center; }

  /* ── Section headings ── */
  h2 {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: #6b7280;
  }

  h3 {
    margin: 0 0 0.6rem;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #4b5563;
  }

  /* ── Now Playing ── */
  .now-playing { flex: 1; min-height: 0; }
  .idle-section { display: flex; flex-direction: column; }
  .idle { color: #374151; font-size: 1.25rem; margin: 0; }

  .match-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(26rem, 1fr));
    gap: 1rem;
  }

  /* ── Lower row ── */
  .lower {
    display: flex;
    gap: 2rem;
    align-items: flex-start;
    flex-shrink: 0;
  }

  /* ── Next Up ── */
  .next-up { flex: 0 0 auto; min-width: 18rem; }

  .next-up ul, .finished-this-round ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .next-up li, .finished-this-round li {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    font-size: 1rem;
  }

  .table-tag {
    background: #1e2130;
    border: 1px solid #2a2d36;
    border-radius: 4px;
    font-size: 0.72rem;
    font-weight: 700;
    color: #6b7280;
    padding: 0.1rem 0.45rem;
    white-space: nowrap;
    letter-spacing: 0.05em;
  }

  .pair { font-weight: 600; color: #d1d5db; }
  .pair.muted { color: #9ca3af; }
  .vs { color: #374151; font-weight: 400; margin: 0 0.35rem; }

  .pair .won { color: #4ade80; }

  .score {
    font-size: 0.95rem;
    font-variant-numeric: tabular-nums;
    font-weight: 700;
    color: #6b7280;
    margin: 0 0.4rem;
  }

  /* ── Finished this round ── */
  .finished-this-round { flex: 0 0 auto; min-width: 16rem; }

  /* ── Scoreboard ── */
  .score-section { flex: 1; min-width: 0; }

  /* ── Round Complete view ── */
  .round-complete {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .round-complete h2 {
    font-size: 1.2rem;
    color: #9ca3af;
    margin: 0;
  }

  .results-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(22rem, 1fr));
    gap: 0.75rem;
  }

  .result-card {
    background: #141824;
    border: 1px solid #1e2435;
    border-radius: 10px;
    padding: 0.65rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .tbl-label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #374151;
    font-weight: 700;
  }

  .result-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .rteam {
    flex: 1;
    font-size: 1.1rem;
    font-weight: 700;
    color: #9ca3af;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rteam.right { text-align: right; }
  .rteam.winner { color: #4ade80; }

  .rscore {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    font-size: 1.5rem;
    font-weight: 900;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .rscore .win  { color: #4ade80; }
  .rscore .dash { color: #1f2937; font-size: 1rem; }

  .next-prompt {
    display: flex;
    justify-content: center;
    padding: 1rem 0;
  }
</style>
