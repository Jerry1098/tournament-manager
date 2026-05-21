<script lang="ts">
  import { standingsStore } from '$lib/stores/tournament.svelte';
  import { onMount } from 'svelte';

  const standings = $derived(standingsStore.value);

  let containerEl: HTMLDivElement;
  let tableEl: HTMLTableElement;
  let offset = $state(0);

  onMount(() => {
    const SPEED = 0.35;   // px per frame
    const PAUSE_MS = 2200;
    let dir = 1;
    let animId: number;
    let pauseTimeout: ReturnType<typeof setTimeout> | null = null;
    let pausing = false;

    function frame() {
      if (!containerEl || !tableEl) { animId = requestAnimationFrame(frame); return; }

      const maxOffset = tableEl.clientHeight - containerEl.clientHeight;

      if (maxOffset <= 8) {
        offset = 0;
        animId = requestAnimationFrame(frame);
        return;
      }

      if (!pausing) {
        offset = Math.max(0, Math.min(maxOffset, offset + dir * SPEED));

        if (offset >= maxOffset && dir > 0) {
          pausing = true;
          pauseTimeout = setTimeout(() => { dir = -1; pausing = false; }, PAUSE_MS);
        } else if (offset <= 0 && dir < 0) {
          pausing = true;
          pauseTimeout = setTimeout(() => { dir = 1; pausing = false; }, PAUSE_MS);
        }
      }

      animId = requestAnimationFrame(frame);
    }

    animId = requestAnimationFrame(frame);

    return () => {
      cancelAnimationFrame(animId);
      if (pauseTimeout) clearTimeout(pauseTimeout);
    };
  });

  function sign(n: number) { return n > 0 ? `+${n}` : `${n}`; }
</script>

<section class="scoreboard">
  <h2>Standings</h2>
  {#if standings.length === 0}
    <p class="empty">No results yet</p>
  {:else}
    <div class="scroll-container" bind:this={containerEl}>
      <table bind:this={tableEl} style="transform: translateY(-{offset}px);">
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
    </div>
  {/if}
</section>

<style>
  .scoreboard {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  h2 {
    margin: 0 0 0.6rem;
    font-size: 1.25rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    opacity: 0.7;
    flex-shrink: 0;
  }

  .empty { opacity: 0.4; font-size: 1.25rem; margin: 0; }

  .scroll-container {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    position: relative;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 1.1rem;
    will-change: transform;
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
  :global(body.light) td.pos { color: #16a34a; }
  :global(body.light) td.neg { color: #dc2626; }
</style>
