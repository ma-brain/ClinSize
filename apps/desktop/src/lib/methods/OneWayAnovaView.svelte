<script lang="ts">
  import SensitivityPanel from "$lib/components/SensitivityPanel.svelte";
  import { oneWayAnovaSensitivityOptions } from "$lib/sensitivity/configs";
  import type { OneWayAnovaInput, OneWayAnovaResult, SolveMode } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let nPerGroup = $state("6");
  let nGroups = $state("3");
  let betweenVariance = $state("1");
  let withinSd = $state("1");
  let dropoutRate = $state("");

  let result = $state<OneWayAnovaResult | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);

  const sensitivityOptions = $derived(
    oneWayAnovaSensitivityOptions(
      solveMode,
      betweenVariance,
      withinSd,
      alpha,
      power,
      dropoutRate,
    ),
  );

  const sensitivitySignature = $derived(
    JSON.stringify({
      solveMode,
      alpha,
      power,
      nPerGroup,
      nGroups,
      betweenVariance,
      withinSd,
      dropoutRate,
    }),
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Total sample size" : "Achieved power",
  );

  function buildInput(): OneWayAnovaInput {
    const withinVariance = Number(withinSd) ** 2;
    const input: OneWayAnovaInput = {
      solveMode,
      alpha: Number(alpha),
      nGroups: Number(nGroups),
      betweenVariance: Number(betweenVariance),
      withinVariance,
    };

    if (solveMode === "sample_size") {
      input.power = Number(power);
    } else {
      input.nPerGroup = Number(nPerGroup);
    }

    if (dropoutRate.trim() !== "") {
      input.dropoutRate = Number(dropoutRate);
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;
    result = null;

    try {
      const input = buildInput();
      result = await invoke<OneWayAnovaResult>("calculate_one_way_anova", { input });
    } catch (error) {
      errorMessage = String(error);
    } finally {
      calculating = false;
    }
  }

  async function exportResult() {
    if (!result) return;
    const input = buildInput();
    const markdown = await invoke<string>("export_one_way_anova_markdown", {
      input,
      result,
    });

    const blob = new Blob([markdown], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "clinsize-one-way-anova.md";
    anchor.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="method-page">
  <header class="page-header">
    <h2>One-way ANOVA</h2>
    <p>Balanced fixed-effect comparison of means across multiple groups.</p>
  </header>

  <div class="panels">
    <section class="panel">
      <h3>Parameters</h3>

      <label>
        Solve mode
        <select bind:value={solveMode}>
          <option value="sample_size">Sample size</option>
          <option value="power">Power</option>
        </select>
      </label>

      <label>
        Number of groups
        <input type="number" min="2" step="1" bind:value={nGroups} />
      </label>

      <label>
        Type I error (alpha)
        <input type="number" min="0" max="1" step="0.001" bind:value={alpha} />
      </label>

      {#if solveMode === "sample_size"}
        <label>
          Target power
          <input type="number" min="0" max="1" step="0.01" bind:value={power} />
        </label>
      {:else}
        <label>
          N per group
          <input type="number" min="2" step="1" bind:value={nPerGroup} />
        </label>
      {/if}

      <label>
        Between-group variance
        <input type="number" min="0" step="0.01" bind:value={betweenVariance} />
      </label>

      <label>
        Within-group SD (σ)
        <input type="number" min="0" step="0.01" bind:value={withinSd} />
      </label>

      <label>
        Dropout rate (optional)
        <input type="number" min="0" max="0.99" step="0.01" bind:value={dropoutRate} />
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
          <dt>N per group</dt>
          <dd>{result.nPerGroup}</dd>
          <dt>Total N</dt>
          <dd>{result.totalN}</dd>
          <dt>Achieved power</dt>
          <dd>{result.achievedPower.toFixed(4)}</dd>
          <dt>Effect size (Cohen's f)</dt>
          <dd>{result.effectSize.toFixed(4)}</dd>
          {#if result.nPerGroupAdjusted !== result.nPerGroup}
            <dt>Dropout-adjusted total N</dt>
            <dd>{result.totalNAdjusted}</dd>
          {/if}
        </dl>

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

        <button class="secondary" onclick={exportResult}>Export Markdown</button>

        <SensitivityPanel
          ready={true}
          inputSignature={sensitivitySignature}
          command="calculate_one_way_anova"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as OneWayAnovaResult;
            return solveMode === "sample_size" ? row.totalN : row.achievedPower;
          }}
          outputLabel={sensitivityOutputLabel}
        />
      {:else}
        <p class="muted">Enter parameters and calculate to see results.</p>
      {/if}
    </section>
  </div>
</div>

<style>
  .method-page {
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

  .panels {
    display: grid;
    grid-template-columns: 20rem 1fr;
    gap: 1rem;
    margin-top: 1.25rem;
  }

  .panel {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
    padding: 1rem;
  }

  h3 {
    margin: 0 0 0.75rem;
    font-size: 0.9375rem;
    font-weight: 600;
  }

  label {
    display: grid;
    gap: 0.25rem;
    margin-bottom: 0.75rem;
    font-size: 0.8125rem;
  }

  input,
  select {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    font-size: 0.875rem;
    background: var(--background);
  }

  button {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.45rem 0.75rem;
    background: var(--background);
    cursor: pointer;
    font-size: 0.875rem;
  }

  button.secondary {
    margin-top: 1rem;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .results {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.35rem 1rem;
    font-size: 0.875rem;
    margin: 0;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    font-weight: 500;
  }

  .warnings {
    margin-top: 1rem;
    font-size: 0.8125rem;
  }

  .warnings h4 {
    margin: 0 0 0.35rem;
    font-size: 0.8125rem;
  }

  .warnings ul {
    margin: 0;
    padding-left: 1.1rem;
    color: var(--muted);
  }

  .error {
    color: #9b1c1c;
    font-size: 0.8125rem;
    margin: 0.75rem 0 0;
  }

  .muted {
    color: var(--muted);
    font-size: 0.875rem;
  }
</style>
