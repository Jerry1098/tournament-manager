<script lang="ts">
  import { getBalanceReport } from '$lib/ipc/commands';
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import type { TeamBalance } from '$lib/ipc/types';

  let report = $state<TeamBalance[]>([]);

  $effect(() => {
    // Re-fetch whenever tournament changes (matches submitted, round generated)
    if (tournamentStore.value) {
      getBalanceReport().then((r) => { report = r; }).catch(() => {});
    }
  });

  const categories = $derived(() => {
    const cats = new Set<string>();
    report.forEach((b) => Object.keys(b.counts).forEach((c) => cats.add(c)));
    return [...cats].sort();
  });

  // Total games played by each team (sum of all categories)
  const maxPlayed = $derived(() => {
    return Math.max(1, ...report.map((b) => Object.values(b.counts).reduce((a, v) => a + v, 0)));
  });

  function imbalanced(b: TeamBalance): boolean {
    const vals = Object.values(b.counts);
    if (vals.length < 2) return false;
    const min = Math.min(...vals);
    const max = Math.max(...vals);
    return max - min > 1;
  }
</script>

{#if report.length > 0 && categories().length > 1}
  <section class="balance">
    <h3>Location balance</h3>
    <table>
      <thead>
        <tr>
          <th class="left">Team</th>
          {#each categories() as cat}
            <th>{cat}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each report as b (b.teamId)}
          <tr class:warn={imbalanced(b)}>
            <td class="name">{b.teamName}</td>
            {#each categories() as cat}
              <td class="num">{b.counts[cat] ?? 0}</td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
    <p class="hint">Teams are highlighted if their category counts differ by more than 1.</p>
  </section>
{/if}

<style>
  .balance { margin-top: 1.5rem; }

  h3 {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.6;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8rem;
  }

  th {
    text-align: right;
    font-weight: 600;
    opacity: 0.5;
    padding: 0.2rem 0.5rem 0.4rem;
    font-size: 0.75rem;
    text-transform: capitalize;
  }

  th.left { text-align: left; }

  td {
    padding: 0.3rem 0.5rem;
    text-align: right;
    border-top: 1px solid #2a2d36;
  }

  td.name { text-align: left; font-weight: 500; }
  td.num { font-variant-numeric: tabular-nums; }

  tr.warn td { background: #422006; }
  tr.warn td.name::after { content: ' ⚠'; color: #f59e0b; font-size: 0.7rem; }

  .hint {
    margin: 0.4rem 0 0;
    font-size: 0.72rem;
    opacity: 0.4;
  }
</style>
