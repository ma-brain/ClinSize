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
  import { twoProportionSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { calculateMethod, exportMethodMarkdown } from "$lib/workflow/methodDispatch";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    Alternative,
    SolveMode,
    StudyObjective,
    TwoProportionDifferenceInput,
    TwoProportionDifferenceResult,
  } from "$lib/types";

  let solveMode = $state<SolveMode>("sample_size");
  let studyObjective = $state<StudyObjective>("superiority");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let controlN = $state("163");
  let controlRate = $state("0.3");
  let treatmentRate = $state("0.45");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let noninferiorityMargin = $state("0.1");
  let dropoutRate = $state("");

  let result = $state<TwoProportionDifferenceResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let rationale = $state<string | null>(null);
  let protocolText = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);
  let lastCalculatedSignature = $state<string | null>(null);

  const inputSignature = $derived(
    JSON.stringify({
      solveMode,
      studyObjective,
      alpha,
      power,
      controlN,
      controlRate,
      treatmentRate,
      allocationRatio,
      alternative,
      noninferiorityMargin,
      dropoutRate,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOptions = $derived(
    twoProportionSensitivityOptions(
      solveMode,
      studyObjective,
      controlRate,
      treatmentRate,
      alpha,
      power,
      allocationRatio,
      dropoutRate,
      noninferiorityMargin,
    ),
  );

  const objectiveLabel = $derived(
    studyObjective === "superiority" ? "Superiority" : "Non-inferiority",
  );

  const solveModeLabel = $derived(
    solveMode === "sample_size" ? "Sample size" : "Power",
  );

  const alternativeLabel = $derived(
    studyObjective === "non_inferiority"
      ? "One-sided (greater)"
      : alternative === "two_sided"
        ? "Two-sided"
        : alternative === "greater"
          ? "Greater"
          : "Less",
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
          { label: "Rate difference (treatment − control)", value: result.rateDifference.toFixed(4) },
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

  function buildInput(): TwoProportionDifferenceInput {
    const input: TwoProportionDifferenceInput = {
      solveMode,
      alpha: Number(alpha),
      controlRate: Number(controlRate),
      treatmentRate: Number(treatmentRate),
      allocationRatio: Number(allocationRatio),
      alternative,
      studyObjective,
    };

    if (solveMode === "sample_size") input.power = Number(power);
    else input.controlN = Number(controlN);

    if (studyObjective === "non_inferiority") {
      input.noninferiorityMargin = Number(noninferiorityMargin);
      input.alternative = "greater";
    }

    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await calculateMethod<TwoProportionDifferenceInput, TwoProportionDifferenceResult>(
        "binary.two_proportion_difference",
        input,
      );
      exportMarkdown = await exportMethodMarkdown<
        TwoProportionDifferenceInput,
        TwoProportionDifferenceResult
      >(
        "binary.two_proportion_difference",
        input,
        result,
      );
      rationale = await fetchCalculationRationale(
        "binary.two_proportion_difference",
        input,
        result,
      );
      protocolText = await fetchProtocolText(
        "binary.two_proportion_difference",
        input,
        result,
      );
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "binary.two_proportion_difference",
        methodName: "Difference in proportions",
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
      title="Difference in proportions"
      description="Compare event rates between treatment and control using a normal approximation."
      category="Binary"
      badges={[objectiveLabel, solveModeLabel, alternativeLabel]}
    />
  {/snippet}

  {#snippet parameters()}
    <Panel title="Parameters">
      <Section title="Design">
        <Field label="Study objective">
          {#snippet control()}
            <select bind:value={studyObjective}>
              <option value="superiority">Superiority</option>
              <option value="non_inferiority">Non-inferiority</option>
            </select>
          {/snippet}
        </Field>

        <Field label="Solve mode">
          {#snippet control()}
            <select bind:value={solveMode}>
              <option value="sample_size">Sample size</option>
              <option value="power">Power</option>
            </select>
          {/snippet}
        </Field>

        {#if studyObjective === "superiority"}
          <Field label="Alternative hypothesis">
            {#snippet control()}
              <select bind:value={alternative}>
                <option value="two_sided">Two-sided</option>
                <option value="greater">Greater (treatment &gt; control)</option>
                <option value="less">Less (treatment &lt; control)</option>
              </select>
            {/snippet}
          </Field>
        {/if}

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

        <Field label="Control event rate">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.01" bind:value={controlRate} />
          {/snippet}
        </Field>

        <Field label="Treatment event rate">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.01" bind:value={treatmentRate} />
          {/snippet}
        </Field>

        {#if studyObjective === "non_inferiority"}
          <Field label="Non-inferiority margin (max acceptable deficit)">
            {#snippet control()}
              <input type="number" min="0" max="0.99" step="0.01" bind:value={noninferiorityMargin} />
            {/snippet}
          </Field>
        {/if}
      </Section>

      <Section title="Advanced" collapsible defaultCollapsed={true}>
        <Field label="Allocation ratio (treatment / control)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={allocationRatio} />
          {/snippet}
        </Field>

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
            "Normal approximation to the difference in binomial proportions.",
            "Independent observations with fixed follow-up or event ascertainment.",
            "Rates sufficiently away from 0 and 1 for asymptotic validity.",
          ]}
        />
        <ExportMenu title="Difference in proportions" markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem="clinsize-sensitivity-two-proportion-difference"
          inputSignature={lastCalculatedSignature ?? inputSignature}
          methodId="binary.two_proportion_difference"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as TwoProportionDifferenceResult;
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
