<script lang="ts">
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import { persistCalculation } from "$lib/workflow/record";
  import type {
    Alternative,
    BlindedSsreInput,
    BlindedSsreResult,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let alpha = $state("0.05");
  let targetPower = $state("0.8");
  let meanDifference = $state("1");
  let plannedStandardDeviation = $state("1");
  let blindedInterimStandardDeviation = $state("");
  let interimFraction = $state("0.5");
  let allocationRatio = $state("1");
  let maxSampleSizeMultiplier = $state("1.5");
  let alternative = $state<Alternative>("two_sided");

  let result = $state<BlindedSsreResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);

  function buildInput(): BlindedSsreInput {
    const input: BlindedSsreInput = {
      alpha: Number(alpha),
      targetPower: Number(targetPower),
      meanDifference: Number(meanDifference),
      plannedStandardDeviation: Number(plannedStandardDeviation),
      interimFraction: Number(interimFraction),
      allocationRatio: Number(allocationRatio),
      maxSampleSizeMultiplier: Number(maxSampleSizeMultiplier),
      alternative,
    };

    const interimSd = blindedInterimStandardDeviation.trim();
    if (interimSd !== "") {
      input.blindedInterimStandardDeviation = Number(interimSd);
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await invoke<BlindedSsreResult>("calculate_blinded_ssre", { input });
      exportMarkdown = await invoke<string>("export_blinded_ssre_markdown", {
        input,
        result,
      });
      persistCalculation({
        methodId: "design.blinded_ssre",
        methodName: "Blinded sample size re-estimation",
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
    <h2>Blinded sample size re-estimation</h2>
    <p>
      Plan a two-sample t-test with blinded variance re-estimation at an interim
      look. Uses the Friede-Kieser rule: update sample size from the blinded
      pooled interim SD while holding the planned treatment effect fixed.
    </p>
  </header>

  <div class="panels">
    <section class="panel">
      <h3>Parameters</h3>

      <label>
        Alpha
        <input type="number" min="0" max="1" step="0.001" bind:value={alpha} />
      </label>

      <label>
        Target power
        <input type="number" min="0" max="1" step="0.01" bind:value={targetPower} />
      </label>

      <label>
        Mean difference (treatment − control)
        <input type="number" step="0.1" bind:value={meanDifference} />
      </label>

      <label>
        Planned standard deviation (σ₀)
        <input type="number" min="0" step="0.1" bind:value={plannedStandardDeviation} />
      </label>

      <label>
        Blinded interim SD (s_b, optional)
        <input
          type="number"
          min="0"
          step="0.1"
          bind:value={blindedInterimStandardDeviation}
          placeholder="Defaults to planned SD"
        />
      </label>

      <label>
        Interim fraction (τ)
        <input type="number" min="0" max="1" step="0.05" bind:value={interimFraction} />
      </label>

      <label>
        Allocation ratio (treatment / control)
        <input type="number" min="0" step="0.1" bind:value={allocationRatio} />
      </label>

      <label>
        Maximum sample size multiplier
        <input type="number" min="1" step="0.1" bind:value={maxSampleSizeMultiplier} />
      </label>

      <label>
        Alternative
        <select bind:value={alternative}>
          <option value="two_sided">Two-sided</option>
          <option value="greater">Greater (one-sided)</option>
          <option value="less">Less (one-sided)</option>
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
          <dt>Planned per-arm N</dt>
          <dd>{result.plannedNControl} / {result.plannedNTreatment}</dd>
          <dt>Planned total N</dt>
          <dd>{result.plannedTotalN}</dd>
          <dt>Interim per-arm N</dt>
          <dd>{result.interimNControl} / {result.interimNTreatment}</dd>
          <dt>Variance ratio (s_b/σ₀)²</dt>
          <dd>{result.varianceRatio.toFixed(4)}</dd>
          <dt>Re-estimated per-arm N</dt>
          <dd>{result.reEstimatedNControl} / {result.reEstimatedNTreatment}</dd>
          <dt>Inflation factor</dt>
          <dd>{result.sampleSizeInflationFactor.toFixed(4)}</dd>
          <dt>Capped per-arm N</dt>
          <dd>{result.cappedNControl} / {result.cappedNTreatment}</dd>
          <dt>Capped total N</dt>
          <dd>{result.cappedTotalN}</dd>
          <dt>Capped inflation factor</dt>
          <dd>{result.cappedInflationFactor.toFixed(4)}</dd>
          <dt>Achieved power at capped N</dt>
          <dd>{result.achievedPowerAtCapped.toFixed(4)}</dd>
        </dl>

        {#if result.wasCapped}
          <p class="hint">
            Re-estimated sample size exceeded the maximum multiplier and was
            reduced to the pre-specified cap.
          </p>
        {/if}

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
          title="Blinded sample size re-estimation"
          markdown={exportMarkdown}
          disabled={calculating}
        />
      {:else}
        <p class="muted">Run a calculation to see re-estimated sample sizes.</p>
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
