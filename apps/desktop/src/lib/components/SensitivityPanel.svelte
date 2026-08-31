<script lang="ts">
  import SensitivityChart from "$lib/components/SensitivityChart.svelte";
  import { linearRange } from "$lib/sensitivity/ranges";
  import type { SensitivityOptionDef, SensitivityPoint } from "$lib/sensitivity/types";
  import { calculateMethod } from "$lib/workflow/methodDispatch";

  let {
    inputSignature,
    methodId,
    buildInput,
    options,
    getOutputValue,
    outputLabel,
    formatParameterValue,
    formatOutputValue,
    chartFileStem,
  }: {
    inputSignature: string;
    methodId: string;
    buildInput: () => unknown;
    options: SensitivityOptionDef[];
    getOutputValue: (result: unknown, parameterId: string) => number;
    outputLabel: string;
    formatParameterValue?: (parameterId: string, value: number) => string;
    formatOutputValue?: (value: number) => string;
    chartFileStem?: string;
  } = $props();

  const MIN_POINTS = 2;
  const MAX_POINTS = 41;
  const DEBOUNCE_MS = 350;

  let selectedOptionId = $state("");
  let rangeMin = $state(0);
  let rangeMax = $state(1);
  // Slider track bounds and drag granularity. These are deliberately plain state,
  // NOT derived from rangeMin/rangeMax: they're also the min/max/step attributes of
  // the very inputs that write rangeMin/rangeMax, so deriving them reactively from
  // the value being dragged creates a feedback loop (each drag tick widens the
  // track, which widens the value-per-pixel ratio, which widens the track further —
  // the interval explodes within a second of dragging). They're set once per
  // parameter selection and only ever widened (never during a drag — a native
  // slider can't produce a value outside its own min/max) if a typed value in the
  // number inputs falls outside the current track.
  let trackMin = $state(0);
  let trackMax = $state(1);
  let dragStep = $state(0.1);
  let points = $state<SensitivityPoint[]>([]);
  let running = $state(false);
  let sweepError = $state<string | null>(null);

  const selectedOption = $derived(
    options.find((option) => option.id === selectedOptionId) ?? options[0] ?? null,
  );

  // Binary floating point can't represent most decimal fractions exactly, so
  // multiplying by a power-of-ten magnitude (below) tends to leave noise like
  // 0.06329999999999999 instead of 0.0633. Snap back to a clean decimal string at
  // the same precision the rounding was meant to produce.
  function cleanDecimal(value: number, magnitudeExponent: number): number {
    if (!Number.isFinite(value) || magnitudeExponent >= 0) return value;
    return Number(value.toFixed(-magnitudeExponent));
  }

  // Round to ~3 significant figures so computed defaults look clean (0.25, 2, 17 —
  // not 0.24999999996).
  function roundNice(value: number): number {
    if (!Number.isFinite(value) || value === 0) return value;
    const exponent = Math.floor(Math.log10(Math.abs(value))) - 2;
    const magnitude = Math.pow(10, exponent);
    return cleanDecimal(Math.round(value / magnitude) * magnitude, exponent);
  }

  // Pick a "nice" 1/2/5×10^k drag increment for a given span, so dragging the slider
  // never lands on ugly long-decimal values.
  function niceStep(rawStep: number): number {
    if (!Number.isFinite(rawStep) || rawStep <= 0) return 1;
    const exponent = Math.floor(Math.log10(rawStep));
    const magnitude = Math.pow(10, exponent);
    const residual = rawStep / magnitude;
    const niceResidual = residual <= 1 ? 1 : residual <= 2 ? 2 : residual <= 5 ? 5 : 10;
    return cleanDecimal(niceResidual * magnitude, exponent);
  }

  // Dropout rate is a 0–1 fraction but should behave like whole percentage points.
  function dropoutStep(span: number): number {
    if (span <= 0.1) return 0.01;
    if (span <= 0.3) return 0.05;
    return 0.1;
  }

  function defaultBoundsFor(option: SensitivityOptionDef): { min: number; max: number } {
    const values = option.getValues();
    let lo = Math.min(...values);
    let hi = Math.max(...values);
    if (!Number.isFinite(lo) || !Number.isFinite(hi) || lo === hi) {
      lo = (lo || 0) - 1;
      hi = (hi || 0) + 1;
    }
    if (option.id === "dropoutRate") {
      lo = 0;
    }
    return { min: roundNice(lo), max: roundNice(hi) };
  }

  // (Re)initialize the interval whenever the selected parameter changes.
  $effect(() => {
    if (!options.some((option) => option.id === selectedOptionId)) {
      selectedOptionId = options[0]?.id ?? "";
    }
  });

  let lastOptionId = "";
  $effect(() => {
    if (selectedOption && selectedOption.id !== lastOptionId) {
      lastOptionId = selectedOption.id;
      const { min, max } = defaultBoundsFor(selectedOption);
      rangeMin = min;
      rangeMax = max;

      // The slider track gets generous room to widen the interval beyond the
      // suggested default, but dropout rate is pinned to [0, 90%] since it's a
      // fraction.
      if (selectedOption.id === "dropoutRate") {
        trackMin = 0;
        trackMax = 0.9;
        dragStep = dropoutStep(max - min);
      } else {
        const span = max - min || 1;
        const values = selectedOption.getValues();
        const lo = roundNice(min - span);
        trackMin = values.length > 0 && Math.min(...values) >= 0 ? Math.max(0, lo) : lo;
        trackMax = roundNice(max + span);
        dragStep = niceStep(span / 20);
      }
    }
  });

  // If a typed value (the number inputs allow free typing) falls outside the
  // current track, widen the track to fit it — but only ever widen, and only in
  // reaction to the value, never derived from it in the other direction, so this
  // can't compound into the feedback loop above.
  $effect(() => {
    if (rangeMin < trackMin) trackMin = rangeMin;
    if (rangeMax > trackMax) trackMax = rangeMax;
  });

  const pointCount = $derived(
    Math.min(
      MAX_POINTS,
      Math.max(MIN_POINTS, Math.round((rangeMax - rangeMin) / (dragStep || 1)) + 1),
    ),
  );

  // Guard against the min/max controls being typed past each other into an
  // inverted or degenerate interval.
  $effect(() => {
    if (rangeMin >= rangeMax) {
      rangeMax = Math.min(trackMax, rangeMin + (dragStep || Math.abs(rangeMin) * 0.1 || 1));
      if (rangeMax <= rangeMin) {
        rangeMin = Math.max(trackMin, rangeMax - (dragStep || 1));
      }
    }
  });

  let runToken = 0;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  async function runSweep() {
    if (!selectedOption || !(rangeMax > rangeMin)) return;

    const token = ++runToken;
    running = true;
    sweepError = null;
    const baseInput = buildInput();
    const values = linearRange(rangeMin, rangeMax, pointCount);
    const nextPoints: SensitivityPoint[] = [];

    for (const value of values) {
      const input = selectedOption.mutate(baseInput, value);
      try {
        const result = await calculateMethod<unknown, unknown>(methodId, input);
        nextPoints.push({
          parameterValue: value,
          outputValue: getOutputValue(result, selectedOption.id),
        });
      } catch (error) {
        nextPoints.push({
          parameterValue: value,
          outputValue: null,
          error: String(error),
        });
      }
      if (token !== runToken) return;
    }

    points = nextPoints;
    running = false;
  }

  function scheduleSweep() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(runSweep, DEBOUNCE_MS);
  }

  $effect(() => {
    // Re-run automatically whenever the base inputs, the chosen parameter, or the
    // interval changes.
    inputSignature;
    selectedOptionId;
    rangeMin;
    rangeMax;
    scheduleSweep();
    return () => {
      if (debounceTimer) clearTimeout(debounceTimer);
    };
  });

  function defaultFormatParameter(parameterId: string, value: number): string {
    // Alpha can be a small fraction (e.g. a gatekeeping-adjusted value); keep it more
    // precise so it doesn't round away to 0.00. Dropout rate reads as a whole
    // percentage. Everything else is 2 decimals.
    if (parameterId === "alpha") {
      return value.toFixed(4);
    }
    if (parameterId === "dropoutRate") {
      return `${Math.round(value * 100)}%`;
    }
    return value.toFixed(2);
  }

  function defaultFormatOutput(value: number): string {
    if (outputLabel.toLowerCase().includes("power")) {
      return value.toFixed(2);
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

  // Thumb positions as percentages along the visible track, for the highlighted-range
  // fill between the two handles.
  const minPercent = $derived(
    trackMax > trackMin ? ((rangeMin - trackMin) / (trackMax - trackMin)) * 100 : 0,
  );
  const maxPercent = $derived(
    trackMax > trackMin ? ((rangeMax - trackMin) / (trackMax - trackMin)) * 100 : 100,
  );
</script>

<div class="sensitivity">
  <div class="controls">
    <label>
      Vary parameter
      <select bind:value={selectedOptionId}>
        {#each options as option}
          <option value={option.id}>{option.label}</option>
        {/each}
      </select>
    </label>
  </div>

  {#if selectedOption}
    <div class="range-controls">
      <div class="range-field">
        <div class="range-label">
          <span>Interval</span>
          <span class="range-values">
            <input
              class="range-value"
              type="number"
              step="any"
              bind:value={rangeMin}
              aria-label="Interval minimum"
            />
            <span class="range-sep">–</span>
            <input
              class="range-value"
              type="number"
              step="any"
              bind:value={rangeMax}
              aria-label="Interval maximum"
            />
          </span>
        </div>

        <div class="dual-slider">
          <div class="dual-slider-track"></div>
          <div
            class="dual-slider-fill"
            style="left: {minPercent}%; right: {100 - maxPercent}%"
          ></div>
          <input
            type="range"
            class="slider"
            min={trackMin}
            max={trackMax}
            step={dragStep}
            bind:value={rangeMin}
            aria-label="Interval minimum slider"
          />
          <input
            type="range"
            class="slider"
            min={trackMin}
            max={trackMax}
            step={dragStep}
            bind:value={rangeMax}
            aria-label="Interval maximum slider"
          />
        </div>

        <p class="range-hint">Evaluating {pointCount} points across the interval.</p>
      </div>
    </div>
  {/if}

  {#if running}
    <p class="status">Updating…</p>
  {/if}

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
</div>

<style>
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: flex-end;
  }

  label {
    display: grid;
    gap: 0.35rem;
    font-size: 0.8125rem;
    font-weight: 500;
  }

  .controls select {
    box-sizing: border-box;
    min-height: 2.375rem;
    min-width: 12rem;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font: inherit;
    font-size: 0.875rem;
    line-height: 1.2;
    color: var(--text-primary);
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

  .range-controls {
    display: grid;
    gap: 0.75rem;
    margin-top: 0.9rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border);
  }

  .range-field {
    display: grid;
    gap: 0.4rem;
  }

  .range-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-muted);
  }

  .range-values {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .range-sep {
    color: var(--text-muted);
  }

  .range-hint {
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .range-value {
    box-sizing: border-box;
    width: 5.5rem;
    padding: 0.2rem 0.4rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-panel);
    color: var(--text-primary);
    font: inherit;
    font-size: 0.75rem;
    text-align: right;
  }

  /* Dual-handle range slider: two overlapping native <input type="range"> elements
     with transparent tracks so only their thumbs are interactive; a separate div
     renders the highlighted span between the two handles. */
  .dual-slider {
    position: relative;
    height: 1.5rem;
    display: flex;
    align-items: center;
  }

  .dual-slider-track {
    position: absolute;
    left: 0;
    right: 0;
    height: 4px;
    border-radius: 999px;
    background: var(--border);
  }

  .dual-slider-fill {
    position: absolute;
    height: 4px;
    border-radius: 999px;
    background: var(--accent);
  }

  .dual-slider .slider {
    position: absolute;
    left: 0;
    right: 0;
    width: 100%;
    margin: 0;
    background: none;
    appearance: none;
    -webkit-appearance: none;
    pointer-events: none;
  }

  .dual-slider .slider::-webkit-slider-runnable-track {
    -webkit-appearance: none;
    background: transparent;
    height: 4px;
  }

  .dual-slider .slider::-moz-range-track {
    background: transparent;
    height: 4px;
  }

  .dual-slider .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    pointer-events: auto;
    width: 16px;
    height: 16px;
    margin-top: -6px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid var(--bg-panel);
    box-shadow: 0 0 0 1px var(--accent);
    cursor: pointer;
  }

  .dual-slider .slider::-moz-range-thumb {
    pointer-events: auto;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    border: 2px solid var(--bg-panel);
    box-shadow: 0 0 0 1px var(--accent);
    cursor: pointer;
  }

  .status {
    margin: 0.6rem 0 0;
    font-size: 0.75rem;
    color: var(--text-muted);
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
</style>
