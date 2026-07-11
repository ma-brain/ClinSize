<script lang="ts">
  import SensitivityPanel from "$lib/components/SensitivityPanel.svelte";
  import { binaryEffectSensitivityOptions } from "$lib/sensitivity/configs";
  import type {
    Alternative,
    OddsRatioInput,
    OddsRatioResult,
    RiskRatioInput,
    RiskRatioResult,
    SolveMode,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  type Variant = "odds_ratio" | "risk_ratio";

  let {
    title,
    description,
    variant,
    calculateCommand,
    exportCommand,
    exportFilename,
    effectLabel,
  }: {
    title: string;
    description: string;
    variant: Variant;
    calculateCommand: string;
    exportCommand: string;
    exportFilename: string;
    effectLabel: string;
  } = $props();

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let controlN = $state("156");
  let controlRate = $state("0.25");
  let treatmentRate = $state("0.4");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let dropoutRate = $state("");

  let oddsResult = $state<OddsRatioResult | null>(null);
  let riskResult = $state<RiskRatioResult | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);

  const sensitivityOptions = $derived(
    binaryEffectSensitivityOptions(
      solveMode,
      controlRate,
      treatmentRate,
      alpha,
      power,
      allocationRatio,
      dropoutRate,
    ),
  );

  const sensitivitySignature = $derived(
    JSON.stringify({
      variant,
      solveMode,
      alpha,
      power,
      controlN,
      controlRate,
      treatmentRate,
      allocationRatio,
      alternative,
      dropoutRate,
    }),
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Total sample size" : "Achieved power",
  );

  function buildOddsInput(): OddsRatioInput {
    const input: OddsRatioInput = {
      solveMode,
      alpha: Number(alpha),
      controlRate: Number(controlRate),
      treatmentRate: Number(treatmentRate),
      allocationRatio: Number(allocationRatio),
      alternative,
    };
    if (solveMode === "sample_size") input.power = Number(power);
    else input.controlN = Number(controlN);
    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);
    return input;
  }

  function buildRiskInput(): RiskRatioInput {
    const input: RiskRatioInput = {
      solveMode,
      alpha: Number(alpha),
      controlRate: Number(controlRate),
      treatmentRate: Number(treatmentRate),
      allocationRatio: Number(allocationRatio),
      alternative,
    };
    if (solveMode === "sample_size") input.power = Number(power);
    else input.controlN = Number(controlN);
    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);
    return input;
  }

  function buildInput(): OddsRatioInput | RiskRatioInput {
    return variant === "odds_ratio" ? buildOddsInput() : buildRiskInput();
  }

  function effectValue(): number | null {
    if (variant === "odds_ratio" && oddsResult) return oddsResult.oddsRatio;
    if (variant === "risk_ratio" && riskResult) return riskResult.riskRatio;
    return null;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;
    oddsResult = null;
    riskResult = null;

    try {
      if (variant === "odds_ratio") {
        oddsResult = await invoke<OddsRatioResult>(calculateCommand, {
          input: buildOddsInput(),
        });
      } else {
        riskResult = await invoke<RiskRatioResult>(calculateCommand, {
          input: buildRiskInput(),
        });
      }
    } catch (error) {
      errorMessage = String(error);
    } finally {
      calculating = false;
    }
  }

  async function exportResult() {
    if (variant === "odds_ratio" && oddsResult) {
      const markdown = await invoke<string>(exportCommand, {
        input: buildOddsInput(),
        result: oddsResult,
      });
      downloadMarkdown(markdown, exportFilename);
    } else if (variant === "risk_ratio" && riskResult) {
      const markdown = await invoke<string>(exportCommand, {
        input: buildRiskInput(),
        result: riskResult,
      });
      downloadMarkdown(markdown, exportFilename);
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
        Control event rate
        <input type="number" min="0" max="1" step="0.01" bind:value={controlRate} />
      </label>

      <label>
        Treatment event rate
        <input type="number" min="0" max="1" step="0.01" bind:value={treatmentRate} />
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

      {#if oddsResult || riskResult}
        {@const active = oddsResult ?? riskResult}
        <dl class="results">
          <dt>Control N</dt>
          <dd>{active?.nControl}</dd>
          <dt>Treatment N</dt>
          <dd>{active?.nTreatment}</dd>
          <dt>Total N</dt>
          <dd>{active?.totalN}</dd>
          <dt>Achieved power</dt>
          <dd>{active?.achievedPower.toFixed(4)}</dd>
          <dt>{effectLabel}</dt>
          <dd>{effectValue()?.toFixed(4)}</dd>
          {#if active && active.nControlAdjusted !== active.nControl}
            <dt>Dropout-adjusted total N</dt>
            <dd>{active.totalNAdjusted}</dd>
          {/if}
        </dl>

        {#if active && active.warnings.length > 0}
          <div class="warnings">
            <h4>Warnings</h4>
            <ul>
              {#each active.warnings as warning}
                <li><strong>{warning.code}:</strong> {warning.message}</li>
              {/each}
            </ul>
          </div>
        {/if}

        <button class="secondary" onclick={exportResult}>Export Markdown</button>

        <SensitivityPanel
          ready={true}
          inputSignature={sensitivitySignature}
          command={calculateCommand}
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as OddsRatioResult | RiskRatioResult;
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
