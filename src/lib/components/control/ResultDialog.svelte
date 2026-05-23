<script lang="ts">
  import type { Match, Tournament } from '$lib/ipc/types';

  let {
    match,
    tournament,
    onSubmit,
    onClose,
  }: {
    match: Match;
    tournament: Tournament;
    onSubmit: (cupsA: number, cupsB: number) => Promise<void>;
    onClose: () => void;
  } = $props();

  const teamA = $derived(tournament.teams.find((t) => t.id === match.teamA)?.name ?? match.teamA);
  const teamB = $derived(tournament.teams.find((t) => t.id === match.teamB)?.name ?? match.teamB);

  // Intentional snapshot: local editable state initialized from prop, not kept reactive
  // svelte-ignore state_referenced_locally
  let cupsA = $state(match.cupsA);
  // svelte-ignore state_referenced_locally
  let cupsB = $state(match.cupsB);
  let submitting = $state(false);
  let error = $state('');

  async function handleSubmit() {
    error = '';
    submitting = true;
    try {
      await onSubmit(cupsA, cupsB);
      onClose();
    } catch (e) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }

  function onBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="backdrop" role="dialog" tabindex="-1" onclick={onBackdrop} onkeydown={() => {}} aria-modal="true">
  <div class="dialog">
    <h2>Submit Result</h2>

    <p class="hint">Enter cups remaining in each team's rack. More cups wins. Equal cups = draw.</p>

    <div class="matchup">
      <div class="team">
        <span class="team-name">{teamA}</span>
        <input
          type="number"
          bind:value={cupsA}
          min="0"
          max="10"
          class="cups"
          class:is-winner={cupsA > cupsB}
          aria-label="Cups remaining for {teamA}"
        />
        <span class="cups-label">cups left</span>
      </div>
      <span class="vs">vs</span>
      <div class="team right">
        <span class="cups-label">cups left</span>
        <input
          type="number"
          bind:value={cupsB}
          min="0"
          max="10"
          class="cups"
          class:is-winner={cupsB > cupsA}
          aria-label="Cups remaining for {teamB}"
        />
        <span class="team-name">{teamB}</span>
      </div>
    </div>

    {#if cupsA === cupsB}
      <p class="draw">Draw</p>
    {:else}
      <p class="winner">
        Winner: <strong>{cupsA > cupsB ? teamA : teamB}</strong>
      </p>
    {/if}

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <div class="actions">
      <button onclick={onClose} disabled={submitting}>Cancel</button>
      <button class="primary" onclick={handleSubmit} disabled={submitting}>
        {submitting ? 'Saving…' : 'Confirm'}
      </button>
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: #00000080;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: var(--bg-raised, #1e2028);
    border: 1px solid var(--border, #3a3d4a);
    border-radius: 12px;
    padding: 1.75rem 2rem;
    width: 26rem;
    max-width: 90vw;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  h2 { margin: 0; font-size: 1.1rem; }

  .hint {
    margin: -0.25rem 0 0;
    font-size: 0.78rem;
    color: var(--text-muted, #7a7f8e);
    text-align: center;
  }

  .matchup {
    display: flex;
    align-items: center;
    gap: 1rem;
    justify-content: center;
  }

  .team {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.35rem;
    flex: 1;
  }

  .team.right { flex-direction: column-reverse; }

  .team-name {
    font-size: 0.9rem;
    font-weight: 600;
    text-align: center;
    color: var(--text-primary, #f1f3f7);
  }

  .cups-label {
    font-size: 0.68rem;
    color: var(--text-muted, #7a7f8e);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .cups {
    width: 5rem;
    text-align: center;
    font-size: 2rem;
    font-weight: 700;
    background: var(--bg-input, #32363f);
    border: 2px solid var(--border, #3a3d4a);
    border-radius: 8px;
    color: var(--text-primary, #f1f3f7);
    padding: 0.3rem;
  }

  .cups.is-winner { border-color: #4ade80; color: #4ade80; }
  .cups:focus { outline: none; border-color: var(--border-focus, #2563eb); }

  .vs {
    opacity: 0.4;
    font-size: 0.9rem;
    flex-shrink: 0;
  }

  .draw {
    text-align: center;
    margin: 0;
    font-size: 0.9rem;
    color: #fbbf24;
    font-weight: 600;
  }

  .winner {
    text-align: center;
    margin: 0;
    font-size: 0.9rem;
    color: #4ade80;
  }

  .error {
    margin: 0;
    color: #f87171;
    font-size: 0.85rem;
    text-align: center;
  }

  .actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
  }

  button {
    background: var(--bg-elevated, #2a2d36);
    border: 1px solid var(--border, #3a3d4a);
    border-radius: 6px;
    color: var(--text-primary, #e9ecef);
    cursor: pointer;
    font-size: 0.875rem;
    padding: 0.45rem 1.1rem;
  }

  button:disabled { opacity: 0.4; cursor: not-allowed; }
  button.primary { background: #2563eb; border-color: #1d4ed8; font-weight: 600; }
  button.primary:not(:disabled):hover { background: #1d4ed8; }

  /* Light mode */
  :global(body.light) .backdrop { background: #00000050; }
  :global(body.light) .dialog {
    background: #ffffff;
    border-color: #d1d5db;
  }
  :global(body.light) h2 { color: #111827; }
  :global(body.light) .hint { color: #6b7280; }
  :global(body.light) .team-name { color: #111827; }
  :global(body.light) .cups-label { color: #6b7280; }
  :global(body.light) .cups {
    background: #f3f4f6;
    border-color: #d1d5db;
    color: #111827;
  }
  :global(body.light) .cups.is-winner { border-color: #16a34a; color: #16a34a; }
  :global(body.light) .vs { color: #374151; opacity: 0.6; }
  :global(body.light) .winner { color: #16a34a; }
  :global(body.light) button {
    background: #f3f4f6;
    border-color: #d1d5db;
    color: #111827;
  }
  :global(body.light) button.primary { background: #2563eb; border-color: #1d4ed8; color: #fff; }
</style>
