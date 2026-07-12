<script lang="ts">
  import SensitivityChart from "$lib/components/SensitivityChart.svelte";
  import type { SensitivityOptionDef, SensitivityPoint } from "$lib/sensitivity/types";
  import { calculateMethod } from "$lib/workflow/methodDispatch";

  let {
    ready,
    inputSignature,
    methodId,
    buildInput,
    options,
    getOutputValue,
    outputLabel,
    formatParameterValue,
    formatOutputValue,
    defaultExpanded = false,
    chartFileStem,
  }: {
    ready: boolean;
    inputSignature: string;
    methodId: string;
    buildInput: () => unknown;
    options: SensitivityOptionDef[];
    getOutputValue: (result: unknown) => number;
    outputLabel: string;
    formatParameterValue?: (parameterId: string, value: number) => string;
    formatOutputValue?: (value: number) => string;
    defaultExpanded?: boolean;
    chartFileStem?: string;
  } = $props();

  const initialExpanded = defaultExpanded;
  let expanded = $state(initialExpanded);
  let selectedOptionId = $state("");
  let points = $state<SensitivityPoint[]>([]);
  let running = $state(false);
  let sweepError = $state<string | null>(null);

  const selectedOption = $derived(
    options.find((option) => option.id === selectedOptionId) ?? options[0] ?? null,
  );

  $effect(() => {
    inputSignature;
    points = [];
    sweepError = null;
  });

  $effect(() => {
    if (!options.some((option) => option.id === selectedOptionId)) {
      selectedOptionId = options[0]?.id ?? "";
    }
  });

  async function runSensitivity() {
    if (!ready || !selectedOption) return;

    running = true;
    sweepError = null;
    const baseInput = buildInput();
    const values = selectedOption.getValues();
    const nextPoints: SensitivityPoint[] = [];

    for (const value of values) {
      const input = selectedOption.mutate(baseInput, value);
      try {
        const result = await calculateMethod<unknown, unknown>(methodId, input);
        nextPoints.push({
          parameterValue: value,
          outputValue: getOutputValue(result),
        });
      } catch (error) {
        nextPoints.push({
          parameterValue: value,
          outputValue: null,
          error: String(error),
        });
      }
    }

    points = nextPoints;
    running = false;
  }

  function defaultFormatParameter(parameterId: string, value: number): string {
    if (parameterId === "alpha" || parameterId === "power" || parameterId === "dropoutRate") {
      return value.toFixed(3);
    }
    if (parameterId === "baselineOutcomeCorrelation") {
      return value.toFixed(2);
    }
    return value.toFixed(2);
  }

  function defaultFormatOutput(value: number): string {
    if (outputLabel.toLowerCase().includes("power")) {
      return value.toFixed(3);
    }
    return String(Math.ceil(value));
  }

  const formatParameter = (parameterId: string, value: number) =>
    formatParameterValue?.(parameterId, value) ?? defaultFormatParameter(parameterId, value);

  const formatOutput = (value: number) =>
    formatOutputValue?.(value) ?? defaultFormatOutput(value);

  const chartStem = $derived(
    chartFileStem ??
      `clinsize-sensitivity-${(selectedOption?.label ?? "chart")
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/(^-|-$)/g, "")}`,
  );
</script>

<section class="sensitivity">
  <button
    type="button"
    class="toggle"
    onclick={() => {
      expanded = !expanded;
    }}
    aria-expanded={expanded}
  >
    {expanded ? "Hide sensitivity analysis" : "Show sensitivity analysis"}
  </button>

  {#if expanded}
    {#if !ready}
      <p class="muted">Run a base calculation first to enable sensitivity analysis.</p>
    {:else}
      <div class="controls">
        <label>
          Vary parameter
          <select bind:value={selectedOptionId} disabled={running}>
            {#each options as option}
              <option value={option.id}>{option.label}</option>
            {/each}
          </select>
        </label>

        <button type="button" class="run-button" onclick={runSensitivity} disabled={running || !selectedOption}>
          {running ? "Running…" : "Run sensitivity"}
        </button>
      </div>

      {#if sweepError}
        <p class="error">{sweepError}</p>
      {/if}

      {#if points.length > 0 && selectedOption}
        <SensitivityChart
          {points}
          parameterLabel={selectedOption.label}
          {outputLabel}
          fileStem={chartStem}
          formatParameterValue={(value) => formatParameter(selectedOption.id, value)}
          formatOutputValue={formatOutput}
        />

        <table class="table">
          <thead>
            <tr>
              <th>{selectedOption.label}</th>
              <th>{outputLabel}</th>
            </tr>
          </thead>
          <tbody>
            {#each points as point}
              <tr class:invalid={point.outputValue === null}>
                <td>{formatParameter(selectedOption.id, point.parameterValue)}</td>
                <td>
                  {#if point.outputValue === null}
                    Invalid
                  {:else}
                    {formatOutput(point.outputValue)}
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/if}
  {/if}
</section>

<style>
  .sensitivity {
    margin-top: 1.25rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .toggle {
    border: none;
    background: none;
    padding: 0;
    color: var(--accent);
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
  }

  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: flex-end;
    margin-top: 0.75rem;
  }

  label {
    display: grid;
    gap: 0.35rem;
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .controls select,
  .run-button {
    box-sizing: border-box;
    min-height: 2.375rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font: inherit;
    font-size: 0.875rem;
    line-height: 1.2;
    color: var(--text-primary);
  }

  .run-button {
    background: var(--bg-panel);
  }

  .controls select {
    min-width: 12rem;
    appearance: none;
    background-color: var(--bg-panel);
    background-image: linear-gradient(45deg, transparent 50%, var(--text-muted) 50%),
      linear-gradient(135deg, var(--text-muted) 50%, transparent 50%);
    background-position:
      calc(100% - 1.1rem) calc(50% - 0.15rem),
      calc(100% - 0.8rem) calc(50% - 0.15rem);
    background-size:
      0.35rem 0.35rem,
      0.35rem 0.35rem;
    background-repeat: no-repeat;
    padding-right: 2rem;
  }

  .run-button {
    cursor: pointer;
    white-space: nowrap;
  }

  .run-button:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
  }

  .run-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 0.75rem;
    font-size: 0.8125rem;
  }

  th,
  td {
    border: 1px solid var(--border);
    padding: 0.35rem 0.5rem;
    text-align: left;
  }

  th {
    background: var(--background);
    color: var(--muted);
    font-weight: 500;
  }

  tr.invalid td {
    color: #9b1c1c;
  }

  .error {
    color: #9b1c1c;
    font-size: 0.8125rem;
    margin: 0.5rem 0 0;
  }

  .muted {
    color: var(--muted);
    font-size: 0.8125rem;
    margin: 0.5rem 0 0;
  }
</style>
