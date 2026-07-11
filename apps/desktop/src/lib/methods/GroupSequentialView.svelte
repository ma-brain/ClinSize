<script lang="ts">
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import { persistCalculation } from "$lib/workflow/record";
  import type {
    GroupSequentialInput,
    GroupSequentialResult,
    SpendingFunction,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let alpha = $state("0.05");
  let targetPower = $state("0.8");
  let numberOfLooks = $state("3");
  let spendingFunction = $state<SpendingFunction>("obrien_fleming");

  let result = $state<GroupSequentialResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);

  function buildInput(): GroupSequentialInput {
    return {
      alpha: Number(alpha),
      targetPower: Number(targetPower),
      numberOfLooks: Number(numberOfLooks),
      spendingFunction,
    };
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await invoke<GroupSequentialResult>("calculate_group_sequential", { input });
      exportMarkdown = await invoke<string>("export_group_sequential_markdown", {
        input,
        result,
      });
      persistCalculation({
        methodId: "design.group_sequential",
        methodName: "Group sequential design",
        input,
        result,
      });
    } catch (error) {
      result = null;
      exportMarkdown = null;
      errorMessage = String(error);
    } finally {
      calculating = false;
    }
  }
</script>

<div class="method-page">
  <header class="page-header">
    <h2>Group sequential design</h2>
    <p>
      Plan interim efficacy boundaries and sample size inflation for equally
      spaced looks using Lan-DeMets alpha spending.
    </p>
  </header>

  <div class="panels">
    <section class="panel">
      <h3>Parameters</h3>

      <label>
        Two-sided alpha
        <input type="number" min="0" max="1" step="0.001" bind:value={alpha} />
      </label>

      <label>
        Target power
        <input type="number" min="0" max="1" step="0.01" bind:value={targetPower} />
      </label>

      <label>
        Number of looks
        <input type="number" min="2" max="10" step="1" bind:value={numberOfLooks} />
      </label>

      <label>
        Spending function
        <select bind:value={spendingFunction}>
          <option value="obrien_fleming">O'Brien-Fleming</option>
          <option value="pocock">Pocock</option>
        </select>
      </label>

      <button onclick={calculate} disabled={calculating}>
        {calculating ? "Calculating…" : "Calculate"}
      </button>

      {#if errorMessage}
        <p class="error">{errorMessage}</p>
      {/if}
    </section>

    <section class="panel">
      <h3>Results</h3>

      {#if result}
        <dl class="results">
          <dt>Sample size inflation factor</dt>
          <dd>{result.sampleSizeInflationFactor.toFixed(4)}</dd>
          <dt>Achieved power</dt>
          <dd>{result.achievedPower.toFixed(4)}</dd>
          <dt>Fixed-design drift</dt>
          <dd>{result.fixedDesignDrift.toFixed(4)}</dd>
        </dl>

        <table class="looks">
          <thead>
            <tr>
              <th>Look</th>
              <th>Info %</th>
              <th>Upper Z</th>
              <th>Cum. α</th>
            </tr>
          </thead>
          <tbody>
            {#each result.looks as look}
              <tr>
                <td>{look.look}</td>
                <td>{(look.informationFraction * 100).toFixed(0)}</td>
                <td>{look.upperZBoundary.toFixed(3)}</td>
                <td>{look.cumulativeAlphaSpent.toFixed(4)}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <p class="hint">
          Multiply the fixed-design sample size by the inflation factor to
          obtain the maximum sample size under this group sequential plan.
        </p>

        {#if result.warnings.length > 0}
          <div class="warnings">
            <h4>Warnings</h4>
            <ul>
              {#each result.warnings as warning}
                <li><strong>{warning.code}:</strong> {warning.message}</li>
              {/each}
            </ul>
          </div>
        {/if}

        <ExportMenu
          title="Group sequential design"
          markdown={exportMarkdown}
          disabled={calculating}
        />
      {:else}
        <p class="muted">Run a calculation to see interim boundaries.</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .method-page {
    padding: 1.5rem;
  }

  .page-header {
    margin-bottom: 1.25rem;
  }

  .page-header h2 {
    margin: 0 0 0.35rem;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .page-header p {
    margin: 0;
    color: var(--muted);
    font-size: 0.875rem;
    line-height: 1.5;
    max-width: 42rem;
  }

  .panels {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));
    gap: 1rem;
  }

  .panel {
    padding: 1.25rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
  }

  .panel h3 {
    margin: 0 0 1rem;
    font-size: 0.9375rem;
    font-weight: 600;
  }

  label {
    display: grid;
    gap: 0.35rem;
    margin-bottom: 0.85rem;
    font-size: 0.8125rem;
    font-weight: 500;
  }

  input,
  select {
    padding: 0.45rem 0.55rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    font: inherit;
  }

  button {
    margin-top: 0.25rem;
    padding: 0.5rem 0.85rem;
    border: none;
    border-radius: 4px;
    background: var(--accent);
    color: #fff;
    font: inherit;
    font-weight: 500;
    cursor: pointer;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .results {
    margin: 0 0 1rem;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.35rem 1rem;
    font-size: 0.875rem;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    font-weight: 500;
  }

  .looks {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }

  .looks th,
  .looks td {
    padding: 0.35rem 0.5rem;
    border-bottom: 1px solid var(--border);
    text-align: left;
  }

  .looks th {
    color: var(--muted);
    font-weight: 500;
  }

  .hint {
    margin: 1rem 0 0;
    font-size: 0.8125rem;
    color: var(--muted);
    line-height: 1.5;
  }

  .warnings {
    margin-top: 1rem;
    font-size: 0.8125rem;
  }

  .warnings h4 {
    margin: 0 0 0.5rem;
    font-size: 0.8125rem;
  }

  .warnings ul {
    margin: 0;
    padding-left: 1.1rem;
    color: var(--muted);
  }

  .error {
    margin: 0.75rem 0 0;
    color: #b42318;
    font-size: 0.8125rem;
  }

  .muted {
    margin: 0;
    color: var(--muted);
    font-size: 0.875rem;
  }
</style>
