<script lang="ts">
  import {
    createScenario,
    projectState,
    removeScenario,
  } from "$lib/stores/project.svelte";

  let scenarioName = $state("Scenario A");
  let selectedIds = $state<string[]>([]);
  let message = $state<string | null>(null);

  function toggleSelection(id: string) {
    if (selectedIds.includes(id)) {
      selectedIds = selectedIds.filter((entry) => entry !== id);
    } else {
      selectedIds = [...selectedIds, id];
    }
  }

  function addScenario() {
    if (selectedIds.length === 0) {
      message = "Select at least one calculation.";
      return;
    }
    createScenario(scenarioName.trim() || "Scenario", selectedIds);
    selectedIds = [];
    message = "Scenario added.";
  }
</script>

<div class="scenarios-page">
  <header class="page-header">
    <h2>Scenario comparison</h2>
    <p>Group saved calculations and compare primary outputs side by side.</p>
  </header>

  <section class="panel">
    <h3>Create scenario</h3>
    <label>
      Scenario name
      <input bind:value={scenarioName} />
    </label>

    {#if projectState.project.calculations.length === 0}
      <p class="muted">Save calculations from a method page first.</p>
    {:else}
      <div class="choices">
        {#each projectState.project.calculations as entry}
          <label class="choice">
            <input
              type="checkbox"
              checked={selectedIds.includes(entry.id)}
              onchange={() => toggleSelection(entry.id)}
            />
            <span>{entry.label || entry.methodName}</span>
            <span class="detail">{entry.summary.primaryLabel}: {entry.summary.primaryValue}</span>
          </label>
        {/each}
      </div>
      <button onclick={addScenario}>Add scenario</button>
    {/if}
    {#if message}
      <p class="meta">{message}</p>
    {/if}
  </section>

  <section class="panel">
    <h3>Comparison table</h3>
    {#if projectState.project.scenarios.length === 0}
      <p class="muted">No scenarios yet.</p>
    {:else}
      {#each projectState.project.scenarios as scenario}
        <div class="scenario-block">
          <div class="scenario-header">
            <h4>{scenario.name}</h4>
            <button class="link" onclick={() => removeScenario(scenario.id)}>Remove</button>
          </div>
          <table>
            <thead>
              <tr>
                <th>Calculation</th>
                <th>Method</th>
                <th>Primary</th>
                <th>Secondary</th>
              </tr>
            </thead>
            <tbody>
              {#each scenario.calculationIds as calcId}
                {@const entry = projectState.project.calculations.find((item) => item.id === calcId)}
                {#if entry}
                  <tr>
                    <td>{entry.label || entry.methodName}</td>
                    <td>{entry.methodName}</td>
                    <td>{entry.summary.primaryLabel}: {entry.summary.primaryValue}</td>
                    <td>
                      {#if entry.summary.secondaryLabel}
                        {entry.summary.secondaryLabel}: {entry.summary.secondaryValue}
                      {:else}
                        —
                      {/if}
                    </td>
                  </tr>
                {/if}
              {/each}
            </tbody>
          </table>
        </div>
      {/each}
    {/if}
  </section>
</div>

<style>
  .scenarios-page {
    padding: 1.5rem;
  }

  .page-header h2 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .page-header p {
    margin: 0.35rem 0 0;
    color: var(--muted);
    font-size: 0.875rem;
  }

  .panel {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
    padding: 1rem;
    margin-top: 1rem;
  }

  h3,
  h4 {
    margin: 0 0 0.75rem;
    font-size: 0.9375rem;
  }

  label {
    display: grid;
    gap: 0.25rem;
    font-size: 0.8125rem;
    margin-bottom: 0.75rem;
  }

  input[type="text"],
  input:not([type="checkbox"]) {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    background: var(--background);
  }

  .choices {
    display: grid;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .choice {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.35rem 0.5rem;
    align-items: center;
  }

  .detail {
    grid-column: 2;
    color: var(--muted);
    font-size: 0.75rem;
  }

  button {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.45rem 0.75rem;
    background: var(--background);
    cursor: pointer;
    font-size: 0.875rem;
  }

  button.link {
    border: none;
    padding: 0;
    color: var(--accent);
    background: transparent;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }

  th,
  td {
    border-bottom: 1px solid var(--border);
    padding: 0.5rem 0.35rem;
    text-align: left;
  }

  .scenario-block + .scenario-block {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .scenario-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }

  .meta,
  .muted {
    color: var(--muted);
    font-size: 0.8125rem;
  }
</style>
