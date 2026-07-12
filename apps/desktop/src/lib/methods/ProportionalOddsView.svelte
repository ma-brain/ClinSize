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
  import { proportionalOddsSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { calculateMethod, exportMethodMarkdown } from "$lib/workflow/methodDispatch";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    ProportionalOddsInput,
    ProportionalOddsResult,
    SolveMode,
  } from "$lib/types";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let controlN = $state("115");
  let categoryProbabilities = $state("0.2, 0.5, 0.2, 0.1");
  let oddsRatio = $state("2");
  let treatmentFraction = $state("0.5");
  let dropoutRate = $state("");

  let result = $state<ProportionalOddsResult | null>(null);
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
      controlN,
      categoryProbabilities,
      oddsRatio,
      treatmentFraction,
      dropoutRate,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOptions = $derived(
    proportionalOddsSensitivityOptions(
      solveMode,
      categoryProbabilities,
      oddsRatio,
      treatmentFraction,
      alpha,
      power,
      dropoutRate,
    ),
  );

  const solveModeLabel = $derived(
    solveMode === "sample_size" ? "Sample size" : "Power",
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
          { label: "Control N", value: String(result.nControl) },
          { label: "Treatment N", value: String(result.nTreatment) },
          { label: "Total N", value: String(result.totalN) },
          { label: "Achieved power", value: result.achievedPower.toFixed(4) },
          { label: "Efficiency (ps)", value: result.efficiency.toFixed(4) },
          ...(result.nControlAdjusted !== result.nControl
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

  function parseCategoryProbabilities(): number[] {
    return categoryProbabilities
      .split(",")
      .map((value) => value.trim())
      .filter((value) => value.length > 0)
      .map(Number);
  }

  function buildInput(): ProportionalOddsInput {
    const input: ProportionalOddsInput = {
      solveMode,
      alpha: Number(alpha),
      categoryProbabilities: parseCategoryProbabilities(),
      oddsRatio: Number(oddsRatio),
      treatmentFraction: Number(treatmentFraction),
    };

    if (solveMode === "sample_size") {
      input.power = Number(power);
    } else {
      input.controlN = Number(controlN);
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
      result = await calculateMethod<ProportionalOddsInput, ProportionalOddsResult>(
        "ordinal.proportional_odds",
        input,
      );
      exportMarkdown = await exportMethodMarkdown<ProportionalOddsInput, ProportionalOddsResult>(
        "ordinal.proportional_odds",
        input,
        result,
      );
      rationale = await fetchCalculationRationale("ordinal.proportional_odds", input, result);
      protocolText = await fetchProtocolText("ordinal.proportional_odds", input, result);
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "ordinal.proportional_odds",
        methodName: "Proportional odds",
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
      title="Proportional odds"
      description="Two-group ordinal comparison under the proportional odds model (Whitehead 1993)."
      category="Ordinal"
      badges={[solveModeLabel, "Two-sided", "Superiority"]}
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
          <Field label="Control group N">
            {#snippet control()}
              <input type="number" min="2" step="1" bind:value={controlN} />
            {/snippet}
          </Field>
        {/if}

        <Field label="Category probabilities (best → worst, comma-separated)">
          {#snippet control()}
            <input type="text" bind:value={categoryProbabilities} />
          {/snippet}
        </Field>

        <Field label="Odds ratio (> 1)">
          {#snippet control()}
            <input type="number" min="1" step="0.01" bind:value={oddsRatio} />
          {/snippet}
        </Field>

        <Field label="Treatment fraction">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.01" bind:value={treatmentFraction} />
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
            "Proportional odds holds across cumulative response thresholds.",
            "Control-group category probabilities ordered best to worst.",
            "Two-group comparison with specified treatment allocation fraction.",
          ]}
        />
        <ExportMenu title="Proportional odds" markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem="clinsize-sensitivity-proportional-odds"
          inputSignature={lastCalculatedSignature ?? inputSignature}
          methodId="ordinal.proportional_odds"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as ProportionalOddsResult;
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
