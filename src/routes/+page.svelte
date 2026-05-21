<script lang="ts">
  import { generateNextRound, startPlayoffs } from '$lib/ipc/commands';
  import TournamentBar from '$lib/components/control/TournamentBar.svelte';
  import ConfigPanel from '$lib/components/control/ConfigPanel.svelte';
  import TournamentPicker from '$lib/components/control/TournamentPicker.svelte';
  import TeamList from '$lib/components/control/TeamList.svelte';
  import RoundView from '$lib/components/control/RoundView.svelte';
  import StandingsTable from '$lib/components/control/StandingsTable.svelte';
  import BalanceReport from '$lib/components/control/BalanceReport.svelte';
  import PlayoffBracket from '$lib/components/control/PlayoffBracket.svelte';
  import TournamentResults from '$lib/components/control/TournamentResults.svelte';
  import { tournamentStore } from '$lib/stores/tournament.svelte';

  let showNewTournament = $state(false);
  let activeRound = $state(0);
  let generatingFirst = $state(false);

  const t = $derived(tournamentStore.value);
  const phase = $derived(t?.phase);

  // Plain variable (not reactive) — used to detect when rounds are added.
  // Reading it inside $effect creates no dependency, so it won't cause loops.
  let _prevRoundCount = 0;

  $effect(() => {
    if (!t || t.rounds.length === 0) return;
    const count = t.rounds.length;
    const firstIncomplete = t.rounds.findIndex(
      (r) => r.matches.some((m) => m.status !== 'completed' && m.status !== 'bye')
    );
    const target = firstIncomplete === -1 ? count - 1 : firstIncomplete;

    if (count !== _prevRoundCount) {
      // Rounds were added (or initial load) — auto-navigate to the active round.
      if (activeRound >= count || activeRound === target - 1) {
        activeRound = target;
      }
      _prevRoundCount = count;
    } else if (activeRound >= count) {
      // Fix out-of-bounds without overriding a deliberate selection.
      activeRound = count - 1;
    }
  });

  // All rounds done = every match in every round is terminal
  const allSwissRoundsDone = $derived(
    t != null &&
    t.rounds.length > 0 &&
    t.rounds.every((r) => r.matches.every((m) => m.status === 'completed' || m.status === 'bye'))
  );

  let startingPlayoffs = $state(false);

  async function handleStartPlayoffs() {
    startingPlayoffs = true;
    await startPlayoffs().catch((e) => alert(e));
    startingPlayoffs = false;
  }
</script>

<div class="app">
  <TournamentBar onNewTournament={() => { showNewTournament = true; }} />

  <main class="content">
    {#if showNewTournament && !t}
      <ConfigPanel onCreated={() => { showNewTournament = false; }} />

    {:else if !t}
      <TournamentPicker onNewTournament={() => { showNewTournament = true; }} />

    {:else if phase === 'setup'}
      <div class="setup-layout">
        <TeamList />
        <div class="setup-hint">
          <h3>Ready to start?</h3>

          <!-- Live scheduling calculator -->
          {#if t.teams.length > 0 && t.config.tables.length > 0}
            {@const totalMatches = Math.ceil(t.teams.length * t.config.swissRounds / 2)}
            {@const schedRounds = Math.ceil(totalMatches / t.config.tables.length)}
            <div class="schedule-calc">
              <span class="calc-row">
                <span class="val">{t.teams.length}</span> teams ×
                <span class="val">{t.config.swissRounds}</span> matches ÷ 2 ÷
                <span class="val">{t.config.tables.length}</span> tables
              </span>
              <span class="calc-arrow">→</span>
              <span class="calc-result">
                <strong>{schedRounds}</strong> scheduling rounds
              </span>
            </div>
          {/if}

          <ul class="config-summary">
            <li><span>Matches per team</span><span>{t.config.swissRounds}</span></li>
            <li><span>Playoff teams</span><span>{t.config.playoffTeamCount}</span></li>
            <li><span>Tables</span><span>{t.config.tables.length}</span></li>
            <li><span>Teams added</span><span class:ok={t.teams.length >= 2}>{t.teams.length}</span></li>
          </ul>

          {#if t.teams.length >= 2}
            <button
              class="primary"
              disabled={generatingFirst}
              onclick={async () => {
                generatingFirst = true;
                await generateNextRound().catch((e) => alert(e));
                generatingFirst = false;
              }}
            >
              {generatingFirst ? 'Generating schedule…' : 'Generate Schedule & Start'}
            </button>
          {:else}
            <p class="muted">Add at least 2 teams to begin.</p>
          {/if}
        </div>
      </div>

    {:else if phase === 'group'}
      <div class="group-layout">
        <!-- Left: round tabs + match cards -->
        <div class="rounds-panel">
          {#if t.rounds.length > 0}
            <!-- Round tabs (all pre-generated) -->
            <div class="tabs">
              {#each t.rounds as r, i}
                <button
                  class="tab"
                  class:active={activeRound === i}
                  onclick={() => { activeRound = i; }}
                >
                  Round {i + 1}
                  {#if r.matches.every(m => m.status === 'completed' || m.status === 'bye')}
                    <span class="check">✓</span>
                  {/if}
                </button>
              {/each}
            </div>

            <RoundView roundIndex={activeRound} />

            {#if allSwissRoundsDone}
              <div class="playoffs-cta">
                <h3>Swiss phase complete!</h3>
                <p>Top {t.config.playoffTeamCount} teams will advance to playoffs.</p>
                <button class="primary" onclick={handleStartPlayoffs} disabled={startingPlayoffs}>
                  {startingPlayoffs ? 'Starting…' : 'Start Playoffs'}
                </button>
              </div>
            {/if}
          {/if}
        </div>

        <!-- Right: standings + balance -->
        <aside class="sidebar">
          <StandingsTable />
          <BalanceReport />
        </aside>
      </div>

    {:else if phase === 'playoffs' && t.playoffs}
      <div class="group-layout">
        <div class="rounds-panel">
          <PlayoffBracket />
        </div>
        <aside class="sidebar">
          <StandingsTable />
        </aside>
      </div>

    {:else if phase === 'finished'}
      <TournamentResults />
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .content {
    flex: 1;
    padding: 1.25rem 1.5rem;
    overflow-y: auto;
    min-height: 0;
  }

  /* Setup */
  .setup-layout {
    display: flex;
    gap: 2rem;
    align-items: flex-start;
    max-width: 56rem;
  }

  .setup-hint {
    flex: 1;
    background: #2a2d36;
    border-radius: 10px;
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .setup-hint h3 { margin: 0; }
  .setup-hint p  { margin: 0; opacity: 0.8; font-size: 0.9rem; }

  /* Scheduling calculator */
  .schedule-calc {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    background: #0d0f14;
    border: 1px solid #2a2d36;
    border-radius: 8px;
    padding: 0.65rem 0.9rem;
    font-size: 0.82rem;
    flex-wrap: wrap;
    color: #7a7f8e;
  }

  .calc-row .val { color: #f1f3f7; font-weight: 600; }
  .calc-arrow    { color: #4a4f60; }
  .calc-result   { color: #f1f3f7; }
  .calc-result strong { color: #60a5fa; font-size: 1.05rem; }

  /* Config summary list */
  .setup-hint ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  .setup-hint li {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    color: #7a7f8e;
    border-bottom: 1px solid #2a2d3640;
    padding-bottom: 0.25rem;
  }

  .setup-hint li span:last-child { color: #b8bcc8; font-weight: 600; }
  .setup-hint li span:last-child.ok { color: #4ade80; }

  /* Group */
  .group-layout {
    display: flex;
    gap: 1.5rem;
    align-items: flex-start;
    min-height: 100%;
  }

  .rounds-panel {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .tabs {
    display: flex;
    gap: 0.3rem;
    flex-wrap: wrap;
  }

  .tab {
    background: #2a2d36;
    border: 1px solid #3a3d4a;
    border-radius: 6px;
    color: #e9ecef;
    cursor: pointer;
    font-size: 0.82rem;
    padding: 0.3rem 0.85rem;
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .tab.active { background: #1e2a4a; border-color: #2563eb; }
  .tab .check { color: #4ade80; font-size: 0.7rem; }

  .sidebar {
    width: 18rem;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 0;
    position: sticky;
    top: 0;
    max-height: calc(100vh - 3.5rem);
    overflow-y: auto;
  }

  .no-rounds {
    text-align: center;
    padding: 3rem 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }

  .no-rounds p { opacity: 0.5; margin: 0; }

  .playoffs-cta {
    background: #1e2a1e;
    border: 1px solid #14532d;
    border-radius: 10px;
    padding: 1.25rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    align-items: flex-start;
    margin-top: 0.5rem;
  }

  .playoffs-cta h3 { margin: 0; color: #4ade80; }
  .playoffs-cta p  { margin: 0; font-size: 0.875rem; opacity: 0.7; }

  .muted { opacity: 0.45 !important; font-size: 0.8rem !important; }

  button.primary {
    background: #2563eb;
    border: none;
    border-radius: 7px;
    color: #fff;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 600;
    padding: 0.5rem 1.25rem;
  }

  button.primary:disabled { opacity: 0.45; cursor: not-allowed; }

  /* Light mode overrides */
  :global(body.light) .setup-hint { background: #e8eaed; }

  :global(body.light) .schedule-calc {
    background: #f0f2f5;
    border-color: #d1d5db;
    color: #6b7280;
  }

  :global(body.light) .calc-row .val,
  :global(body.light) .calc-result { color: #111827; }
  :global(body.light) .calc-result strong { color: #2563eb; }

  :global(body.light) .setup-hint li {
    color: #6b7280;
    border-bottom-color: #d1d5db;
  }

  :global(body.light) .setup-hint li span:last-child { color: #374151; }
  :global(body.light) .setup-hint li span:last-child.ok { color: #16a34a; }

  :global(body.light) .tab {
    background: #e8eaed;
    border-color: #d1d5db;
    color: #111827;
  }

  :global(body.light) .tab.active {
    background: #dbeafe;
    border-color: #2563eb;
    color: #1e3a8a;
  }

  :global(body.light) .tab .check { color: #16a34a; }

  :global(body.light) .playoffs-cta {
    background: #f0fdf4;
    border-color: #16a34a;
  }

  :global(body.light) .playoffs-cta h3 { color: #16a34a; }
  :global(body.light) .playoffs-cta p { color: #374151; }
</style>
