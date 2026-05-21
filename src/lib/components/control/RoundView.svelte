<script lang="ts">
  import { startAllAssignedMatches } from '$lib/ipc/commands';
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import MatchCard from './MatchCard.svelte';

  let { roundIndex }: { roundIndex: number } = $props();

  const t = $derived(tournamentStore.value!);
  const round = $derived(t.rounds[roundIndex]);
  const totalRounds = $derived(t.rounds.length);

  const allTerminal = $derived(
    round?.matches.every((m) => m.status === 'completed' || m.status === 'bye') ?? false
  );

  const hasAssignedScheduled = $derived(
    round?.matches.some((m) => m.status === 'scheduled' && m.tableId) ?? false
  );

  let startingAll = $state(false);

  async function handleStartAll() {
    startingAll = true;
    await startAllAssignedMatches(roundIndex).catch((e) => alert(e));
    startingAll = false;
  }
</script>

{#if round}
  <div class="round-view">
    <div class="round-header">
      <div class="title-block">
        <h3>Round {roundIndex + 1} <span class="of">of {totalRounds}</span></h3>
        <span class="match-count">
          {round.matches.filter(m => m.status !== 'bye').length} matches
          {#if allTerminal}<span class="done-badge">✓ Done</span>{/if}
        </span>
      </div>

      {#if hasAssignedScheduled}
        <button class="start-all" onclick={handleStartAll} disabled={startingAll}>
          {startingAll ? 'Starting…' : '▶ Start all assigned'}
        </button>
      {/if}
    </div>

    <div class="match-grid">
      {#each round.matches as match (match.id)}
        <MatchCard {match} />
      {/each}
    </div>
  </div>
{/if}

<style>
  .round-view {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .round-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .title-block {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
  }

  h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 700;
  }

  .of { opacity: 0.4; font-weight: 400; font-size: 0.9rem; }

  .match-count {
    font-size: 0.8rem;
    color: #7a7f8e;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .done-badge {
    background: #14532d;
    color: #86efac;
    border-radius: 4px;
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.1rem 0.4rem;
  }

  .match-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(22rem, 1fr));
    gap: 0.65rem;
  }

  .start-all {
    background: #1e3a8a;
    border: 1px solid #2563eb;
    border-radius: 6px;
    color: #93c5fd;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 600;
    padding: 0.3rem 0.85rem;
    flex-shrink: 0;
  }

  .start-all:hover:not(:disabled) { background: #1d4ed8; color: #fff; }
  .start-all:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
