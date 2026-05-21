<script lang="ts">
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import RunningMatch from './RunningMatch.svelte';
  import Scoreboard from './Scoreboard.svelte';
  import type { Match } from '$lib/ipc/types';

  const t = $derived(tournamentStore.value!);
  const totalRounds = $derived(t.rounds.length);

  let viewIdx = $state(0);

  $effect(() => {
    const first = t.rounds.findIndex(
      (r) => r.matches.some((m) => m.status !== 'completed' && m.status !== 'bye')
    );
    const target = first === -1 ? totalRounds - 1 : first;
    if (viewIdx < target) viewIdx = target;
  });

  const round     = $derived(t.rounds[viewIdx] ?? null);
  const nextRound = $derived(viewIdx < totalRounds - 1 ? t.rounds[viewIdx + 1] : null);

  const playing   = $derived(round?.matches.filter((m) => m.status === 'inProgress')  ?? []);
  const scheduled = $derived(round?.matches.filter((m) => m.status === 'scheduled')   ?? []);
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

  <!-- ── Top bar ─────────────────────────────────────────────────────────── -->
  <div class="topbar">
    <span class="round-label">
      Round <strong>{viewIdx + 1}</strong>
      <span class="of">of {totalRounds}</span>
    </span>
    {#if roundDone && viewIdx < totalRounds - 1}
      <button class="next-btn" onclick={advanceRound}>Next Round →</button>
    {:else if roundDone && viewIdx === totalRounds - 1}
      <span class="all-done">All rounds complete — ready for playoffs!</span>
    {/if}
  </div>

  <!-- ── Body: current-round matches (left+center) + scoreboard (right) ── -->
  <div class="body">

    <!-- Left panel: current round matches + next round preview -->
    <div class="left-panel">

      <!-- Now Playing -->
      {#if playing.length > 0}
        <section class="now-playing">
          <h2>Now Playing</h2>
          <div class="match-grid">
            {#each playing as m (m.id)}
              <RunningMatch match={m} tournament={t} />
            {/each}
          </div>
        </section>
      {:else if scheduled.length === 0 && finished.length === 0}
        <section class="now-playing idle-section">
          <h2>Now Playing</h2>
          <p class="idle">No matches in progress</p>
        </section>
      {/if}

      <!-- Not Started (scheduled matches of current round) -->
      {#if scheduled.length > 0}
        <section class="status-section">
          <h3>Not Started</h3>
          <div class="small-grid">
            {#each scheduled as m (m.id)}
              {@const tbl = tableName(m.tableId)}
              <div class="small-card sched-card">
                {#if tbl}<div class="small-table">{tbl}</div>{/if}
                <div class="small-teams">
                  <span class="small-team">{teamName(m.teamA)}</span>
                  <span class="small-vs">vs</span>
                  <span class="small-team right">{teamName(m.teamB)}</span>
                </div>
              </div>
            {/each}
          </div>
        </section>
      {/if}

      <!-- Completed matches of current round -->
      {#if finished.length > 0}
        <section class="status-section">
          <h3>Completed</h3>
          <div class="small-grid">
            {#each finished as m (m.id)}
              {#if m.status !== 'bye'}
                {@const aWon = m.cupsA < m.cupsB}
                <div class="small-card done-card">
                  <div class="small-teams">
                    <span class="small-team" class:won={aWon}>{teamName(m.teamA)}</span>
                    <div class="small-score-block">
                      <span class="small-score" class:win={aWon}>{m.cupsA}</span>
                      <span class="small-dash">–</span>
                      <span class="small-score" class:win={!aWon}>{m.cupsB}</span>
                    </div>
                    <span class="small-team right" class:won={!aWon}>{teamName(m.teamB)}</span>
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        </section>
      {/if}

      <!-- ── Next round preview ──────────────────────────────────────────── -->
      <div class="next-round-preview">
        {#if nextRound}
          <h3>Up Next — Round {viewIdx + 2}</h3>
          <ul>
            {#each nextRound.matches.filter(m => m.teamB !== 'BYE') as m (m.id)}
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
        {:else if roundDone}
          <span class="all-done">All rounds complete — ready for playoffs!</span>
        {/if}
      </div>

    </div><!-- /left-panel -->

    <!-- Right panel: always-visible standings -->
    <div class="right-panel">
      <Scoreboard />
    </div>

  </div><!-- /body -->

</div>

<style>
  .view {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 1.5rem 2rem;
    gap: 1rem;
    overflow: hidden;
  }

  /* ── Top bar ── */
  .topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-shrink: 0;
  }

  .round-label { font-size: 1.1rem; color: #6b7280; letter-spacing: 0.05em; }
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
  .all-done { color: #4ade80; font-size: 0.95rem; font-weight: 600; }

  /* ── Body (left + right) ── */
  .body {
    display: flex;
    flex: 1;
    min-height: 0;
    gap: 2rem;
  }

  /* ── Left panel ── */
  .left-panel {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow-y: auto;
  }

  /* Section headings */
  h2 {
    margin: 0 0 0.65rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: #6b7280;
  }

  h3 {
    margin: 0 0 0.5rem;
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #4b5563;
  }

  /* ── Now Playing ── */
  .now-playing { flex-shrink: 0; }
  .idle-section { display: flex; flex-direction: column; }
  .idle { color: #374151; font-size: 1.25rem; margin: 0; }

  .match-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(26rem, 1fr));
    gap: 1rem;
  }

  /* ── Status sections (not-started + completed) ── */
  .status-section { flex-shrink: 0; }

  /* Small card grid — same column logic as match-grid but narrower min */
  .small-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr));
    gap: 0.65rem;
  }

  .small-card {
    border-radius: 10px;
    padding: 0.7rem 1rem 0.85rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .sched-card {
    background: #0e1220;
    border: 1.5px solid #2a3050;
  }

  .done-card {
    background: #0d1a10;
    border: 1.5px solid #14532d;
  }

  .small-table {
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: #6b7280;
  }

  .small-teams {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .small-team {
    flex: 1;
    font-size: 1rem;
    font-weight: 800;
    color: #d1d5db;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.2;
  }

  .small-team.right { text-align: right; }
  .small-team.won   { color: #4ade80; }

  .small-vs {
    font-size: 0.8rem;
    color: #374151;
    flex-shrink: 0;
  }

  .small-score-block {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    flex-shrink: 0;
  }

  .small-score {
    font-size: 1.5rem;
    font-weight: 900;
    font-variant-numeric: tabular-nums;
    line-height: 1;
    color: #9ca3af;
  }

  .small-score.win { color: #4ade80; }
  .small-dash      { font-size: 1rem; opacity: 0.3; }

  /* Next-round preview retains list layout */
  .next-round-preview ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .next-round-preview li {
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
  .vs   { color: #374151; font-weight: 400; margin: 0 0.3rem; }

  /* ── Next round preview ── */
  .next-round-preview { flex-shrink: 0; margin-top: auto; padding-top: 0.5rem; border-top: 1px solid #1e243540; }

  /* ── Right panel (scoreboard) ── */
  .right-panel {
    width: 20rem;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  /* Light mode */
  :global(body.light) .round-label { color: #6b7280; }
  :global(body.light) .round-label strong { color: #111827; }
  :global(body.light) .all-done { color: #16a34a; }
  :global(body.light) h2 { color: #6b7280; }
  :global(body.light) h3 { color: #9ca3af; }
  :global(body.light) .sched-card { background: #f3f4f6; border-color: #d1d5db; }
  :global(body.light) .done-card  { background: #f0fdf4; border-color: #16a34a; }
  :global(body.light) .small-team  { color: #111827; }
  :global(body.light) .small-team.won { color: #16a34a; }
  :global(body.light) .small-vs   { color: #9ca3af; }
  :global(body.light) .small-score { color: #374151; }
  :global(body.light) .small-score.win { color: #16a34a; }
  :global(body.light) .table-tag { background: #e8eaed; border-color: #d1d5db; color: #6b7280; }
  :global(body.light) .pair { color: #111827; }
  :global(body.light) .vs  { color: #9ca3af; }
  :global(body.light) .idle { color: #9ca3af; }
  :global(body.light) .next-round-preview { border-top-color: #d1d5db40; }
</style>
