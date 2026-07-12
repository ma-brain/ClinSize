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
  import { mannWhitneySensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { calculateMethod, exportMethodMarkdown } from "$lib/workflow/methodDispatch";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    Alternative,
    MannWhitneyInput,
    MannWhitneyResult,
    SolveMode,
  } from "$lib/types";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let controlN = $state("131");
  let meanDifference = $state("0.3583");
  let standardDeviation = $state("1");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let dropoutRate = $state("");

  let result = $state<MannWhitneyResult | null>(null);
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
      meanDifference,
      standardDeviation,
      allocationRatio,
      alternative,
      dropoutRate,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOptions = $derived(
    mannWhitneySensitivityOptions(
      solveMode,
      meanDifference,
      standardDeviation,
      alpha,
      power,
      allocationRatio,
      dropoutRate,
    ),
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Total sample size" : "Achieved power",
  );

  const solveModeLabel = $derived(
    solveMode === "sample_size" ? "Sample size" : "Power",
  );

  const alternativeLabel = $derived(
    alternative === "two_sided"
      ? "Two-sided"
      : alternative === "greater"
        ? "Greater"
        : "Less",
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
          {
            label: "P(treatment > control)",
            value: result.probabilitySuperiority.toFixed(4),
          },
          { label: "Effect size (Cohen's d)", value: result.effectSize.toFixed(4) },
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

  function buildInput(): MannWhitneyInput {
    const input: MannWhitneyInput = {
      solveMode,
      alpha: Number(alpha),
      meanDifference: Number(meanDifference),
      standardDeviation: Number(standardDeviation),
      allocationRatio: Number(allocationRatio),
      alternative,
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
      result = await calculateMethod<MannWhitneyInput, MannWhitneyResult>(
        "continuous.mann_whitney",
        input,
      );
      exportMarkdown = await exportMethodMarkdown<MannWhitneyInput, MannWhitneyResult>(
        "continuous.mann_whitney",
        input,
        result,
      );
      rationale = await fetchCalculationRationale(
        "continuous.mann_whitney",
        input,
        result,
      );
      protocolText = await fetchProtocolText(
        "continuous.mann_whitney",
        input,
        result,
      );
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "continuous.mann_whitney",
        methodName: "Mann-Whitney U",
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
      title="Mann-Whitney U"
      description="Nonparametric two-group comparison using Noether (1987) normal approximation."
      category="Continuous"
      badges={[solveModeLabel, alternativeLabel, "Superiority"]}
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

        <Field label="Alternative hypothesis">
          {#snippet control()}
            <select bind:value={alternative}>
              <option value="two_sided">Two-sided</option>
              <option value="greater">Greater</option>
              <option value="less">Less</option>
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
              <input type="number" min="1" step="1" bind:value={controlN} />
            {/snippet}
          </Field>
        {/if}

        <Field label="Mean difference (treatment − control)">
          {#snippet control()}
            <input type="number" step="0.01" bind:value={meanDifference} />
          {/snippet}
        </Field>

        <Field label="Common standard deviation">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={standardDeviation} />
          {/snippet}
        </Field>

        <Field label="Allocation ratio (treatment : control)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={allocationRatio} />
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
            "Continuous endpoint without ties; location shift mapped to P(treatment > control).",
            "Equal within-group variance under normality for planning purposes.",
            "Noether (1987) normal approximation; exact rank-based power is not implemented.",
          ]}
        />
        <ExportMenu title="Mann-Whitney U" markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem="clinsize-sensitivity-mann-whitney"
          inputSignature={lastCalculatedSignature ?? inputSignature}
          methodId="continuous.mann_whitney"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as MannWhitneyResult;
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
