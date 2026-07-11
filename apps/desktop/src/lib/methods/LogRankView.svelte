<script lang="ts">
  import SensitivityPanel from "$lib/components/SensitivityPanel.svelte";
  import { logRankSensitivityOptions } from "$lib/sensitivity/configs";
  import type { Alternative, LogRankInput, LogRankResult, SolveMode } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let totalEvents = $state("66");
  let hazardRatio = $state("0.5");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");

  let result = $state<LogRankResult | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);

  const sensitivityOptions = $derived(
    logRankSensitivityOptions(
      solveMode,
      hazardRatio,
      alpha,
      power,
      allocationRatio,
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
    }),
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Required total events" : "Achieved power",
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

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;
    result = null;

    try {
      const input = buildInput();
      result = await invoke<LogRankResult>("calculate_log_rank", { input });
    } catch (error) {
      errorMessage = String(error);
    } finally {
      calculating = false;
    }
  }

  async function exportResult() {
    if (!result) return;
    const markdown = await invoke<string>("export_log_rank_markdown", {
      input: buildInput(),
      result,
    });
    downloadMarkdown(markdown, "clinsize-log-rank.md");
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
    <h2>Log-rank test</h2>
    <p>
      Two-arm time-to-event design using the Schoenfeld approximation. Results
      are required events, not enrolled subjects.
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
          command="calculate_log_rank"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as LogRankResult;
            return solveMode === "sample_size" ? row.requiredEvents : row.achievedPower;
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
