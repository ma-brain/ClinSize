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
  import { mmrmSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    Alternative,
    CorrelationStructure,
    MmrmInput,
    MmrmResult,
    SolveMode,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let controlN = $state("4");
  let treatmentEffect = $state("2");
  let residualStandardDeviation = $state("2");
  let correlationStructure = $state<CorrelationStructure>("unstructured");
  let correlation = $state("0.5");
  let nPostBaselineVisits = $state("3");
  let perVisitDropoutRate = $state("0.05");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");

  let result = $state<MmrmResult | null>(null);
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
      treatmentEffect,
      residualStandardDeviation,
      correlationStructure,
      correlation,
      nPostBaselineVisits,
      perVisitDropoutRate,
      allocationRatio,
      alternative,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOptions = $derived(
    mmrmSensitivityOptions(
      solveMode,
      treatmentEffect,
      residualStandardDeviation,
      correlation,
      nPostBaselineVisits,
      alpha,
      power,
      allocationRatio,
      perVisitDropoutRate,
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

  const heroLabel = $derived(
    solveMode === "sample_size" ? "Total enrollable N" : "Achieved power",
  );

  const heroValue = $derived(
    result
      ? solveMode === "sample_size"
        ? String(result.totalNAdjusted)
        : result.achievedPower.toFixed(4)
      : "—",
  );

  const resultItems = $derived(
    result
      ? [
          { label: "Control N (evaluable)", value: String(result.nControl) },
          { label: "Treatment N (evaluable)", value: String(result.nTreatment) },
          { label: "Total N (evaluable)", value: String(result.totalN) },
          { label: "Enrollable total N", value: String(result.totalNAdjusted), highlight: true },
          { label: "Achieved power", value: result.achievedPower.toFixed(4) },
          {
            label: "GLS variance efficiency factor",
            value: result.glsVarianceEfficiencyFactor.toFixed(3),
          },
          {
            label: "Cumulative dropout",
            value: `${(result.cumulativeDropout * 100).toFixed(1)}%`,
          },
          { label: "ρ_final", value: result.rhoFinal.toFixed(4) },
          { label: "V_eff", value: result.vEff.toFixed(4) },
        ]
      : [],
  );

  function buildInput(): MmrmInput {
    const input: MmrmInput = {
      solveMode,
      alpha: Number(alpha),
      treatmentEffect: Number(treatmentEffect),
      residualStandardDeviation: Number(residualStandardDeviation),
      correlationStructure,
      correlation: Number(correlation),
      nPostBaselineVisits: Number(nPostBaselineVisits),
      allocationRatio: Number(allocationRatio),
      alternative,
    };

    if (solveMode === "sample_size") {
      input.power = Number(power);
    } else {
      input.controlN = Number(controlN);
    }

    if (perVisitDropoutRate.trim() !== "") {
      input.perVisitDropoutRate = Number(perVisitDropoutRate);
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await invoke<MmrmResult>("calculate_mmrm", { input });
      exportMarkdown = await invoke<string>("export_mmrm_markdown", { input, result });
      rationale = await fetchCalculationRationale("continuous.mmrm", input, result);
      protocolText = await fetchProtocolText("continuous.mmrm", input, result);
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "continuous.mmrm",
        methodName: "MMRM (longitudinal)",
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
      title="MMRM (longitudinal)"
      description="Parallel-group comparison at the final post-baseline visit under a mixed model for repeated measures."
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
          <Field label="Control group N (evaluable)">
            {#snippet control()}
              <input type="number" min="1" step="1" bind:value={controlN} />
            {/snippet}
          </Field>
        {/if}

        <Field label="Treatment effect δ (final visit)">
          {#snippet control()}
            <input type="number" step="0.01" bind:value={treatmentEffect} />
          {/snippet}
        </Field>

        <Field label="Residual standard deviation (σ)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={residualStandardDeviation} />
          {/snippet}
        </Field>
      </Section>

      <Section title="Longitudinal structure">
        <Field label="Correlation structure">
          {#snippet control()}
            <select bind:value={correlationStructure}>
              <option value="unstructured">Unstructured</option>
              <option value="ar1">AR(1)</option>
              <option value="compound_symmetry">Compound symmetry</option>
              <option value="toeplitz">Toeplitz</option>
              <option value="csh">CSH</option>
            </select>
          {/snippet}
        </Field>

        <Field label="Correlation (ρ)">
          {#snippet control()}
            <input type="number" min="-0.99" max="0.99" step="0.01" bind:value={correlation} />
          {/snippet}
        </Field>

        <Field label="Post-baseline visits (k)">
          {#snippet control()}
            <input type="number" min="1" step="1" bind:value={nPostBaselineVisits} />
          {/snippet}
        </Field>
      </Section>

      <Section title="Advanced" collapsible defaultCollapsed={true}>
        <Field label="Allocation ratio (treatment / control)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={allocationRatio} />
          {/snippet}
        </Field>

        <Field label="Per-visit dropout rate (optional)">
          {#snippet control()}
            <input type="number" min="0" max="0.99" step="0.01" bind:value={perVisitDropoutRate} />
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
            "MMRM with visit as a categorical factor.",
            "Simplified single-ρ within-subject correlation parameterization.",
            "Equal residual variance across arms; effect at the final post-baseline visit.",
            "Independent visit-level dropout with constant per-visit rate.",
          ]}
        />
        <ExportMenu title="MMRM (longitudinal)" markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem="clinsize-sensitivity-mmrm"
          inputSignature={lastCalculatedSignature ?? inputSignature}
          command="calculate_mmrm"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as MmrmResult;
            return solveMode === "sample_size" ? row.totalNAdjusted : row.achievedPower;
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
