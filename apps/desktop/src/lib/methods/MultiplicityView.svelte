<script lang="ts">
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import { persistCalculation } from "$lib/workflow/record";
  import type {
    MultiplicityInput,
    MultiplicityMethod,
    MultiplicityResult,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let familyWiseAlpha = $state("0.05");
  let numberOfComparisons = $state("2");
  let adjustmentMethod = $state<MultiplicityMethod>("bonferroni");
  let gatePosition = $state("1");

  let result = $state<MultiplicityResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);
  const comparisonLabel = $derived(
    adjustmentMethod === "dunnett"
      ? "Number of treatment arms (vs control)"
      : "Number of comparisons",
  );
  const showGatePosition = $derived(adjustmentMethod === "holm");

  function buildInput(): MultiplicityInput {
    const input: MultiplicityInput = {
      familyWiseAlpha: Number(familyWiseAlpha),
      numberOfComparisons: Number(numberOfComparisons),
      adjustmentMethod,
    };

    if (adjustmentMethod === "holm") {
      input.gatePosition = Number(gatePosition);
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await invoke<MultiplicityResult>("calculate_multiplicity", { input });
      exportMarkdown = await invoke<string>("export_multiplicity_markdown", {
        input,
        result,
      });
      persistCalculation({
        methodId: "design.multiplicity",
        methodName: "Multiplicity adjustment",
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
    <h2>Multiplicity adjustment</h2>
    <p>
      Convert a family-wise Type I error rate into a per-comparison alpha for
      use in endpoint sample size calculations.
    </p>
  </header>

  <div class="panels">
    <section class="panel">
      <h3>Parameters</h3>

      <label>
        Family-wise alpha
        <input type="number" min="0" max="1" step="0.001" bind:value={familyWiseAlpha} />
      </label>

      <label>
        {comparisonLabel}
        <input type="number" min="1" step="1" bind:value={numberOfComparisons} />
      </label>

      <label>
        Adjustment method
        <select bind:value={adjustmentMethod}>
          <option value="bonferroni">Bonferroni</option>
          <option value="sidak">Šidák (Sidak)</option>
          <option value="dunnett">Dunnett (arms vs control)</option>
          <option value="holm">Holm gatekeeping</option>
        </select>
      </label>

      {#if showGatePosition}
        <label>
          Gate position (1 = first hypothesis)
          <input type="number" min="1" step="1" bind:value={gatePosition} />
        </label>
      {/if}

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
          <dt>Adjusted per-comparison alpha</dt>
          <dd>{result.adjustedAlpha.toFixed(6)}</dd>
          <dt>Family-wise alpha</dt>
          <dd>{result.familyWiseAlpha.toFixed(4)}</dd>
          <dt>Number of comparisons</dt>
          <dd>{result.numberOfComparisons}</dd>
          {#if result.gatePosition}
            <dt>Gate position</dt>
            <dd>{result.gatePosition}</dd>
          {/if}
          <dt>Alpha reduction factor</dt>
          <dd>{result.alphaReductionFactor.toFixed(4)}</dd>
        </dl>

        <p class="hint">
          Use the adjusted per-comparison alpha as the Type I error input in
          your endpoint calculation (for example, two-sample t-test).
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
          title="Multiplicity adjustment"
          markdown={exportMarkdown}
          disabled={calculating}
        />
      {:else}
        <p class="muted">Run a calculation to see adjusted alpha.</p>
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
    margin: 0;
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
