<script lang="ts">
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import type { Match } from '$lib/ipc/types';
  import PlayoffMatchCard from './PlayoffMatchCard.svelte';

  const t = $derived(tournamentStore.value!);
  const playoffs = $derived(t.playoffs!);

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
      else if (r === 1) labels.push('Semi-finals');
      else if (r === 2) labels.push('Quarter-finals');
      else labels.push(`Round of ${2 ** (r + 1)}`);
    }
    return labels;
  });
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
              <PlayoffMatchCard {match} />
            {:else}
              <div class="match-placeholder"><span class="tbd">TBD</span></div>
            {/if}
          {/each}
        </div>
      </div>
    {/each}
  </div>

  <!-- 3rd-place match -->
  {#if playoffs.thirdPlaceMatch}
    <div class="third-place">
      <h3>3rd Place</h3>
      <div class="third-card">
        <PlayoffMatchCard match={playoffs.thirdPlaceMatch} />
      </div>
    </div>
  {/if}
</div>

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
    align-items: flex-start;
    min-width: min-content;
  }

  .round-col {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 20rem;
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
    gap: 0.65rem;
    flex: 1;
  }

  .match-placeholder {
    background: #22252e;
    border: 1px solid #3a3d4a;
    border-radius: 10px;
    min-height: 9.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.35;
  }

  .tbd {
    font-size: 0.85rem;
    color: #6b7280;
  }

  .third-place {
    border-top: 1px solid #2a2d36;
    padding-top: 1.25rem;
  }

  .third-card {
    max-width: 22rem;
  }

  /* Light mode */
  :global(body.light) .bracket-wrap h2,
  :global(body.light) .bracket-wrap h3 { color: #111827; }

  :global(body.light) .round-label {
    border-bottom-color: #e5e7eb;
    color: #6b7280;
  }

  :global(body.light) .match-placeholder {
    background: #ffffff;
    border-color: #d1d5db;
  }

  :global(body.light) .third-place { border-top-color: #e5e7eb; }
</style>
