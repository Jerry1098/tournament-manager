<script lang="ts">
  import { addTeam, removeTeam, renameTeam } from '$lib/ipc/commands';
  import { tournamentStore } from '$lib/stores/tournament.svelte';

  let newName = $state('');
  let editingId = $state<string | null>(null);
  let editName = $state('');

  const teams = $derived(tournamentStore.value?.teams ?? []);
  const isSetup = $derived(tournamentStore.value?.phase === 'setup');

  async function handleAdd() {
    if (!newName.trim()) return;
    await addTeam(newName.trim()).catch(console.error);
    newName = '';
  }

  async function startEdit(id: string, name: string) {
    editingId = id;
    editName = name;
  }

  async function commitEdit() {
    if (editingId && editName.trim()) {
      await renameTeam(editingId, editName.trim()).catch(console.error);
    }
    editingId = null;
  }
</script>

<section class="team-list">
  <h3>Teams <span class="count">{teams.length}</span></h3>

  {#if isSetup}
    <form class="add-row" onsubmit={(e) => { e.preventDefault(); handleAdd(); }}>
      <input bind:value={newName} placeholder="Team name…" />
      <button type="submit" disabled={!newName.trim()}>Add</button>
    </form>
  {/if}

  <ul>
    {#each teams as team (team.id)}
      <li>
        {#if editingId === team.id}
          <input
            class="edit-input"
            bind:value={editName}
            onblur={commitEdit}
            onkeydown={(e) => e.key === 'Enter' && commitEdit()}
            autofocus
          />
        {:else}
          <span class="team-name">{team.name}</span>
        {/if}

        {#if isSetup}
          <div class="actions">
            <button class="icon" onclick={() => startEdit(team.id, team.name)}>✎</button>
            <button class="icon danger" onclick={() => removeTeam(team.id).catch(console.error)}>✕</button>
          </div>
        {/if}
      </li>
    {/each}

    {#if teams.length === 0}
      <li class="empty">No teams yet.</li>
    {/if}
  </ul>
</section>

<style>
  .team-list {
    min-width: 14rem;
  }

  h3 {
    margin: 0 0 0.75rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    opacity: 0.6;
  }

  .count {
    background: #2a2d36;
    border-radius: 10px;
    font-size: 0.75rem;
    padding: 0 0.4rem;
  }

  .add-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .add-row input { flex: 1; }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #2a2d36;
    border-radius: 6px;
    padding: 0.45rem 0.75rem;
    gap: 0.5rem;
  }

  li.empty {
    opacity: 0.4;
    font-size: 0.875rem;
    justify-content: center;
    background: transparent;
  }

  .team-name { flex: 1; font-size: 0.95rem; }

  .edit-input {
    flex: 1;
    background: #13151a;
    border: 1px solid #4a5568;
    border-radius: 4px;
    color: #e9ecef;
    font-size: 0.95rem;
    padding: 0.15rem 0.4rem;
  }

  .actions { display: flex; gap: 0.25rem; }

  input {
    background: #2a2d36;
    border: 1px solid #3a3d4a;
    border-radius: 6px;
    color: #e9ecef;
    font-size: 0.875rem;
    padding: 0.4rem 0.65rem;
  }

  button {
    background: #363a48;
    border: 1px solid #3a3d4a;
    border-radius: 6px;
    color: #e9ecef;
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0.3rem 0.7rem;
  }

  button.icon { padding: 0.25rem 0.5rem; }
  button.danger { color: #f87171; }
  button:disabled { opacity: 0.4; cursor: not-allowed; }

  /* Light mode */
  :global(body.light) .count { background: #e8eaed; color: #374151; }
  :global(body.light) li { background: #e8eaed; }
  :global(body.light) li.empty { background: transparent; }
  :global(body.light) .add-row input {
    background: var(--bg-input);
    border-color: var(--border);
    color: var(--text-primary);
  }
  :global(body.light) .edit-input {
    background: var(--bg-input);
    border-color: var(--border-focus);
    color: var(--text-primary);
  }
  :global(body.light) .add-row button,
  :global(body.light) .actions button {
    background: var(--bg-elevated);
    border-color: var(--border);
    color: var(--text-primary);
  }
  :global(body.light) .actions button.danger { color: #dc2626; }
</style>
