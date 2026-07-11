<script lang="ts">
  import type { SensitivityPoint } from "$lib/sensitivity/types";

  let {
    points,
    parameterLabel,
    outputLabel,
    formatParameterValue = (value: number) => value.toFixed(3),
    formatOutputValue = (value: number) => value.toFixed(2),
  }: {
    points: SensitivityPoint[];
    parameterLabel: string;
    outputLabel: string;
    formatParameterValue?: (value: number) => string;
    formatOutputValue?: (value: number) => string;
  } = $props();

  const width = 520;
  const height = 220;
  const margin = { top: 16, right: 16, bottom: 44, left: 56 };
  const plotWidth = width - margin.left - margin.right;
  const plotHeight = height - margin.top - margin.bottom;

  const validPoints = $derived(points.filter((point) => point.outputValue !== null));

  const scales = $derived.by(() => {
    if (validPoints.length === 0) {
      return null;
    }

    const xValues = validPoints.map((point) => point.parameterValue);
    const yValues = validPoints.map((point) => point.outputValue as number);
    const xMin = Math.min(...xValues);
    const xMax = Math.max(...xValues);
    const yMin = Math.min(...yValues);
    const yMax = Math.max(...yValues);
    const xSpan = xMax - xMin || 1;
    const ySpan = yMax - yMin || 1;
    const yPadding = ySpan * 0.08;

    return {
      xMin,
      xMax,
      yMin: yMin - yPadding,
      yMax: yMax + yPadding,
      xToPx: (value: number) => margin.left + ((value - xMin) / xSpan) * plotWidth,
      yToPx: (value: number) =>
        margin.top + plotHeight - ((value - (yMin - yPadding)) / (ySpan + 2 * yPadding)) * plotHeight,
      yTicks: [yMin - yPadding, (yMin + yMax) / 2, yMax + yPadding],
      xTicks: [xMin, (xMin + xMax) / 2, xMax],
    };
  });

  const polyline = $derived.by(() => {
    if (!scales) return "";
    return validPoints
      .map((point) => `${scales.xToPx(point.parameterValue)},${scales.yToPx(point.outputValue as number)}`)
      .join(" ");
  });
</script>

{#if validPoints.length === 0}
  <p class="muted">No valid sensitivity points to plot.</p>
{:else if scales}
  <figure class="chart">
    <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Sensitivity chart">
      <line
        x1={margin.left}
        y1={margin.top + plotHeight}
        x2={margin.left + plotWidth}
        y2={margin.top + plotHeight}
        class="axis"
      />
      <line
        x1={margin.left}
        y1={margin.top}
        x2={margin.left}
        y2={margin.top + plotHeight}
        class="axis"
      />

      {#each scales.yTicks as tick}
        <line
          x1={margin.left}
          y1={scales.yToPx(tick)}
          x2={margin.left + plotWidth}
          y2={scales.yToPx(tick)}
          class="grid"
        />
        <text x={margin.left - 8} y={scales.yToPx(tick) + 4} class="tick" text-anchor="end">
          {formatOutputValue(tick)}
        </text>
      {/each}

      {#each scales.xTicks as tick}
        <text
          x={scales.xToPx(tick)}
          y={margin.top + plotHeight + 20}
          class="tick"
          text-anchor="middle"
        >
          {formatParameterValue(tick)}
        </text>
      {/each}

      <polyline points={polyline} class="line" />
      {#each validPoints as point}
        <circle
          cx={scales.xToPx(point.parameterValue)}
          cy={scales.yToPx(point.outputValue as number)}
          r="3.5"
          class="point"
        />
      {/each}

      <text
        x={margin.left + plotWidth / 2}
        y={height - 6}
        class="axis-label"
        text-anchor="middle"
      >
        {parameterLabel}
      </text>
      <text
        x={14}
        y={margin.top + plotHeight / 2}
        class="axis-label"
        text-anchor="middle"
        transform={`rotate(-90 14 ${margin.top + plotHeight / 2})`}
      >
        {outputLabel}
      </text>
    </svg>
  </figure>
{/if}

<style>
  .chart {
    margin: 0;
    width: 100%;
  }

  svg {
    width: 100%;
    height: auto;
    display: block;
  }

  .axis {
    stroke: var(--border);
    stroke-width: 1;
  }

  .grid {
    stroke: #eef1f5;
    stroke-width: 1;
  }

  .line {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
  }

  .point {
    fill: var(--accent);
  }

  .tick {
    fill: var(--muted);
    font-size: 10px;
  }

  .axis-label {
    fill: var(--muted);
    font-size: 11px;
  }

  .muted {
    color: var(--muted);
    font-size: 0.8125rem;
    margin: 0;
  }
</style>
