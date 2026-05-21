<script lang="ts">
  import { onMount } from 'svelte';
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import GroupPhaseView from '$lib/components/projector/GroupPhaseView.svelte';
  import PlayoffsView from '$lib/components/projector/PlayoffsView.svelte';
  import { toggleFullscreenProjector } from '$lib/ipc/commands';

  const t = $derived(tournamentStore.value);
  const phase = $derived(t?.phase);

  let scale = $state(1.0);

  onMount(() => {
    const saved = localStorage.getItem('projector-scale');
    if (saved) scale = Math.max(0.4, Math.min(2.0, parseFloat(saved) || 1.0));
  });

  function changeScale(delta: number) {
    scale = parseFloat(Math.max(0.4, Math.min(2.0, scale + delta)).toFixed(1));
    localStorage.setItem('projector-scale', String(scale));
  }

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'F11') {
      e.preventDefault();
      toggleFullscreenProjector().catch(() => {});
    }
  }

  const scaleW = $derived((100 / scale).toFixed(2) + '%');
  const scaleH = $derived((100 / scale).toFixed(2) + 'vh');
</script>

<svelte:window onkeydown={handleKey} />

<div class="projector">
  <!-- Scaled content -->
  <div class="content-wrap" style="transform: scale({scale}); transform-origin: top left; width: {scaleW}; height: {scaleH};">
    {#if !t || phase === 'setup'}
      <div class="idle">
        <div class="logo">🏆</div>
        <p>Waiting for tournament…</p>
      </div>

    {:else if phase === 'group'}
      <GroupPhaseView />

    {:else if phase === 'playoffs' && t.playoffs}
      <PlayoffsView />

    {:else if phase === 'finished'}
      <div class="finished">
        <h1>Tournament Complete!</h1>
        <p class="sub">Thanks for playing</p>
      </div>
    {/if}
  </div>

  <!-- Scale controls (not scaled) -->
  <div class="scale-controls">
    <button onclick={() => changeScale(-0.1)} title="Zoom out" disabled={scale <= 0.4}>−</button>
    <span class="scale-val">{Math.round(scale * 100)}%</span>
    <button onclick={() => changeScale(0.1)} title="Zoom in" disabled={scale >= 2.0}>+</button>
  </div>

  <div class="hint">Press F11 to toggle fullscreen</div>
</div>

<style>
  .projector {
    position: relative;
    height: 100vh;
    overflow: hidden;
  }

  .content-wrap {
    /* dimensions are set inline; transform-origin: top left in inline style */
  }

  .idle, .finished {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 1rem;
    text-align: center;
  }

  .logo { font-size: 5rem; }

  .idle p {
    opacity: 0.3;
    font-size: 1.5rem;
  }

  .finished h1 {
    margin: 0;
    font-size: 4rem;
    font-weight: 900;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .finished .sub { opacity: 0.4; font-size: 1.25rem; margin: 0; }

  /* Scale controls */
  .scale-controls {
    position: fixed;
    bottom: 0.75rem;
    left: 1rem;
    display: flex;
    align-items: center;
    gap: 0.3rem;
    opacity: 0.15;
    transition: opacity 0.2s;
    z-index: 100;
  }

  .scale-controls:hover { opacity: 1; }

  .scale-controls button {
    background: #1a1c23;
    border: 1px solid #2a2d36;
    border-radius: 4px;
    color: #e9ecef;
    cursor: pointer;
    font-size: 1rem;
    font-weight: 700;
    line-height: 1;
    padding: 0.15rem 0.55rem;
    transition: background 0.12s;
  }

  .scale-controls button:hover:not(:disabled) { background: #2a2d36; }
  .scale-controls button:disabled { opacity: 0.3; cursor: not-allowed; }

  .scale-val {
    font-size: 0.7rem;
    color: #e9ecef;
    min-width: 2.75rem;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .hint {
    position: fixed;
    bottom: 0.75rem;
    right: 1rem;
    font-size: 0.7rem;
    opacity: 0.15;
  }

  /* Light mode: make scale controls visible against light bg */
  :global(body.light) .scale-controls button {
    background: #e8eaed;
    border-color: #d1d5db;
    color: #111827;
  }

  :global(body.light) .scale-controls button:hover:not(:disabled) {
    background: #d1d5db;
  }

  :global(body.light) .scale-val { color: #374151; }
  :global(body.light) .hint { color: #374151; }
</style>
