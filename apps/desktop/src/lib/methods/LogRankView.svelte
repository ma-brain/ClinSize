<script lang="ts">
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import SensitivityPanel from "$lib/components/SensitivityPanel.svelte";
  import { logRankSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import type { Alternative, LogRankInput, LogRankResult, SolveMode } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let totalEvents = $state("66");
  let hazardRatio = $state("0.5");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let controlHazardRate = $state("0.1155");
  let accrualDuration = $state("12");
  let minimumFollowUp = $state("18");
  let dropoutHazardRate = $state("");
  let includeAccrual = $state(true);

  let result = $state<LogRankResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);

  const sensitivityOptions = $derived(
    logRankSensitivityOptions(
      solveMode,
      hazardRatio,
      alpha,
      power,
      allocationRatio,
      includeAccrual,
      controlHazardRate,
      accrualDuration,
      minimumFollowUp,
      dropoutHazardRate,
    ),
  );

  const sensitivitySignature = $derived(
    JSON.stringify({
      solveMode,
      alpha,
      power,
      totalEvents,
      hazardRatio,
      allocationRatio,
      alternative,
      includeAccrual,
      controlHazardRate,
      accrualDuration,
      minimumFollowUp,
      dropoutHazardRate,
    }),
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size"
      ? includeAccrual
        ? "Total enrolled subjects"
        : "Required total events"
      : "Achieved power",
  );

  function buildInput(): LogRankInput {
    const input: LogRankInput = {
      solveMode,
      alpha: Number(alpha),
      hazardRatio: Number(hazardRatio),
      allocationRatio: Number(allocationRatio),
      alternative,
    };

    if (solveMode === "sample_size") input.power = Number(power);
    else input.totalEvents = Number(totalEvents);

    if (includeAccrual) {
      input.controlHazardRate = Number(controlHazardRate);
      input.accrualDuration = Number(accrualDuration);
      input.minimumFollowUp = Number(minimumFollowUp);
      if (dropoutHazardRate.trim() !== "") {
        input.dropoutHazardRate = Number(dropoutHazardRate);
      }
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;
    result = null;

    try {
      const input = buildInput();
      result = await invoke<LogRankResult>("calculate_log_rank", { input });
      exportMarkdown = await invoke<string>("export_log_rank_markdown", { input, result });
      persistCalculation({
        methodId: "survival.log_rank",
        methodName: "Log-rank test",
        input,
        result,
      });
    } catch (error) {
      errorMessage = String(error);
      exportMarkdown = null;
    } finally {
      calculating = false;
    }
  }

</script>

<div class="method-page">
  <header class="page-header">
    <h2>Log-rank test</h2>
    <p>
      Two-arm time-to-event design using the Schoenfeld approximation. Provide
      accrual assumptions to translate required events into enrolled subjects.
    </p>
  </header>

  <div class="panels">
    <section class="panel">
      <h3>Parameters</h3>

      <label>
        Solve mode
        <select bind:value={solveMode}>
          <option value="sample_size">Required events</option>
          <option value="power">Power</option>
        </select>
      </label>

      <label>
        Alternative hypothesis
        <select bind:value={alternative}>
          <option value="two_sided">Two-sided</option>
          <option value="greater">One-sided (treatment hazard &lt; control)</option>
          <option value="less">One-sided (treatment hazard &gt; control)</option>
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
          Total events
          <input type="number" min="1" step="1" bind:value={totalEvents} />
        </label>
      {/if}

      <label>
        Hazard ratio (treatment / control)
        <input type="number" min="0" step="0.01" bind:value={hazardRatio} />
      </label>

      <label>
        Allocation ratio (treatment / control)
        <input type="number" min="0" step="0.01" bind:value={allocationRatio} />
      </label>

      <label class="checkbox">
        <input type="checkbox" bind:checked={includeAccrual} />
        Include accrual and follow-up assumptions
      </label>

      {#if includeAccrual}
        <label>
          Control hazard rate
          <input type="number" min="0" step="0.001" bind:value={controlHazardRate} />
        </label>

        <label>
          Accrual duration
          <input type="number" min="0" step="0.1" bind:value={accrualDuration} />
        </label>

        <label>
          Minimum follow-up
          <input type="number" min="0" step="0.1" bind:value={minimumFollowUp} />
        </label>

        <label>
          Dropout hazard rate (optional)
          <input type="number" min="0" step="0.0001" bind:value={dropoutHazardRate} />
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
          <dt>Required total events</dt>
          <dd>{result.requiredEvents}</dd>
          <dt>Expected control events</dt>
          <dd>{result.eventsControl}</dd>
          <dt>Expected treatment events</dt>
          <dd>{result.eventsTreatment}</dd>
          <dt>Achieved power</dt>
          <dd>{result.achievedPower.toFixed(4)}</dd>
          <dt>Hazard ratio</dt>
          <dd>{result.hazardRatio.toFixed(4)}</dd>
          {#if result.totalN}
            <dt>Control N</dt>
            <dd>{result.nControl}</dd>
            <dt>Treatment N</dt>
            <dd>{result.nTreatment}</dd>
            <dt>Total enrolled subjects</dt>
            <dd>{result.totalN}</dd>
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

        <ExportMenu title="Log-rank test" markdown={exportMarkdown} />

        <SensitivityPanel
          ready={true}
          inputSignature={sensitivitySignature}
          command="calculate_log_rank"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as LogRankResult;
            if (solveMode === "sample_size") {
              return includeAccrual ? (row.totalN ?? row.requiredEvents) : row.requiredEvents;
            }
            return row.achievedPower;
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

  .checkbox {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .checkbox input {
    width: auto;
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
