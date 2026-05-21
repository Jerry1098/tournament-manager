<script lang="ts">
  import { submitPlayoffResult } from '$lib/ipc/commands';
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import type { Match } from '$lib/ipc/types';
  import ResultDialog from './ResultDialog.svelte';

  const t = $derived(tournamentStore.value!);
  const playoffs = $derived(t.playoffs!);

  let showDialog = $state<Match | null>(null);

  function teamName(id: string) {
    if (!id) return '?';
    return t.teams.find((x) => x.id === id)?.name ?? id;
  }

  // Group matches by round for display
  const maxRound = $derived(
    Math.max(0, ...playoffs.bracket.map((n) => n.round))
  );

  // rounds[r] = array of matches in round r, ordered by position
  const rounds = $derived(() => {
    const result: Match[][] = [];
    for (let r = maxRound; r >= 0; r--) {
      const nodes = playoffs.bracket
        .filter((n) => n.round === r)
        .sort((a, b) => a.position - b.position);
      result.push(
        nodes.map((n) => playoffs.matches.find((m) => m.id === n.matchId)!)
      );
    }
    return result; // rounds[0] = first round (leftmost), rounds[last] = final
  });

  const roundLabels = $derived(() => {
    const labels: string[] = [];
    for (let r = maxRound; r >= 0; r--) {
      if (r === 0) labels.push('Final');
      else if (r === 1) labels.push(maxRound >= 2 ? 'Semi-finals' : 'Semi-finals');
      else if (r === 2) labels.push('Quarter-finals');
      else labels.push(`Round of ${2 ** (r + 1)}`);
    }
    return labels;
  });

  async function handleSubmit(match: Match, cupsA: number, cupsB: number) {
    await submitPlayoffResult(match.id, cupsA, cupsB);
  }
</script>

<div class="bracket-wrap">
  <h2>Playoffs</h2>

  <div class="bracket">
    {#each rounds() as roundMatches, col}
      <div class="round-col">
        <div class="round-label">{roundLabels()[col]}</div>
        <div class="match-col">
          {#each roundMatches as match (match?.id)}
            {#if match}
              <div
                class="match"
                class:in-progress={match.status === 'inProgress'}
                class:completed={match.status === 'completed'}
                class:pending={!match.teamA || !match.teamB}
              >
                <div class="team" class:winner={match.status === 'completed' && match.cupsA > match.cupsB}>
                  <span class="name">{teamName(match.teamA)}</span>
                  {#if match.status === 'completed'}
                    <span class="cups">{match.cupsA}</span>
                  {/if}
                </div>
                <div class="team" class:winner={match.status === 'completed' && match.cupsB > match.cupsA}>
                  <span class="name">{teamName(match.teamB)}</span>
                  {#if match.status === 'completed'}
                    <span class="cups">{match.cupsB}</span>
                  {/if}
                </div>
                {#if match.teamA && match.teamB && match.status !== 'completed'}
                  <button
                    class="result-btn"
                    onclick={() => { showDialog = match; }}
                  >
                    Enter result
                  </button>
                {/if}
              </div>
            {:else}
              <div class="match pending"><span class="tbd">TBD</span></div>
            {/if}
          {/each}
        </div>
      </div>
    {/each}
  </div>

  <!-- 3rd-place match -->
  {#if playoffs.thirdPlaceMatch}
    {@const tpm = playoffs.thirdPlaceMatch}
    <div class="third-place">
      <h3>3rd Place</h3>
      <div class="match" class:completed={tpm.status === 'completed'}>
        <div class="team" class:winner={tpm.status === 'completed' && tpm.cupsA > tpm.cupsB}>
          <span class="name">{teamName(tpm.teamA)}</span>
          {#if tpm.status === 'completed'}<span class="cups">{tpm.cupsA}</span>{/if}
        </div>
        <div class="team" class:winner={tpm.status === 'completed' && tpm.cupsB > tpm.cupsA}>
          <span class="name">{teamName(tpm.teamB)}</span>
          {#if tpm.status === 'completed'}<span class="cups">{tpm.cupsB}</span>{/if}
        </div>
        {#if tpm.teamA && tpm.teamB && tpm.status !== 'completed'}
          <button class="result-btn" onclick={() => { showDialog = tpm; }}>Enter result</button>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#if showDialog}
  <ResultDialog
    match={showDialog}
    tournament={t}
    onSubmit={(a, b) => handleSubmit(showDialog!, a, b)}
    onClose={() => { showDialog = null; }}
  />
{/if}

<style>
  .bracket-wrap {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    overflow-x: auto;
    padding-bottom: 1rem;
  }

  h2 { margin: 0; font-size: 1.25rem; }
  h3 { margin: 0 0 0.5rem; font-size: 1rem; opacity: 0.75; }

  .bracket {
    display: flex;
    gap: 1.5rem;
    align-items: center;
    min-width: min-content;
  }

  .round-col {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 14rem;
  }

  .round-label {
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    opacity: 0.5;
    text-align: center;
    padding-bottom: 0.25rem;
    border-bottom: 1px solid #2a2d36;
  }

  .match-col {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    justify-content: space-around;
    flex: 1;
  }

  .match {
    background: #22252e;
    border: 1px solid #3a3d4a;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 5rem;
  }

  .match.in-progress { border-color: #2563eb; }
  .match.completed { border-color: #166534; }
  .match.pending { opacity: 0.4; }

  .team {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.4rem 0.75rem;
    gap: 0.5rem;
    border-bottom: 1px solid #2a2d3640;
    min-height: 2rem;
  }

  .team:last-of-type { border-bottom: none; }
  .team.winner { background: #14532d30; }
  .team.winner .name { color: #4ade80; font-weight: 700; }

  .name {
    font-size: 0.9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .cups {
    font-weight: 700;
    font-size: 1rem;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .tbd {
    display: block;
    text-align: center;
    padding: 0.75rem;
    opacity: 0.3;
    font-size: 0.85rem;
  }

  .result-btn {
    background: transparent;
    border: none;
    border-top: 1px solid #2a2d36;
    color: #60a5fa;
    cursor: pointer;
    font-size: 0.78rem;
    padding: 0.3rem;
    text-align: center;
    width: 100%;
  }

  .result-btn:hover { background: #1e2a4a; }

  .third-place {
    border-top: 1px solid #2a2d36;
    padding-top: 1.25rem;
    max-width: 16rem;
  }

  /* Light mode */
  :global(body.light) .bracket-wrap h2,
  :global(body.light) .bracket-wrap h3 { color: #111827; }

  :global(body.light) .round-label {
    border-bottom-color: #e5e7eb;
    color: #6b7280;
  }

  :global(body.light) .match {
    background: #ffffff;
    border-color: #d1d5db;
  }

  :global(body.light) .match.in-progress { border-color: #2563eb; }
  :global(body.light) .match.completed { border-color: #16a34a; }

  :global(body.light) .team {
    border-bottom-color: rgba(0, 0, 0, 0.08);
  }

  :global(body.light) .team.winner { background: #f0fdf4; }
  :global(body.light) .team.winner .name { color: #16a34a; }

  :global(body.light) .name { color: #111827; }
  :global(body.light) .cups { color: #111827; }

  :global(body.light) .result-btn {
    color: #2563eb;
    border-top-color: #e5e7eb;
  }

  :global(body.light) .result-btn:hover { background: #dbeafe; }

  :global(body.light) .third-place { border-top-color: #e5e7eb; }
</style>
