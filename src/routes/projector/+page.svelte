<script lang="ts">
  import { tournamentStore } from '$lib/stores/tournament.svelte';
  import GroupPhaseView from '$lib/components/projector/GroupPhaseView.svelte';
  import PlayoffsView from '$lib/components/projector/PlayoffsView.svelte';
  import { toggleFullscreenProjector } from '$lib/ipc/commands';

  const t = $derived(tournamentStore.value);
  const phase = $derived(t?.phase);

  function handleKey(e: KeyboardEvent) {
    if (e.key === 'F11') {
      e.preventDefault();
      toggleFullscreenProjector().catch(() => {});
    }
  }
</script>

<svelte:window onkeydown={handleKey} />

<div class="projector">
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

  <!-- Fullscreen hint -->
  <div class="hint">Press F11 to toggle fullscreen</div>
</div>

<style>
  .projector {
    position: relative;
    min-height: 100vh;
  }

  .idle, .finished {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
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

  .hint {
    position: fixed;
    bottom: 0.75rem;
    right: 1rem;
    font-size: 0.7rem;
    opacity: 0.15;
  }
</style>
