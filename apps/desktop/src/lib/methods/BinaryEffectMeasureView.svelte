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
  import { binaryEffectSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    Alternative,
    OddsRatioInput,
    OddsRatioResult,
    RiskRatioInput,
    RiskRatioResult,
    SolveMode,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  type Variant = "odds_ratio" | "risk_ratio";

  let {
    title,
    description,
    variant,
    calculateCommand,
    exportCommand,
    effectLabel,
  }: {
    title: string;
    description: string;
    variant: Variant;
    calculateCommand: string;
    exportCommand: string;
    exportFilename: string;
    effectLabel: string;
  } = $props();

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let controlN = $state("156");
  let controlRate = $state("0.25");
  let treatmentRate = $state("0.4");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let dropoutRate = $state("");

  let oddsResult = $state<OddsRatioResult | null>(null);
  let riskResult = $state<RiskRatioResult | null>(null);
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
      controlN,
      controlRate,
      treatmentRate,
      allocationRatio,
      alternative,
      dropoutRate,
    }),
  );

  const resultsStale = $derived(
    (oddsResult !== null || riskResult !== null) &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOptions = $derived(
    binaryEffectSensitivityOptions(
      solveMode,
      controlRate,
      treatmentRate,
      alpha,
      power,
      allocationRatio,
      dropoutRate,
    ),
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

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size" ? "Total sample size" : "Achieved power",
  );

  const activeResult = $derived(oddsResult ?? riskResult);

  const heroLabel = $derived(
    solveMode === "sample_size" ? "Total sample size" : "Achieved power",
  );

  const heroValue = $derived(
    activeResult
      ? solveMode === "sample_size"
        ? String(activeResult.totalN)
        : activeResult.achievedPower.toFixed(4)
      : "—",
  );

  const resultItems = $derived(
    activeResult
      ? [
          { label: "Control N", value: String(activeResult.nControl) },
          { label: "Treatment N", value: String(activeResult.nTreatment) },
          { label: "Total N", value: String(activeResult.totalN) },
          { label: "Achieved power", value: activeResult.achievedPower.toFixed(4) },
          { label: effectLabel, value: effectValue()?.toFixed(4) ?? "—" },
          ...(activeResult.nControlAdjusted !== activeResult.nControl
            ? [
                {
                  label: "Dropout-adjusted total N",
                  value: String(activeResult.totalNAdjusted),
                  highlight: true,
                },
              ]
            : []),
        ]
      : [],
  );

  function buildOddsInput(): OddsRatioInput {
    const input: OddsRatioInput = {
      solveMode,
      alpha: Number(alpha),
      controlRate: Number(controlRate),
      treatmentRate: Number(treatmentRate),
      allocationRatio: Number(allocationRatio),
      alternative,
    };
    if (solveMode === "sample_size") input.power = Number(power);
    else input.controlN = Number(controlN);
    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);
    return input;
  }

  function buildRiskInput(): RiskRatioInput {
    const input: RiskRatioInput = {
      solveMode,
      alpha: Number(alpha),
      controlRate: Number(controlRate),
      treatmentRate: Number(treatmentRate),
      allocationRatio: Number(allocationRatio),
      alternative,
    };
    if (solveMode === "sample_size") input.power = Number(power);
    else input.controlN = Number(controlN);
    if (dropoutRate.trim() !== "") input.dropoutRate = Number(dropoutRate);
    return input;
  }

  function buildInput(): OddsRatioInput | RiskRatioInput {
    return variant === "odds_ratio" ? buildOddsInput() : buildRiskInput();
  }

  function effectValue(): number | null {
    if (variant === "odds_ratio" && oddsResult) return oddsResult.oddsRatio;
    if (variant === "risk_ratio" && riskResult) return riskResult.riskRatio;
    return null;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;
    oddsResult = null;
    riskResult = null;
    exportMarkdown = null;
    rationale = null;
    protocolText = null;

    try {
      if (variant === "odds_ratio") {
        const input = buildOddsInput();
        oddsResult = await invoke<OddsRatioResult>(calculateCommand, { input });
        exportMarkdown = await invoke<string>(exportCommand, { input, result: oddsResult });
        rationale = await fetchCalculationRationale("binary.odds_ratio", input, oddsResult);
        protocolText = await fetchProtocolText("binary.odds_ratio", input, oddsResult);
        lastCalculatedSignature = inputSignature;
        persistCalculation({
          methodId: "binary.odds_ratio",
          methodName: title,
          input,
          result: oddsResult,
        });
      } else {
        const input = buildRiskInput();
        riskResult = await invoke<RiskRatioResult>(calculateCommand, { input });
        exportMarkdown = await invoke<string>(exportCommand, { input, result: riskResult });
        rationale = await fetchCalculationRationale("binary.risk_ratio", input, riskResult);
        protocolText = await fetchProtocolText("binary.risk_ratio", input, riskResult);
        lastCalculatedSignature = inputSignature;
        persistCalculation({
          methodId: "binary.risk_ratio",
          methodName: title,
          input,
          result: riskResult,
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
              <option value="greater">Greater (treatment &gt; control)</option>
              <option value="less">Less (treatment &lt; control)</option>
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
      {#if activeResult}
        <ResultHero label={heroLabel} value={heroValue} />
        <ResultGrid items={resultItems} />
        {#if rationale}
          <RationaleCard text={rationale} />
        {/if}
        {#if protocolText}
          <ProtocolTextCard text={protocolText} />
        {/if}
        <WarningList warnings={activeResult.warnings} />
        <AssumptionsCard
          items={[
            `Log-${variant === "odds_ratio" ? "odds" : "risk"}-ratio normal approximation.`,
            "Independent observations with fixed follow-up or event ascertainment.",
            "Rates sufficiently away from 0 and 1 for asymptotic validity.",
          ]}
        />
        <ExportMenu {title} markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem={`clinsize-sensitivity-${variant.replace("_", "-")}`}
          inputSignature={lastCalculatedSignature ?? inputSignature}
          command={calculateCommand}
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as OddsRatioResult | RiskRatioResult;
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
