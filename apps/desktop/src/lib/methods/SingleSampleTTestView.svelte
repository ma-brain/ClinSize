<script lang="ts">
  import SensitivityPanel from "$lib/components/SensitivityPanel.svelte";
  import {
    oneSampleSensitivityOptions,
    pairedSensitivityOptions,
  } from "$lib/sensitivity/configs";
  import type {
    Alternative,
    OneSampleTTestInput,
    OneSampleTTestResult,
    PairedTTestInput,
    PairedTTestResult,
    SolveMode,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  type Variant = "one_sample" | "paired";

  let {
    title,
    description,
    meanDifferenceLabel,
    sizeLabel,
    variant,
  }: {
    title: string;
    description: string;
    meanDifferenceLabel: string;
    sizeLabel: string;
    variant: Variant;
  } = $props();

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let size = $state("10");
  let meanDifference = $state("1");
  let standardDeviation = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let dropoutRate = $state("");

  let oneSampleResult = $state<OneSampleTTestResult | null>(null);
  let pairedResult = $state<PairedTTestResult | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);

  const sensitivitySignature = $derived(
    JSON.stringify({
      variant,
      solveMode,
      alpha,
      power,
      size,
      meanDifference,
      standardDeviation,
      alternative,
      dropoutRate,
    }),
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Sample size" : "Achieved power",
  );

  const oneSampleSensitivity = $derived(
    oneSampleSensitivityOptions(
      solveMode,
      meanDifference,
      standardDeviation,
      alpha,
      power,
      dropoutRate,
    ),
  );

  const pairedSensitivity = $derived(
    pairedSensitivityOptions(
      solveMode,
      meanDifference,
      standardDeviation,
      alpha,
      power,
      dropoutRate,
    ),
  );

  function buildOneSampleInput(): OneSampleTTestInput {
    const input: OneSampleTTestInput = {
      solveMode,
      alpha: Number(alpha),
      meanDifference: Number(meanDifference),
      standardDeviation: Number(standardDeviation),
      alternative,
    };
    if (solveMode === "sample_size") input.power = Number(power);
    else input.n = Number(size);
    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);
    return input;
  }

  function buildPairedInput(): PairedTTestInput {
    const input: PairedTTestInput = {
      solveMode,
      alpha: Number(alpha),
      meanDifference: Number(meanDifference),
      standardDeviation: Number(standardDeviation),
      alternative,
    };
    if (solveMode === "sample_size") input.power = Number(power);
    else input.nPairs = Number(size);
    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);
    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;
    oneSampleResult = null;
    pairedResult = null;

    try {
      if (variant === "one_sample") {
        const input = buildOneSampleInput();
        oneSampleResult = await invoke<OneSampleTTestResult>(
          "calculate_one_sample_ttest",
          { input },
        );
      } else {
        const input = buildPairedInput();
        pairedResult = await invoke<PairedTTestResult>("calculate_paired_ttest", {
          input,
        });
      }
    } catch (error) {
      errorMessage = String(error);
    } finally {
      calculating = false;
    }
  }

  async function exportResult() {
    if (variant === "one_sample" && oneSampleResult) {
      const input = buildOneSampleInput();
      const markdown = await invoke<string>("export_one_sample_ttest_markdown", {
        input,
        result: oneSampleResult,
      });
      downloadMarkdown(markdown, "clinsize-one-sample-ttest.md");
    } else if (variant === "paired" && pairedResult) {
      const input = buildPairedInput();
      const markdown = await invoke<string>("export_paired_ttest_markdown", {
        input,
        result: pairedResult,
      });
      downloadMarkdown(markdown, "clinsize-paired-ttest.md");
    }
  }

  function downloadMarkdown(markdown: string, filename: string) {
    const blob = new Blob([markdown], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = filename;
    anchor.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="method-page">
  <header class="page-header">
    <h2>{title}</h2>
    <p>{description}</p>
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
          <option value="greater">Greater</option>
          <option value="less">Less</option>
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
          {sizeLabel}
          <input type="number" min="2" step="1" bind:value={size} />
        </label>
      {/if}

      <label>
        {meanDifferenceLabel}
        <input type="number" step="0.01" bind:value={meanDifference} />
      </label>

      <label>
        Standard deviation
        <input type="number" min="0" step="0.01" bind:value={standardDeviation} />
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

      {#if oneSampleResult}
        <dl class="results">
          <dt>{sizeLabel}</dt>
          <dd>{oneSampleResult.n}</dd>
          <dt>Achieved power</dt>
          <dd>{oneSampleResult.achievedPower.toFixed(4)}</dd>
          <dt>Effect size (Cohen's d)</dt>
          <dd>{oneSampleResult.effectSize.toFixed(4)}</dd>
          {#if oneSampleResult.nAdjusted !== oneSampleResult.n}
            <dt>Dropout-adjusted N</dt>
            <dd>{oneSampleResult.nAdjusted}</dd>
          {/if}
        </dl>
        {#if oneSampleResult.warnings.length > 0}
          <div class="warnings">
            <h4>Warnings</h4>
            <ul>
              {#each oneSampleResult.warnings as warning}
                <li><strong>{warning.code}:</strong> {warning.message}</li>
              {/each}
            </ul>
          </div>
        {/if}
        <button class="secondary" onclick={exportResult}>Export Markdown</button>
        <SensitivityPanel
          ready={true}
          inputSignature={sensitivitySignature}
          command="calculate_one_sample_ttest"
          buildInput={buildOneSampleInput}
          options={oneSampleSensitivity}
          getOutputValue={(value) => {
            const result = value as OneSampleTTestResult;
            return solveMode === "sample_size" ? result.n : result.achievedPower;
          }}
          outputLabel={sensitivityOutputLabel}
        />
      {:else if pairedResult}
        <dl class="results">
          <dt>{sizeLabel}</dt>
          <dd>{pairedResult.nPairs}</dd>
          <dt>Achieved power</dt>
          <dd>{pairedResult.achievedPower.toFixed(4)}</dd>
          <dt>Effect size (Cohen's d)</dt>
          <dd>{pairedResult.effectSize.toFixed(4)}</dd>
          {#if pairedResult.nPairsAdjusted !== pairedResult.nPairs}
            <dt>Dropout-adjusted pairs</dt>
            <dd>{pairedResult.nPairsAdjusted}</dd>
          {/if}
        </dl>
        {#if pairedResult.warnings.length > 0}
          <div class="warnings">
            <h4>Warnings</h4>
            <ul>
              {#each pairedResult.warnings as warning}
                <li><strong>{warning.code}:</strong> {warning.message}</li>
              {/each}
            </ul>
          </div>
        {/if}
        <button class="secondary" onclick={exportResult}>Export Markdown</button>
        <SensitivityPanel
          ready={true}
          inputSignature={sensitivitySignature}
          command="calculate_paired_ttest"
          buildInput={buildPairedInput}
          options={pairedSensitivity}
          getOutputValue={(value) => {
            const result = value as PairedTTestResult;
            return solveMode === "sample_size" ? result.nPairs : result.achievedPower;
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
