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
  import {
    oneSampleSensitivityOptions,
    pairedSensitivityOptions,
  } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { calculateMethod, exportMethodMarkdown } from "$lib/workflow/methodDispatch";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    Alternative,
    OneSampleTTestInput,
    OneSampleTTestResult,
    PairedTTestInput,
    PairedTTestResult,
    SolveMode,
  } from "$lib/types";

  type Variant = "one_sample" | "paired";

  let {
    title,
    description,
    meanDifferenceLabel,
    sizeLabel,
    variant,
  }: {
    title: string;
    description: string;
    meanDifferenceLabel: string;
    sizeLabel: string;
    variant: Variant;
  } = $props();

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let size = $state("10");
  let meanDifference = $state("1");
  let standardDeviation = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let dropoutRate = $state("");

  let oneSampleResult = $state<OneSampleTTestResult | null>(null);
  let pairedResult = $state<PairedTTestResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let rationale = $state<string | null>(null);
  let protocolText = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);
  let lastCalculatedSignature = $state<string | null>(null);

  const inputSignature = $derived(
    JSON.stringify({
      variant,
      solveMode,
      alpha,
      power,
      size,
      meanDifference,
      standardDeviation,
      alternative,
      dropoutRate,
    }),
  );

  const resultsStale = $derived(
    (oneSampleResult !== null || pairedResult !== null) &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? sizeLabel : "Achieved power",
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

  const oneSampleSensitivity = $derived(
    oneSampleSensitivityOptions(
      solveMode,
      meanDifference,
      standardDeviation,
      alpha,
      power,
      dropoutRate,
    ),
  );

  const pairedSensitivity = $derived(
    pairedSensitivityOptions(
      solveMode,
      meanDifference,
      standardDeviation,
      alpha,
      power,
      dropoutRate,
    ),
  );

  const heroLabel = $derived(
    solveMode === "sample_size" ? sizeLabel : "Achieved power",
  );

  const heroValue = $derived(
    oneSampleResult
      ? solveMode === "sample_size"
        ? String(oneSampleResult.n)
        : oneSampleResult.achievedPower.toFixed(4)
      : pairedResult
        ? solveMode === "sample_size"
          ? String(pairedResult.nPairs)
          : pairedResult.achievedPower.toFixed(4)
        : "—",
  );

  const resultItems = $derived(
    oneSampleResult
      ? [
          { label: sizeLabel, value: String(oneSampleResult.n) },
          { label: "Achieved power", value: oneSampleResult.achievedPower.toFixed(4) },
          { label: "Effect size (Cohen's d)", value: oneSampleResult.effectSize.toFixed(4) },
          ...(oneSampleResult.nAdjusted !== oneSampleResult.n
            ? [
                {
                  label: "Dropout-adjusted N",
                  value: String(oneSampleResult.nAdjusted),
                  highlight: true,
                },
              ]
            : []),
        ]
      : pairedResult
        ? [
            { label: sizeLabel, value: String(pairedResult.nPairs) },
            { label: "Achieved power", value: pairedResult.achievedPower.toFixed(4) },
            { label: "Effect size (Cohen's d)", value: pairedResult.effectSize.toFixed(4) },
            ...(pairedResult.nPairsAdjusted !== pairedResult.nPairs
              ? [
                  {
                    label: "Dropout-adjusted pairs",
                    value: String(pairedResult.nPairsAdjusted),
                    highlight: true,
                  },
                ]
              : []),
          ]
        : [],
  );

  const activeWarnings = $derived(
    oneSampleResult?.warnings ?? pairedResult?.warnings ?? [],
  );

  function buildOneSampleInput(): OneSampleTTestInput {
    const input: OneSampleTTestInput = {
      solveMode,
      alpha: Number(alpha),
      meanDifference: Number(meanDifference),
      standardDeviation: Number(standardDeviation),
      alternative,
    };
    if (solveMode === "sample_size") input.power = Number(power);
    else input.n = Number(size);
    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);
    return input;
  }

  function buildPairedInput(): PairedTTestInput {
    const input: PairedTTestInput = {
      solveMode,
      alpha: Number(alpha),
      meanDifference: Number(meanDifference),
      standardDeviation: Number(standardDeviation),
      alternative,
    };
    if (solveMode === "sample_size") input.power = Number(power);
    else input.nPairs = Number(size);
    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);
    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;
    oneSampleResult = null;
    pairedResult = null;
    exportMarkdown = null;
    rationale = null;
    protocolText = null;

    try {
      if (variant === "one_sample") {
        const input = buildOneSampleInput();
        oneSampleResult = await calculateMethod<OneSampleTTestInput, OneSampleTTestResult>(
          "continuous.one_sample_ttest",
          input,
        );
        exportMarkdown = await exportMethodMarkdown<OneSampleTTestInput, OneSampleTTestResult>(
          "continuous.one_sample_ttest",
          input,
          oneSampleResult,
        );
        rationale = await fetchCalculationRationale(
          "continuous.one_sample_ttest",
          input,
          oneSampleResult,
        );
        protocolText = await fetchProtocolText(
          "continuous.one_sample_ttest",
          input,
          oneSampleResult,
        );
        lastCalculatedSignature = inputSignature;
        persistCalculation({
          methodId: "continuous.one_sample_ttest",
          methodName: title,
          input,
          result: oneSampleResult,
        });
      } else {
        const input = buildPairedInput();
        pairedResult = await calculateMethod<PairedTTestInput, PairedTTestResult>(
          "continuous.paired_ttest",
          input,
        );
        exportMarkdown = await exportMethodMarkdown<PairedTTestInput, PairedTTestResult>(
          "continuous.paired_ttest",
          input,
          pairedResult,
        );
        rationale = await fetchCalculationRationale(
          "continuous.paired_ttest",
          input,
          pairedResult,
        );
        protocolText = await fetchProtocolText(
          "continuous.paired_ttest",
          input,
          pairedResult,
        );
        lastCalculatedSignature = inputSignature;
        persistCalculation({
          methodId: "continuous.paired_ttest",
          methodName: title,
          input,
          result: pairedResult,
        });
      }
    } catch (error) {
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
      {title}
      {description}
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
          <Field label={sizeLabel}>
            {#snippet control()}
              <input type="number" min="2" step="1" bind:value={size} />
            {/snippet}
          </Field>
        {/if}

        <Field label={meanDifferenceLabel}>
          {#snippet control()}
            <input type="number" step="0.01" bind:value={meanDifference} />
          {/snippet}
        </Field>

        <Field label="Standard deviation">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={standardDeviation} />
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
      {#if oneSampleResult || pairedResult}
        <ResultHero label={heroLabel} value={heroValue} />
        <ResultGrid items={resultItems} />
        {#if rationale}
          <RationaleCard text={rationale} />
        {/if}
        {#if protocolText}
          <ProtocolTextCard text={protocolText} />
        {/if}
        <WarningList warnings={activeWarnings} />
        <AssumptionsCard
          items={variant === "one_sample"
            ? [
                "Independent observations with approximately normal endpoints.",
                "Known or estimated standard deviation held fixed for planning.",
                "Superiority design; use adjusted alpha from multiplicity tools when applicable.",
              ]
            : [
                "Paired differences are approximately normally distributed.",
                "Independence across subject pairs; correlation structure captured by paired SD.",
                "Superiority design; use adjusted alpha from multiplicity tools when applicable.",
              ]}
        />
        <ExportMenu {title} markdown={exportMarkdown} />
        {#if oneSampleResult}
          <SensitivityPanel
            ready={true}
            defaultExpanded={true}
            chartFileStem="clinsize-sensitivity-one-sample-ttest"
            inputSignature={lastCalculatedSignature ?? inputSignature}
            methodId="continuous.one_sample_ttest"
            buildInput={buildOneSampleInput}
            options={oneSampleSensitivity}
            getOutputValue={(value) => {
              const row = value as OneSampleTTestResult;
              return solveMode === "sample_size" ? row.n : row.achievedPower;
            }}
            outputLabel={sensitivityOutputLabel}
          />
        {:else if pairedResult}
          <SensitivityPanel
            ready={true}
            defaultExpanded={true}
            chartFileStem="clinsize-sensitivity-paired-ttest"
            inputSignature={lastCalculatedSignature ?? inputSignature}
            methodId="continuous.paired_ttest"
            buildInput={buildPairedInput}
            options={pairedSensitivity}
            getOutputValue={(value) => {
              const row = value as PairedTTestResult;
              return solveMode === "sample_size" ? row.nPairs : row.achievedPower;
            }}
            outputLabel={sensitivityOutputLabel}
          />
        {/if}
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
