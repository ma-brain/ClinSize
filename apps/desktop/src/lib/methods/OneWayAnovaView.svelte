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
  import { oneWayAnovaSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { calculateMethod, exportMethodMarkdown } from "$lib/workflow/methodDispatch";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type { OneWayAnovaInput, OneWayAnovaResult, SolveMode } from "$lib/types";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let nPerGroup = $state("6");
  let nGroups = $state("3");
  let betweenVariance = $state("1");
  let withinSd = $state("1");
  let dropoutRate = $state("");

  let result = $state<OneWayAnovaResult | null>(null);
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
      nPerGroup,
      nGroups,
      betweenVariance,
      withinSd,
      dropoutRate,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOptions = $derived(
    oneWayAnovaSensitivityOptions(
      solveMode,
      betweenVariance,
      withinSd,
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
          { label: "N per group", value: String(result.nPerGroup) },
          { label: "Total N", value: String(result.totalN) },
          { label: "Achieved power", value: result.achievedPower.toFixed(4) },
          { label: "Effect size (Cohen's f)", value: result.effectSize.toFixed(4) },
          ...(result.nPerGroupAdjusted !== result.nPerGroup
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

  function buildInput(): OneWayAnovaInput {
    const withinVariance = Number(withinSd) ** 2;
    const input: OneWayAnovaInput = {
      solveMode,
      alpha: Number(alpha),
      nGroups: Number(nGroups),
      betweenVariance: Number(betweenVariance),
      withinVariance,
    };

    if (solveMode === "sample_size") {
      input.power = Number(power);
    } else {
      input.nPerGroup = Number(nPerGroup);
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
      result = await calculateMethod<OneWayAnovaInput, OneWayAnovaResult>(
        "continuous.one_way_anova",
        input,
      );
      exportMarkdown = await exportMethodMarkdown<OneWayAnovaInput, OneWayAnovaResult>(
        "continuous.one_way_anova",
        input,
        result,
      );
      rationale = await fetchCalculationRationale("continuous.one_way_anova", input, result);
      protocolText = await fetchProtocolText("continuous.one_way_anova", input, result);
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "continuous.one_way_anova",
        methodName: "One-way ANOVA",
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
      title="One-way ANOVA"
      description="Balanced fixed-effect comparison of means across multiple groups."
      category="Continuous"
      badges={[solveModeLabel, "Balanced groups"]}
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

        <Field label="Number of groups">
          {#snippet control()}
            <input type="number" min="2" step="1" bind:value={nGroups} />
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
          <Field label="N per group">
            {#snippet control()}
              <input type="number" min="2" step="1" bind:value={nPerGroup} />
            {/snippet}
          </Field>
        {/if}

        <Field label="Between-group variance">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={betweenVariance} />
          {/snippet}
        </Field>

        <Field label="Within-group SD (σ)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={withinSd} />
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
            "Balanced allocation with equal N per group.",
            "Independent observations with approximately normal endpoints.",
            "Homogeneous within-group variance across arms.",
          ]}
        />
        <ExportMenu title="One-way ANOVA" markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem="clinsize-sensitivity-one-way-anova"
          inputSignature={lastCalculatedSignature ?? inputSignature}
          methodId="continuous.one_way_anova"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as OneWayAnovaResult;
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
