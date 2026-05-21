<script lang="ts">
  import { standingsStore } from '$lib/stores/tournament.svelte';

  const standings = $derived(standingsStore.value.slice(0, 10));

  function sign(n: number) { return n > 0 ? `+${n}` : `${n}`; }
</script>

<section class="scoreboard">
  <h2>Standings</h2>
  {#if standings.length === 0}
    <p class="empty">No results yet</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>#</th>
          <th class="left">Team</th>
          <th>W</th>
          <th>L</th>
          <th title="Cup difference">±</th>
          <th title="Buchholz">BH</th>
        </tr>
      </thead>
      <tbody>
        {#each standings as s, i (s.teamId)}
          <tr class:top={i < 3}>
            <td class="rank">{s.rank}</td>
            <td class="name">{s.teamName}</td>
            <td>{s.wins}</td>
            <td class="muted">{s.losses}</td>
            <td class:pos={s.cupDiff > 0} class:neg={s.cupDiff < 0}>{sign(s.cupDiff)}</td>
            <td class="muted">{s.buchholz.toFixed(1)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .scoreboard { width: 100%; }

  h2 {
    margin: 0 0 0.75rem;
    font-size: 1.25rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.7;
  }

  .empty { opacity: 0.4; font-size: 1.25rem; }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 1.1rem;
  }

  th {
    text-align: right;
    font-weight: 600;
    opacity: 0.4;
    padding: 0.3rem 0.75rem 0.5rem;
    font-size: 0.85rem;
    letter-spacing: 0.06em;
  }

  th.left { text-align: left; }

  td {
    padding: 0.5rem 0.75rem;
    text-align: right;
    border-top: 1px solid #1e2435;
    font-variant-numeric: tabular-nums;
  }

  td.rank { opacity: 0.35; width: 2rem; }
  td.name { text-align: left; font-weight: 700; font-size: 1.2rem; }
  td.muted { opacity: 0.5; }
  td.pos { color: #4ade80; }
  td.neg { color: #f87171; }

  tr.top td { background: #ffffff08; }
  tr.top td.name { color: #fbbf24; }

  /* Light mode */
  :global(body.light) td { border-top-color: #e5e7eb; }
  :global(body.light) tr.top td { background: rgba(0,0,0,0.04); }
  :global(body.light) tr.top td.name { color: #d97706; }
</style>
