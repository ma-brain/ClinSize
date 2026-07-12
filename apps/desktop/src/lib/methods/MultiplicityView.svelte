<script lang="ts">
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import MethodPage from "$lib/components/MethodPage.svelte";
  import AssumptionsCard from "$lib/components/ui/AssumptionsCard.svelte";
  import Field from "$lib/components/ui/Field.svelte";
  import MethodHeader from "$lib/components/ui/MethodHeader.svelte";
  import Panel from "$lib/components/ui/Panel.svelte";
  import PrimaryButton from "$lib/components/ui/PrimaryButton.svelte";
  import RationaleCard from "$lib/components/ui/RationaleCard.svelte";
  import ProtocolTextCard from "$lib/components/ui/ProtocolTextCard.svelte";
  import ResultGrid from "$lib/components/ui/ResultGrid.svelte";
  import ResultHero from "$lib/components/ui/ResultHero.svelte";
  import Section from "$lib/components/ui/Section.svelte";
  import WarningList from "$lib/components/ui/WarningList.svelte";
  import { persistCalculation } from "$lib/workflow/record";
  import { calculateMethod, exportMethodMarkdown } from "$lib/workflow/methodDispatch";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    MultiplicityInput,
    MultiplicityMethod,
    MultiplicityResult,
  } from "$lib/types";

  let familyWiseAlpha = $state("0.05");
  let numberOfComparisons = $state("2");
  let adjustmentMethod = $state<MultiplicityMethod>("bonferroni");
  let gatePosition = $state("1");
  let comparisonWeights = $state("");

  let result = $state<MultiplicityResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let rationale = $state<string | null>(null);
  let protocolText = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);
  let lastCalculatedSignature = $state<string | null>(null);

  const inputSignature = $derived(
    JSON.stringify({
      familyWiseAlpha,
      numberOfComparisons,
      adjustmentMethod,
      gatePosition,
      comparisonWeights,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const comparisonLabel = $derived(
    adjustmentMethod === "dunnett"
      ? "Number of treatment arms (vs control)"
      : "Number of comparisons",
  );

  const showGatePosition = $derived(
    adjustmentMethod === "holm" ||
      adjustmentMethod === "hochberg" ||
      adjustmentMethod === "graphical",
  );

  const showComparisonWeights = $derived(adjustmentMethod === "graphical");

  const methodLabel = $derived(
    adjustmentMethod === "bonferroni"
      ? "Bonferroni"
      : adjustmentMethod === "sidak"
        ? "Šidák"
        : adjustmentMethod === "dunnett"
          ? "Dunnett"
          : adjustmentMethod === "holm"
            ? "Holm"
            : adjustmentMethod === "hochberg"
              ? "Hochberg"
              : "Graphical",
  );

  const resultItems = $derived(
    result
      ? [
          {
            label: "Adjusted per-comparison alpha",
            value: result.adjustedAlpha.toFixed(6),
          },
          { label: "Family-wise alpha", value: result.familyWiseAlpha.toFixed(4) },
          { label: "Number of comparisons", value: String(result.numberOfComparisons) },
          ...(result.gatePosition
            ? [{ label: "Gate position", value: String(result.gatePosition) }]
            : []),
          ...(result.comparisonWeight
            ? [{ label: "Comparison weight", value: result.comparisonWeight.toFixed(4) }]
            : []),
          {
            label: "Alpha reduction factor",
            value: result.alphaReductionFactor.toFixed(4),
          },
        ]
      : [],
  );

  function parseComparisonWeights(): number[] | undefined {
    const trimmed = comparisonWeights.trim();
    if (!trimmed) {
      return undefined;
    }

    return trimmed.split(",").map((part) => Number(part.trim()));
  }

  function buildInput(): MultiplicityInput {
    const input: MultiplicityInput = {
      familyWiseAlpha: Number(familyWiseAlpha),
      numberOfComparisons: Number(numberOfComparisons),
      adjustmentMethod,
    };

    if (showGatePosition) {
      input.gatePosition = Number(gatePosition);
    }

    if (showComparisonWeights) {
      const weights = parseComparisonWeights();
      if (weights) {
        input.comparisonWeights = weights;
      }
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await calculateMethod<MultiplicityInput, MultiplicityResult>(
        "design.multiplicity",
        input,
      );
      exportMarkdown = await exportMethodMarkdown<MultiplicityInput, MultiplicityResult>(
        "design.multiplicity",
        input,
        result,
      );
      rationale = await fetchCalculationRationale("design.multiplicity", input, result);
      protocolText = await fetchProtocolText("design.multiplicity", input, result);
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "design.multiplicity",
        methodName: "Multiplicity adjustment",
        input,
        result,
      });
    } catch (error) {
      result = null;
      exportMarkdown = null;
      rationale = null;
      protocolText = null;
      lastCalculatedSignature = null;
      errorMessage = String(error);
    } finally {
      calculating = false;
    }
  }
</script>

<MethodPage {resultsStale}>
  {#snippet header()}
    <MethodHeader
      title="Multiplicity adjustment"
      description="Convert a family-wise Type I error rate into a per-comparison alpha for use in endpoint sample size calculations."
      category="Design"
      badges={[methodLabel, `${numberOfComparisons} comparisons`]}
    />
  {/snippet}

  {#snippet parameters()}
    <Panel title="Parameters">
      <Section title="Design">
        <Field label="Family-wise alpha">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.001" bind:value={familyWiseAlpha} />
          {/snippet}
        </Field>

        <Field label={comparisonLabel}>
          {#snippet control()}
            <input type="number" min="1" step="1" bind:value={numberOfComparisons} />
          {/snippet}
        </Field>

        <Field label="Adjustment method">
          {#snippet control()}
            <select bind:value={adjustmentMethod}>
              <option value="bonferroni">Bonferroni</option>
              <option value="sidak">Šidák (Sidak)</option>
              <option value="dunnett">Dunnett (arms vs control)</option>
              <option value="holm">Holm gatekeeping</option>
              <option value="hochberg">Hochberg gatekeeping</option>
              <option value="graphical">Graphical gatekeeping</option>
            </select>
          {/snippet}
        </Field>

        {#if showGatePosition}
          <Field label="Gate position (1 = first hypothesis)">
            {#snippet control()}
              <input type="number" min="1" step="1" bind:value={gatePosition} />
            {/snippet}
          </Field>
        {/if}

        {#if showComparisonWeights}
          <Field label="Comparison weights (comma-separated, optional)">
            {#snippet control()}
              <input
                type="text"
                placeholder="0.5, 0.3, 0.2"
                bind:value={comparisonWeights}
              />
            {/snippet}
          </Field>
        {/if}
      </Section>

      <PrimaryButton fullWidth disabled={calculating} onclick={calculate}>
        {calculating ? "Calculating…" : "Calculate"}
      </PrimaryButton>

      {#if errorMessage}
        <p class="error text-danger">{errorMessage}</p>
      {/if}
    </Panel>
  {/snippet}

  {#snippet results()}
    <Panel title="Results">
      {#if result}
        <ResultHero
          label="Adjusted per-comparison alpha"
          value={result.adjustedAlpha.toFixed(6)}
        />
        <ResultGrid items={resultItems} />
        {#if rationale}
          <RationaleCard text={rationale} />
        {/if}
        {#if protocolText}
          <ProtocolTextCard text={protocolText} />
        {/if}

        <p class="hint">
          Use the adjusted per-comparison alpha as the Type I error input in your endpoint
          calculation (for example, two-sample t-test).
        </p>

        <WarningList warnings={result.warnings} />
        <AssumptionsCard
          items={[
            "Closed testing principle assumed for gatekeeping methods.",
            "Independence or known correlation structure per selected adjustment.",
            "Adjusted alpha feeds endpoint calculations; does not replace them.",
          ]}
        />
        <ExportMenu
          title="Multiplicity adjustment"
          markdown={exportMarkdown}
          disabled={calculating}
        />
      {:else}
        <p class="empty text-muted">Run a calculation to see adjusted alpha.</p>
      {/if}
    </Panel>
  {/snippet}
</MethodPage>

<style>
  .hint {
    margin: 1rem 0 0;
    font-size: 0.8125rem;
    color: var(--text-muted);
    line-height: 1.5;
  }

  .error {
    margin: 0.75rem 0 0;
    font-size: 0.8125rem;
  }

  .empty {
    margin: 0;
    font-size: 0.875rem;
  }
</style>
