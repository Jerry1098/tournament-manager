<script lang="ts">
  import { createTournament } from '$lib/ipc/commands';
  import type { TableConfig, TournamentConfig } from '$lib/ipc/types';

  let { onCreated }: { onCreated: () => void } = $props();

  let name = $state('');
  // "Matches per team" = number of Swiss pairing rounds (in Swiss, each round = 1 match per team)
  let matchesPerTeam = $state(3);
  let maxRoundExtension = $state(3);
  let expectedTeams = $state(16);
  let playoffTeamCount = $state(4);
  let defaultMatchMinutes = $state(15);
  let tables = $state<Array<{ name: string; category: string }>>([
    { name: 'Table 1', category: 'indoor' },
    { name: 'Table 2', category: 'indoor' },
    { name: 'Table 3', category: 'indoor' },
    { name: 'Table 4', category: 'indoor' },
    { name: 'Table 5', category: 'outdoor' },
    { name: 'Table 6', category: 'outdoor' },
  ]);

  // Scheduling rounds = how many time-slots are needed to run all matches
  // formula: ceil(teams * matchesPerTeam / 2 / nTables)
  const schedulingRounds = $derived(
    tables.length > 0
      ? Math.ceil((expectedTeams * matchesPerTeam) / 2 / tables.length)
      : 0
  );

  function addTable() {
    tables = [...tables, { name: `Table ${tables.length + 1}`, category: 'indoor' }];
  }

  function removeTable(i: number) {
    tables = tables.filter((_, idx) => idx !== i);
  }

  async function handleCreate() {
    if (!name.trim() || tables.length === 0) return;

    const tableConfigs: TableConfig[] = tables.map((t, i) => ({
      id: `temp-${i}`, // Rust regenerates as ULIDs on create
      name: t.name,
      category: t.category,
    }));

    const config: TournamentConfig = {
      format: 'swiss',
      tables: tableConfigs,
      swissRounds: matchesPerTeam,
      maxRoundExtension,
      playoffTeamCount,
      allowByes: true,
      randomSeed: Math.floor(Math.random() * 2 ** 32),
      defaultMatchMinutes,
    };

    await createTournament(name.trim(), config).catch(console.error);
    onCreated();
  }
</script>

<div class="panel">
  <h2>New Tournament</h2>

  <label class="field">
    Tournament name
    <input bind:value={name} placeholder="e.g. Summer Cup 2026" />
  </label>

  <!-- Group phase settings -->
  <fieldset>
    <legend>Group phase</legend>

    <div class="row">
      <label class="field">
        Expected teams
        <input type="number" bind:value={expectedTeams} min="2" max="256" />
        <span class="hint">Used to estimate scheduling rounds</span>
      </label>

      <label class="field">
        Matches per team
        <input type="number" bind:value={matchesPerTeam} min="1" max="20" />
        <span class="hint">Each team plays this many group games</span>
      </label>

      <label class="field">
        Max extra rounds
        <input type="number" bind:value={maxRoundExtension} min="0" max="10" />
        <span class="hint">Extra rounds allowed to achieve equal games per category (0 = never extend)</span>
      </label>
    </div>

    <div class="calc-preview">
      <span class="formula">
        {expectedTeams} teams × {matchesPerTeam} matches ÷ 2 ÷ {tables.length || '?'} tables
      </span>
      <span class="arrow">→</span>
      <span class="result">
        {#if tables.length > 0}
          <strong>{schedulingRounds}</strong> scheduling rounds
        {:else}
          add tables first
        {/if}
      </span>
    </div>
  </fieldset>

  <!-- Playoffs + timing -->
  <fieldset>
    <legend>Playoffs &amp; timing</legend>
    <div class="row">
      <label class="field">
        Playoff teams
        <select bind:value={playoffTeamCount}>
          <option value={4}>Top 4</option>
          <option value={8}>Top 8</option>
          <option value={16}>Top 16</option>
        </select>
      </label>

      <label class="field">
        Match duration (min)
        <input type="number" bind:value={defaultMatchMinutes} min="0" max="120" />
        <span class="hint">0 = no timer</span>
      </label>
    </div>
  </fieldset>

  <!-- Tables -->
  <fieldset>
    <legend>Tables / Locations</legend>
    <div class="table-list">
      {#each tables as tbl, i}
        <div class="table-row">
          <input bind:value={tbl.name} placeholder="Table name" />
          <input bind:value={tbl.category} placeholder="Category (e.g. indoor, outdoor)" class="category" />
          <button class="icon-btn danger" onclick={() => removeTable(i)} aria-label="Remove table">✕</button>
        </div>
      {/each}

      {#if tables.length === 0}
        <p class="empty">Add at least one table to continue.</p>
      {/if}

      <button class="add-btn" onclick={addTable}>+ Add table</button>
    </div>
  </fieldset>

  <button
    class="create-btn"
    onclick={handleCreate}
    disabled={!name.trim() || tables.length === 0}
  >
    Create Tournament
  </button>
</div>

<style>
  .panel {
    max-width: 38rem;
    margin: 1.5rem auto;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  h2 { margin: 0; font-size: 1.25rem; font-weight: 700; }

  fieldset {
    border: 1px solid var(--border, #3a3d4a);
    border-radius: 8px;
    padding: 0.9rem 1rem 1rem;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  legend {
    font-size: 0.78rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--text-muted, #7a7f8e);
    padding: 0 0.4rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: var(--text-secondary, #b8bcc8);
    flex: 1;
  }

  /* Let global app.css handle input/select styling — no overrides here */

  .hint {
    font-size: 0.72rem;
    color: var(--text-muted, #7a7f8e);
    line-height: 1.3;
  }

  .row {
    display: flex;
    gap: 1rem;
    align-items: flex-start;
  }

  /* Scheduling rounds calculator */
  .calc-preview {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    background: #13151a;
    border: 1px solid #2a2d36;
    border-radius: 7px;
    padding: 0.6rem 0.9rem;
    font-size: 0.82rem;
    flex-wrap: wrap;
  }

  .formula { color: var(--text-muted, #7a7f8e); font-variant-numeric: tabular-nums; }
  .arrow   { color: var(--text-muted, #7a7f8e); }
  .result  { color: var(--text-primary, #f1f3f7); }
  .result strong { color: #60a5fa; font-size: 1rem; }

  /* Table list */
  .table-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .table-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .table-row input { flex: 1; }
  .table-row .category { flex: 0.8; }

  .icon-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 5px;
    color: var(--text-muted, #7a7f8e);
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0.3rem 0.5rem;
    flex-shrink: 0;
  }

  .icon-btn.danger { color: var(--red, #f87171); }
  .icon-btn:hover  { background: #2a2d36; }

  .empty {
    color: var(--text-muted, #7a7f8e);
    font-size: 0.85rem;
    text-align: center;
    padding: 0.5rem;
  }

  .add-btn {
    align-self: flex-start;
    background: var(--bg-elevated, #2a2d36);
    border: 1px dashed var(--border, #3a3d4a);
    border-radius: 6px;
    color: var(--text-secondary, #b8bcc8);
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0.3rem 0.75rem;
  }

  .add-btn:hover { border-style: solid; color: var(--text-primary, #f1f3f7); }

  .create-btn {
    background: var(--accent, #2563eb);
    border: none;
    border-radius: 8px;
    color: #fff;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 700;
    padding: 0.65rem 1.5rem;
    align-self: flex-end;
  }

  .create-btn:disabled { opacity: 0.35; cursor: not-allowed; }
  .create-btn:hover:not(:disabled) { background: var(--accent-hov, #1d4ed8); }

  /* Light mode */
  :global(body.light) .calc-preview {
    background: var(--bg-input, #f0f2f5);
    border-color: var(--border, #d1d5db);
  }

  :global(body.light) .icon-btn:hover { background: var(--bg-elevated, #e8eaed); }
</style>
