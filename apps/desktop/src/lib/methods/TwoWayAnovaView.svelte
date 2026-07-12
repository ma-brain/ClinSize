<script lang="ts">
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import MethodPage from "$lib/components/MethodPage.svelte";
  import SensitivityPanel from "$lib/components/SensitivityPanel.svelte";
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
  import { twoWayAnovaSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { calculateMethod, exportMethodMarkdown } from "$lib/workflow/methodDispatch";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type { AnovaEffect, SolveMode, TwoWayAnovaInput, TwoWayAnovaResult } from "$lib/types";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let nPerCell = $state("6");
  let nLevelsA = $state("2");
  let nLevelsB = $state("3");
  let primaryEffect = $state<AnovaEffect>("main_a");
  let varianceA = $state("0.5");
  let varianceB = $state("0.5");
  let varianceInteraction = $state("0.5");
  let withinVariance = $state("1");
  let dropoutRate = $state("");

  let result = $state<TwoWayAnovaResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let rationale = $state<string | null>(null);
  let protocolText = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);
  let lastCalculatedSignature = $state<string | null>(null);

  const inputSignature = $derived(
    JSON.stringify({
      solveMode,
      alpha,
      power,
      nPerCell,
      nLevelsA,
      nLevelsB,
      primaryEffect,
      varianceA,
      varianceB,
      varianceInteraction,
      withinVariance,
      dropoutRate,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOptions = $derived(
    twoWayAnovaSensitivityOptions(
      solveMode,
      varianceA,
      varianceB,
      varianceInteraction,
      withinVariance,
      alpha,
      power,
      dropoutRate,
    ),
  );

  const solveModeLabel = $derived(
    solveMode === "sample_size" ? "Sample size" : "Power",
  );

  const primaryEffectLabel = $derived(
    primaryEffect === "main_a"
      ? "Main A"
      : primaryEffect === "main_b"
        ? "Main B"
        : "Interaction",
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Total sample size" : "Achieved power",
  );

  const heroLabel = $derived(
    solveMode === "sample_size" ? "Total sample size" : "Achieved power",
  );

  const heroValue = $derived(
    result
      ? solveMode === "sample_size"
        ? String(result.totalN)
        : result.achievedPower.toFixed(4)
      : "—",
  );

  const resultItems = $derived(
    result
      ? [
          { label: "N per cell", value: String(result.nPerCell) },
          { label: "Total N", value: String(result.totalN) },
          { label: "Achieved power", value: result.achievedPower.toFixed(4) },
          { label: "Effect size (Cohen's f)", value: result.effectSize.toFixed(4) },
          { label: "Primary effect", value: primaryEffectLabel },
          ...(result.nPerCellAdjusted !== result.nPerCell
            ? [
                {
                  label: "Dropout-adjusted total N",
                  value: String(result.totalNAdjusted),
                  highlight: true,
                },
              ]
            : []),
        ]
      : [],
  );

  function buildInput(): TwoWayAnovaInput {
    const input: TwoWayAnovaInput = {
      solveMode,
      alpha: Number(alpha),
      nLevelsA: Number(nLevelsA),
      nLevelsB: Number(nLevelsB),
      primaryEffect,
      varianceA: Number(varianceA),
      varianceB: Number(varianceB),
      varianceInteraction: Number(varianceInteraction),
      withinVariance: Number(withinVariance),
    };

    if (solveMode === "sample_size") {
      input.power = Number(power);
    } else {
      input.nPerCell = Number(nPerCell);
    }

    if (dropoutRate.trim() !== "") {
      input.dropoutRate = Number(dropoutRate);
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await calculateMethod<TwoWayAnovaInput, TwoWayAnovaResult>(
        "continuous.two_way_anova",
        input,
      );
      exportMarkdown = await exportMethodMarkdown<TwoWayAnovaInput, TwoWayAnovaResult>(
        "continuous.two_way_anova",
        input,
        result,
      );
      rationale = await fetchCalculationRationale("continuous.two_way_anova", input, result);
      protocolText = await fetchProtocolText("continuous.two_way_anova", input, result);
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "continuous.two_way_anova",
        methodName: "Two-way ANOVA",
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
      title="Two-way ANOVA"
      description="Balanced two-factor fixed-effects design with main effects and interaction."
      category="Continuous"
      badges={[solveModeLabel, primaryEffectLabel, "Balanced cells"]}
    />
  {/snippet}

  {#snippet parameters()}
    <Panel title="Parameters">
      <Section title="Design">
        <Field label="Solve mode">
          {#snippet control()}
            <select bind:value={solveMode}>
              <option value="sample_size">Sample size</option>
              <option value="power">Power</option>
            </select>
          {/snippet}
        </Field>

        <Field label="Levels of factor A">
          {#snippet control()}
            <input type="number" min="2" step="1" bind:value={nLevelsA} />
          {/snippet}
        </Field>

        <Field label="Levels of factor B">
          {#snippet control()}
            <input type="number" min="2" step="1" bind:value={nLevelsB} />
          {/snippet}
        </Field>

        <Field label="Primary effect">
          {#snippet control()}
            <select bind:value={primaryEffect}>
              <option value="main_a">Main effect A</option>
              <option value="main_b">Main effect B</option>
              <option value="interaction">Interaction (A × B)</option>
            </select>
          {/snippet}
        </Field>

        <Field label="Type I error (alpha)">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.001" bind:value={alpha} />
          {/snippet}
        </Field>

        {#if solveMode === "sample_size"}
          <Field label="Target power">
            {#snippet control()}
              <input type="number" min="0" max="1" step="0.01" bind:value={power} />
            {/snippet}
          </Field>
        {:else}
          <Field label="N per cell">
            {#snippet control()}
              <input type="number" min="2" step="1" bind:value={nPerCell} />
            {/snippet}
          </Field>
        {/if}

        <Field label="Variance A (σ²_A)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={varianceA} />
          {/snippet}
        </Field>

        <Field label="Variance B (σ²_B)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={varianceB} />
          {/snippet}
        </Field>

        <Field label="Variance AB (σ²_AB)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={varianceInteraction} />
          {/snippet}
        </Field>

        <Field label="Within-cell variance (σ²_error)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={withinVariance} />
          {/snippet}
        </Field>
      </Section>

      <Section title="Advanced" collapsible defaultCollapsed={true}>
        <Field label="Dropout rate (optional)">
          {#snippet control()}
            <input type="number" min="0" max="0.99" step="0.01" bind:value={dropoutRate} />
          {/snippet}
        </Field>
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
        <ResultHero label={heroLabel} value={heroValue} />
        <ResultGrid items={resultItems} />
        {#if rationale}
          <RationaleCard text={rationale} />
        {/if}
        {#if protocolText}
          <ProtocolTextCard text={protocolText} />
        {/if}
        <WarningList warnings={result.warnings} />
        <AssumptionsCard
          items={[
            "Balanced allocation with equal N per cell.",
            "Independent observations with approximately normal endpoints.",
            "Homogeneous within-cell variance across all factor combinations.",
            "Power is reported for the selected primary effect only.",
          ]}
        />
        <ExportMenu title="Two-way ANOVA" markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem="clinsize-sensitivity-two-way-anova"
          inputSignature={lastCalculatedSignature ?? inputSignature}
          methodId="continuous.two_way_anova"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as TwoWayAnovaResult;
            return solveMode === "sample_size" ? row.totalN : row.achievedPower;
          }}
          outputLabel={sensitivityOutputLabel}
        />
      {:else}
        <p class="empty text-muted">Enter parameters and calculate to see results.</p>
      {/if}
    </Panel>
  {/snippet}
</MethodPage>

<style>
  .error {
    margin: 0.75rem 0 0;
    font-size: 0.8125rem;
  }

  .empty {
    margin: 0;
    font-size: 0.875rem;
  }
</style>
