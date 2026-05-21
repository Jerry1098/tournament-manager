<script lang="ts">
  import { onMount } from 'svelte';
  import { getStandings } from '$lib/ipc/commands';
  import { standingsStore, tournamentStore } from '$lib/stores/tournament.svelte';

  const t = $derived(tournamentStore.value!);

  onMount(() => {
    getStandings().then((s) => { standingsStore.value = s; }).catch(() => {});
  });

  const standings = $derived(standingsStore.value);

  function teamName(id: string) {
    if (!id) return '?';
    return t.teams.find((x) => x.id === id)?.name ?? id;
  }

  const finalMatch = $derived(t.playoffs?.matches[0]);
  const thirdMatch = $derived(t.playoffs?.thirdPlaceMatch);

  const first = $derived(() => {
    if (!finalMatch) return null;
    return teamName(finalMatch.cupsA < finalMatch.cupsB ? finalMatch.teamA : finalMatch.teamB);
  });
  const second = $derived(() => {
    if (!finalMatch) return null;
    return teamName(finalMatch.cupsA < finalMatch.cupsB ? finalMatch.teamB : finalMatch.teamA);
  });
  const third = $derived(() => {
    if (!thirdMatch) return null;
    return teamName(thirdMatch.cupsA < thirdMatch.cupsB ? thirdMatch.teamA : thirdMatch.teamB);
  });

  function sign(n: number) {
    return n > 0 ? `+${n}` : `${n}`;
  }
</script>

<div class="view">

  <!-- Left: podium + standings -->
  <div class="main-panel">
    <h1>Tournament Complete</h1>

    <!-- Podium -->
    <div class="podium">
      <div class="place second">
        <div class="medal-ring">2</div>
        <div class="place-name">{second()}</div>
        <div class="place-label">2nd Place</div>
      </div>
      <div class="place first">
        <div class="medal-ring gold">1</div>
        <div class="place-name">{first()}</div>
        <div class="place-label">1st Place</div>
      </div>
      {#if third()}
        <div class="place third">
          <div class="medal-ring bronze">3</div>
          <div class="place-name">{third()}</div>
          <div class="place-label">3rd Place</div>
        </div>
      {/if}
    </div>

    <!-- Standings table -->
    {#if standings.length > 0}
      <div class="scoreboard">
        <h2>Final Standings</h2>
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th class="left">Team</th>
              <th title="Wins">W</th>
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
                <td>{s.wins}</td>
                <td class="muted">{s.losses}</td>
                <td class:pos={s.cupDiff > 0} class:neg={s.cupDiff < 0}>{sign(s.cupDiff)}</td>
                <td class="muted">{s.buchholz.toFixed(1)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>

</div>

<style>
  .view {
    display: flex;
    height: 100%;
    padding: 2.5rem 3rem;
    overflow: hidden;
    gap: 3rem;
    align-items: flex-start;
  }

  .main-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    min-width: 0;
    overflow-y: auto;
  }

  h1 {
    margin: 0;
    font-size: 2.5rem;
    font-weight: 900;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    text-align: center;
    opacity: 0.9;
    flex-shrink: 0;
  }

  h2 {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    opacity: 0.45;
  }

  /* ── Podium ── */
  .podium {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    gap: 1.5rem;
    flex-shrink: 0;
  }

  .place {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
    min-width: 13rem;
    background: #14172040;
    border: 2px solid #2a2d36;
    border-radius: 14px;
    padding: 1.5rem 1.25rem 1.25rem;
  }

  .place.first {
    padding-top: 2.25rem;
    border-color: #d97706;
    background: #1c1a10;
  }

  .medal-ring {
    width: 3.5rem;
    height: 3.5rem;
    border-radius: 50%;
    border: 3px solid #6b7280;
    background: #374151;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: 900;
    color: #d1d5db;
  }

  .medal-ring.gold {
    border-color: #d97706;
    background: #451a03;
    color: #fbbf24;
    width: 4rem;
    height: 4rem;
    font-size: 1.75rem;
  }

  .medal-ring.bronze {
    border-color: #92400e;
    background: #1c0f00;
    color: #d97706;
  }

  .place-name {
    font-size: 1.5rem;
    font-weight: 800;
    text-align: center;
    line-height: 1.2;
  }

  .place.first .place-name {
    font-size: 1.75rem;
    color: #fbbf24;
  }

  .place-label {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    opacity: 0.4;
  }

  /* ── Standings table ── */
  .scoreboard { overflow-x: auto; }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 1.05rem;
  }

  th {
    text-align: right;
    font-weight: 600;
    opacity: 0.4;
    padding: 0.3rem 0.75rem 0.5rem;
    font-size: 0.8rem;
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
  td.name { text-align: left; font-weight: 700; font-size: 1.1rem; }
  td.muted { opacity: 0.5; }
  td.pos { color: #4ade80; }
  td.neg { color: #f87171; }

  tr:nth-child(-n+3) td { background: #ffffff06; }
  tr:nth-child(1) td.name { color: #fbbf24; }
  tr:nth-child(2) td.name { color: #d1d5db; }
  tr:nth-child(3) td.name { color: #d97706; }

  /* Light mode */
  :global(body.light) h1,
  :global(body.light) h2 { color: #111827; }

  :global(body.light) .place {
    background: #ffffff;
    border-color: #d1d5db;
  }

  :global(body.light) .place.first {
    background: #fffbeb;
    border-color: #d97706;
  }

  :global(body.light) .medal-ring {
    border-color: #9ca3af;
    background: #f3f4f6;
    color: #374151;
  }

  :global(body.light) .medal-ring.gold {
    border-color: #d97706;
    background: #fef3c7;
    color: #92400e;
  }

  :global(body.light) .medal-ring.bronze {
    border-color: #92400e;
    background: #fef3c7;
    color: #92400e;
  }

  :global(body.light) .place-name { color: #111827; }
  :global(body.light) .place.first .place-name { color: #d97706; }

  :global(body.light) td { border-top-color: #e5e7eb; }
  :global(body.light) tr:nth-child(-n+3) td { background: rgba(0, 0, 0, 0.03); }
  :global(body.light) tr:nth-child(1) td.name { color: #d97706; }
  :global(body.light) tr:nth-child(2) td.name { color: #374151; }
  :global(body.light) tr:nth-child(3) td.name { color: #92400e; }
  :global(body.light) td.pos { color: #16a34a; }
  :global(body.light) td.neg { color: #dc2626; }
</style>
