<script lang="ts">
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import type { Match } from '$lib/ipc/types';

  const t = $derived(tournamentStore.value!);
  const playoffs = $derived(t.playoffs!);

  function teamName(id: string) {
    if (!id) return '?';
    return t.teams.find((x) => x.id === id)?.name ?? id;
  }

  const maxRound = $derived(Math.max(0, ...playoffs.bracket.map((n) => n.round)));

  const rounds = $derived(() => {
    const result: Match[][] = [];
    for (let r = maxRound; r >= 0; r--) {
      const nodes = playoffs.bracket
        .filter((n) => n.round === r)
        .sort((a, b) => a.position - b.position);
      result.push(nodes.map((n) => playoffs.matches.find((m) => m.id === n.matchId)!));
    }
    return result;
  });

  function roundLabel(col: number) {
    const r = maxRound - col;
    if (r === 0) return 'Final';
    if (r === 1) return 'Semi-finals';
    if (r === 2) return 'Quarter-finals';
    return `Round of ${2 ** (r + 1)}`;
  }
</script>

<div class="view">
  <h1>Playoffs</h1>

  <div class="bracket">
    {#each rounds() as roundMatches, col}
      <div class="round-col">
        <div class="round-label">{roundLabel(col)}</div>
        <div class="match-col">
          {#each roundMatches as match (match?.id)}
            <div
              class="match"
              class:completed={match?.status === 'completed'}
              class:pending={!match?.teamA || !match?.teamB}
            >
              {#if match}
                <div class="team" class:winner={match.status === 'completed' && match.cupsA > match.cupsB}>
                  <span class="name">{teamName(match.teamA)}</span>
                  {#if match.status === 'completed'}<span class="cups">{match.cupsA}</span>{/if}
                </div>
                <div class="team" class:winner={match.status === 'completed' && match.cupsB > match.cupsA}>
                  <span class="name">{teamName(match.teamB)}</span>
                  {#if match.status === 'completed'}<span class="cups">{match.cupsB}</span>{/if}
                </div>
              {:else}
                <div class="tbd">TBD</div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  {#if playoffs.thirdPlaceMatch}
    {@const tpm = playoffs.thirdPlaceMatch}
    <div class="third">
      <span class="third-label">3rd Place</span>
      <span class="name">{teamName(tpm.teamA)}</span>
      <span class="vs">vs</span>
      <span class="name">{teamName(tpm.teamB)}</span>
      {#if tpm.status === 'completed'}
        <span class="result">→ {teamName(tpm.cupsA > tpm.cupsB ? tpm.teamA : tpm.teamB)} wins</span>
      {/if}
    </div>
  {/if}
</div>

<style>
  .view {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    padding: 2rem;
    gap: 2rem;
  }

  h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 900;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    opacity: 0.85;
  }

  .bracket {
    display: flex;
    gap: 2rem;
    align-items: center;
    overflow-x: auto;
    flex: 1;
  }

  .round-col {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-width: 18rem;
  }

  .round-label {
    text-align: center;
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    opacity: 0.4;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #1e2435;
  }

  .match-col {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    justify-content: space-around;
    flex: 1;
  }

  .match {
    background: #14172040;
    border: 2px solid #2a2d36;
    border-radius: 10px;
    overflow: hidden;
    min-height: 6rem;
    display: flex;
    flex-direction: column;
  }

  .match.completed { border-color: #166534; }
  .match.pending { opacity: 0.35; }

  .team {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.6rem 1rem;
    border-bottom: 1px solid #1e243510;
    min-height: 2.75rem;
    gap: 0.75rem;
  }

  .team.winner { background: #14532d40; }
  .team.winner .name { color: #4ade80; font-weight: 800; }

  .name {
    font-size: 1.25rem;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cups {
    font-size: 1.75rem;
    font-weight: 900;
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }

  .tbd {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 6rem;
    opacity: 0.2;
    font-size: 1rem;
  }

  .third {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    border-top: 1px solid #1e2435;
    padding-top: 1.25rem;
    font-size: 1.2rem;
  }

  .third-label {
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    opacity: 0.45;
    flex-shrink: 0;
  }

  .vs { opacity: 0.4; }
  .result { color: #4ade80; font-weight: 700; margin-left: 0.5rem; }
</style>
