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
  import { oneSampleBinomialSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { calculateMethod, exportMethodMarkdown } from "$lib/workflow/methodDispatch";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    Alternative,
    OneSampleBinomialInput,
    OneSampleBinomialResult,
    SolveMode,
  } from "$lib/types";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let n = $state("50");
  let referenceRate = $state("0.2");
  let responseRate = $state("0.4");
  let alternative = $state<Alternative>("two_sided");
  let dropoutRate = $state("");

  let result = $state<OneSampleBinomialResult | null>(null);
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
      n,
      referenceRate,
      responseRate,
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
    oneSampleBinomialSensitivityOptions(
      solveMode,
      referenceRate,
      responseRate,
      alpha,
      power,
      dropoutRate,
    ),
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Sample size N" : "Achieved power",
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
    solveMode === "sample_size" ? "Sample size N" : "Achieved power",
  );

  const heroValue = $derived(
    result
      ? solveMode === "sample_size"
        ? String(result.n)
        : result.achievedPower.toFixed(4)
      : "—",
  );

  const resultItems = $derived(
    result
      ? [
          { label: "Sample size N", value: String(result.n) },
          { label: "Achieved power", value: result.achievedPower.toFixed(4) },
          { label: "Rate difference", value: result.rateDifference.toFixed(4) },
          ...(result.nAdjusted !== result.n
            ? [
                {
                  label: "Dropout-adjusted N",
                  value: String(result.nAdjusted),
                  highlight: true,
                },
              ]
            : []),
        ]
      : [],
  );

  function buildInput(): OneSampleBinomialInput {
    const input: OneSampleBinomialInput = {
      solveMode,
      alpha: Number(alpha),
      referenceRate: Number(referenceRate),
      responseRate: Number(responseRate),
      alternative,
    };

    if (solveMode === "sample_size") {
      input.power = Number(power);
    } else {
      input.n = Number(n);
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
      result = await calculateMethod<OneSampleBinomialInput, OneSampleBinomialResult>(
        "binary.one_sample_binomial",
        input,
      );
      exportMarkdown = await exportMethodMarkdown<OneSampleBinomialInput, OneSampleBinomialResult>(
        "binary.one_sample_binomial",
        input,
        result,
      );
      rationale = await fetchCalculationRationale(
        "binary.one_sample_binomial",
        input,
        result,
      );
      protocolText = await fetchProtocolText(
        "binary.one_sample_binomial",
        input,
        result,
      );
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "binary.one_sample_binomial",
        methodName: "One-sample binomial",
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
      title="One-sample binomial"
      description="Compare a single-arm response rate to a reference proportion."
      category="Binary"
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
          <Field label="Sample size N">
            {#snippet control()}
              <input type="number" min="1" step="1" bind:value={n} />
            {/snippet}
          </Field>
        {/if}

        <Field label="Reference response rate">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.01" bind:value={referenceRate} />
          {/snippet}
        </Field>

        <Field label="Hypothesized response rate">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.01" bind:value={responseRate} />
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
            "Independent Bernoulli outcomes on a single arm.",
            "Normal approximation to the binomial; exact methods are not implemented.",
            "Higher response rate is favorable for one-sided greater alternatives.",
          ]}
        />
        <ExportMenu title="One-sample binomial" markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem="clinsize-sensitivity-one-sample-binomial"
          inputSignature={lastCalculatedSignature ?? inputSignature}
          methodId="binary.one_sample_binomial"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as OneSampleBinomialResult;
            return solveMode === "sample_size" ? row.n : row.achievedPower;
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
