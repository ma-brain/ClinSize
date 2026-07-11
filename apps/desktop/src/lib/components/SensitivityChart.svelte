<script lang="ts">
  import type { SensitivityPoint } from "$lib/sensitivity/types";
  import { exportChartAsPng, exportChartAsSvg } from "$lib/workflow/chartExport";

  let {
    points,
    parameterLabel,
    outputLabel,
    formatParameterValue = (value: number) => value.toFixed(3),
    formatOutputValue = (value: number) => value.toFixed(2),
    fileStem = "clinsize-sensitivity-chart",
  }: {
    points: SensitivityPoint[];
    parameterLabel: string;
    outputLabel: string;
    formatParameterValue?: (value: number) => string;
    formatOutputValue?: (value: number) => string;
    fileStem?: string;
  } = $props();

  let svgElement = $state<SVGSVGElement | null>(null);
  let exporting = $state(false);
  let exportMessage = $state<string | null>(null);

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
    const rawYMin = yMin - yPadding;
    const yAxisMin = yMin >= 0 ? Math.max(0, rawYMin) : rawYMin;
    const yAxisMax = yMax + yPadding;
    const yAxisSpan = yAxisMax - yAxisMin || 1;

    return {
      xMin,
      xMax,
      yAxisMin,
      yAxisMax,
      xToPx: (value: number) => margin.left + ((value - xMin) / xSpan) * plotWidth,
      yToPx: (value: number) =>
        margin.top + plotHeight - ((value - yAxisMin) / yAxisSpan) * plotHeight,
      yTicks: [yAxisMin, (yAxisMin + yAxisMax) / 2, yAxisMax],
      xTicks: [xMin, (xMin + xMax) / 2, xMax],
    };
  });

  const polyline = $derived.by(() => {
    if (!scales) return "";
    return validPoints
      .map((point) => `${scales.xToPx(point.parameterValue)},${scales.yToPx(point.outputValue as number)}`)
      .join(" ");
  });

  async function saveChart(format: "png" | "svg") {
    if (!svgElement) return;

    exporting = true;
    exportMessage = null;
    try {
      const saved =
        format === "png"
          ? await exportChartAsPng(svgElement, fileStem)
          : await exportChartAsSvg(svgElement, fileStem);
      if (saved) {
        exportMessage = `Chart saved as ${format.toUpperCase()}.`;
      }
    } catch (error) {
      exportMessage = String(error);
    } finally {
      exporting = false;
    }
  }
</script>

{#if validPoints.length === 0}
  <p class="muted">No valid sensitivity points to plot.</p>
{:else if scales}
  <figure class="chart">
    <svg
      bind:this={svgElement}
      viewBox={`0 0 ${width} ${height}`}
      role="img"
      aria-label="Sensitivity chart"
    >
      <defs>
        <linearGradient id="sensitivity-line-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stop-color="var(--accent)" />
          <stop offset="100%" stop-color="var(--accent-gradient-end)" />
        </linearGradient>
      </defs>
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

      <polyline points={polyline} class="line" fill="none" />
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

    <div class="chart-actions">
      <button type="button" disabled={exporting} onclick={() => saveChart("png")}>
        Save as PNG
      </button>
      <button type="button" disabled={exporting} onclick={() => saveChart("svg")}>
        Save as SVG
      </button>
      {#if exportMessage}
        <p class="export-message">{exportMessage}</p>
      {/if}
    </div>
  </figure>
{/if}

<style>
  .chart {
    margin: 0;
    width: 100%;
  }

  .chart-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    margin-top: 0.65rem;
  }

  .chart-actions button {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 0.4rem 0.7rem;
    background: var(--bg-panel);
    color: var(--text-primary);
    font: inherit;
    font-size: 0.8125rem;
    cursor: pointer;
  }

  .chart-actions button:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--accent);
  }

  .chart-actions button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .export-message {
    width: 100%;
    margin: 0;
    font-size: 0.75rem;
    color: var(--text-muted);
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
    stroke: url(#sensitivity-line-gradient);
    stroke-width: 2;
  }

  .point {
    fill: var(--accent-gradient-end);
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
