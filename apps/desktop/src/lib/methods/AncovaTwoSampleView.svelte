<script lang="ts">
  import SensitivityPanel from "$lib/components/SensitivityPanel.svelte";
  import { ancovaSensitivityOptions } from "$lib/sensitivity/configs";
  import type {
    Alternative,
    AncovaTwoSampleInput,
    AncovaTwoSampleResult,
    SolveMode,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let controlN = $state("132");
  let meanDifference = $state("3");
  let standardDeviation = $state("10");
  let baselineOutcomeCorrelation = $state("0.5");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let dropoutRate = $state("");

  let result = $state<AncovaTwoSampleResult | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);

  const sensitivityOptions = $derived(
    ancovaSensitivityOptions(
      solveMode,
      meanDifference,
      standardDeviation,
      baselineOutcomeCorrelation,
      alpha,
      power,
      allocationRatio,
      dropoutRate,
    ),
  );

  const sensitivitySignature = $derived(
    JSON.stringify({
      solveMode,
      alpha,
      power,
      controlN,
      meanDifference,
      standardDeviation,
      baselineOutcomeCorrelation,
      allocationRatio,
      alternative,
      dropoutRate,
    }),
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Total sample size" : "Achieved power",
  );

  function buildInput(): AncovaTwoSampleInput {
    const input: AncovaTwoSampleInput = {
      solveMode,
      alpha: Number(alpha),
      meanDifference: Number(meanDifference),
      standardDeviation: Number(standardDeviation),
      baselineOutcomeCorrelation: Number(baselineOutcomeCorrelation),
      allocationRatio: Number(allocationRatio),
      alternative,
    };

    if (solveMode === "sample_size") {
      input.power = Number(power);
    } else {
      input.controlN = Number(controlN);
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
      result = await invoke<AncovaTwoSampleResult>("calculate_ancova_two_sample", { input });
    } catch (error) {
      errorMessage = String(error);
    } finally {
      calculating = false;
    }
  }

  async function exportResult() {
    if (!result) return;
    const input = buildInput();
    const markdown = await invoke<string>("export_ancova_two_sample_markdown", {
      input,
      result,
    });

    const blob = new Blob([markdown], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "clinsize-ancova-two-sample.md";
    anchor.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="method-page">
  <header class="page-header">
    <h2>Two-sample ANCOVA</h2>
    <p>
      Parallel-group comparison with baseline covariate adjustment via approximate
      variance reduction.
    </p>
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
        Alternative hypothesis
        <select bind:value={alternative}>
          <option value="two_sided">Two-sided</option>
          <option value="greater">Greater (treatment &gt; control)</option>
          <option value="less">Less (treatment &lt; control)</option>
        </select>
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
          Control group N
          <input type="number" min="2" step="1" bind:value={controlN} />
        </label>
      {/if}

      <label>
        Mean difference (treatment − control)
        <input type="number" step="0.01" bind:value={meanDifference} />
      </label>

      <label>
        Unadjusted outcome standard deviation
        <input type="number" min="0" step="0.01" bind:value={standardDeviation} />
      </label>

      <label>
        Baseline-outcome correlation
        <input
          type="number"
          min="-0.99"
          max="0.99"
          step="0.01"
          bind:value={baselineOutcomeCorrelation}
        />
      </label>

      <label>
        Allocation ratio (treatment / control)
        <input type="number" min="0" step="0.01" bind:value={allocationRatio} />
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
          <dt>Control N</dt>
          <dd>{result.nControl}</dd>
          <dt>Treatment N</dt>
          <dd>{result.nTreatment}</dd>
          <dt>Total N</dt>
          <dd>{result.totalN}</dd>
          <dt>Achieved power</dt>
          <dd>{result.achievedPower.toFixed(4)}</dd>
          <dt>Effect size (Cohen's d, unadjusted SD)</dt>
          <dd>{result.effectSize.toFixed(4)}</dd>
          <dt>Adjusted standard deviation</dt>
          <dd>{result.adjustedStandardDeviation.toFixed(4)}</dd>
          <dt>Variance reduction factor (1 − ρ²)</dt>
          <dd>{result.varianceReductionFactor.toFixed(4)}</dd>
          {#if result.nControlAdjusted !== result.nControl}
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
          command="calculate_ancova_two_sample"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as AncovaTwoSampleResult;
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
