<script lang="ts">
  import { getStandings } from '$lib/ipc/commands';
  import { standingsStore, tournamentStore } from '$lib/stores/tournament.svelte';
  import type { TeamStanding } from '$lib/ipc/types';

  // Load once on mount; live updates come via standingsStore (event-driven)
  $effect(() => {
    if (tournamentStore.value) {
      getStandings().then((s) => { standingsStore.value = s; }).catch(() => {});
    }
  });

  const standings = $derived(standingsStore.value);

  function sign(n: number) {
    return n > 0 ? `+${n}` : `${n}`;
  }
</script>

<section class="standings">
  <h3>Standings</h3>
  {#if standings.length === 0}
    <p class="empty">No results yet.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>#</th>
          <th class="left">Team</th>
          <th title="Wins">W</th>
          <th title="Draws">D</th>
          <th title="Losses">L</th>
          <th title="Cup difference">±</th>
          <th title="Buchholz">BH</th>
        </tr>
      </thead>
      <tbody>
        {#each standings as s (s.teamId)}
          <tr>
            <td class="rank">{s.rank}</td>
            <td class="name">{s.teamName}</td>
            <td class="num">{s.wins}</td>
            <td class="num muted">{s.draws}</td>
            <td class="num">{s.losses}</td>
            <td class="num" class:pos={s.cupDiff > 0} class:neg={s.cupDiff < 0}>{sign(s.cupDiff)}</td>
            <td class="num muted">{s.buchholz.toFixed(1)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .standings { min-width: 16rem; }

  h3 {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.6;
  }

  .empty { opacity: 0.4; font-size: 0.875rem; }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  th {
    text-align: right;
    font-weight: 600;
    opacity: 0.5;
    padding: 0.2rem 0.5rem 0.4rem;
    font-size: 0.75rem;
    letter-spacing: 0.05em;
  }

  th.left { text-align: left; }

  td {
    padding: 0.35rem 0.5rem;
    text-align: right;
    border-top: 1px solid #2a2d36;
  }

  td.rank { opacity: 0.4; width: 1.5rem; }
  td.name { text-align: left; font-weight: 500; }
  td.num { font-variant-numeric: tabular-nums; }
  td.muted { opacity: 0.5; }
  td.pos { color: #4ade80; }
  td.neg { color: #f87171; }

  tr:hover td { background: #2a2d3640; }

  /* Light mode */
  :global(body.light) td { border-top-color: #e5e7eb; }
  :global(body.light) tr:hover td { background: rgba(0,0,0,0.04); }
  :global(body.light) td.pos { color: #16a34a; }
  :global(body.light) td.neg { color: #dc2626; }
</style>
