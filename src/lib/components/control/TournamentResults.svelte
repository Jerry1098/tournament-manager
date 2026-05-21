<script lang="ts">
  import { getStandings } from '$lib/ipc/commands';
  import { standingsStore, tournamentStore } from '$lib/stores/tournament.svelte';
  import PlayoffBracket from './PlayoffBracket.svelte';

  const t = $derived(tournamentStore.value!);

  $effect(() => {
    if (t) {
      getStandings().then((s) => { standingsStore.value = s; }).catch(() => {});
    }
  });

  const standings = $derived(standingsStore.value);

  let showBracket = $state(false);

  function teamName(id: string) {
    if (!id) return '?';
    return t.teams.find((x) => x.id === id)?.name ?? id;
  }

  const finalMatch = $derived(t.playoffs?.matches[0]);
  const thirdMatch = $derived(t.playoffs?.thirdPlaceMatch);

  // Fewer cups remaining = that team won (they cleared opponent's rack)
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

<div class="results-wrap">
  {#if showBracket}
    <div class="bracket-section">
      <button class="back-btn" onclick={() => { showBracket = false; }}>
        ← Back to Results
      </button>
      <PlayoffBracket />
    </div>
  {:else}
    <div class="results-section">
      <h2>Tournament Complete</h2>

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

      <div class="scoreboard">
        <h3>Final Scoreboard</h3>
        {#if standings.length > 0}
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
                  <td class="num">{s.wins}</td>
                  <td class="num">{s.losses}</td>
                  <td class="num" class:pos={s.cupDiff > 0} class:neg={s.cupDiff < 0}>{sign(s.cupDiff)}</td>
                  <td class="num muted">{s.buchholz.toFixed(1)}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {:else}
          <p class="empty">Loading standings…</p>
        {/if}
      </div>

      <button class="bracket-btn" onclick={() => { showBracket = true; }}>View Bracket</button>
    </div>
  {/if}
</div>

<style>
  .results-wrap {
    max-width: 52rem;
    margin: 0 auto;
  }

  .results-section {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  h2 {
    margin: 0;
    font-size: 1.5rem;
    text-align: center;
  }

  h3 {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.6;
  }

  /* Podium */
  .podium {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    gap: 1rem;
  }

  .place {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    min-width: 10rem;
    background: #22252e;
    border: 1px solid #3a3d4a;
    border-radius: 10px;
    padding: 1.25rem 1rem 1rem;
  }

  .place.first {
    padding-top: 1.75rem;
    border-color: #92400e;
    background: #1c1a10;
  }

  .medal-ring {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 50%;
    border: 3px solid #6b7280;
    background: #374151;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.1rem;
    font-weight: 800;
    color: #d1d5db;
  }

  .medal-ring.gold {
    border-color: #d97706;
    background: #451a03;
    color: #fbbf24;
  }

  .medal-ring.bronze {
    border-color: #92400e;
    background: #1c0f00;
    color: #d97706;
  }

  .place-name {
    font-size: 1.05rem;
    font-weight: 700;
    text-align: center;
  }

  .place-label {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    opacity: 0.45;
  }

  /* Scoreboard table */
  .scoreboard { overflow-x: auto; }

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

  .empty { opacity: 0.4; font-size: 0.875rem; }

  /* Bracket toggle */
  .bracket-btn {
    align-self: flex-start;
    background: #2a2d36;
    border: 1px solid #3a3d4a;
    border-radius: 6px;
    color: #e9ecef;
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0.45rem 1.1rem;
  }

  .bracket-btn:hover { background: #32363f; }

  .bracket-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .back-btn {
    background: transparent;
    border: none;
    color: #60a5fa;
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0;
    align-self: flex-start;
  }

  .back-btn:hover { text-decoration: underline; }

  /* Light mode */
  :global(body.light) h2,
  :global(body.light) h3 { color: #111827; }

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

  :global(body.light) td { border-top-color: #e5e7eb; }
  :global(body.light) tr:hover td { background: rgba(0, 0, 0, 0.04); }
  :global(body.light) td.pos { color: #16a34a; }
  :global(body.light) td.neg { color: #dc2626; }

  :global(body.light) .bracket-btn {
    background: #f3f4f6;
    border-color: #d1d5db;
    color: #111827;
  }

  :global(body.light) .bracket-btn:hover { background: #e5e7eb; }
  :global(body.light) .back-btn { color: #2563eb; }
</style>
