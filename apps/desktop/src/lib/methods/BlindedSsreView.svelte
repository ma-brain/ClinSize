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
    Alternative,
    BlindedSsreInput,
    BlindedSsreResult,
  } from "$lib/types";

  let alpha = $state("0.05");
  let targetPower = $state("0.8");
  let meanDifference = $state("1");
  let plannedStandardDeviation = $state("1");
  let blindedInterimStandardDeviation = $state("");
  let interimFraction = $state("0.5");
  let allocationRatio = $state("1");
  let maxSampleSizeMultiplier = $state("1.5");
  let alternative = $state<Alternative>("two_sided");

  let result = $state<BlindedSsreResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let rationale = $state<string | null>(null);
  let protocolText = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);
  let lastCalculatedSignature = $state<string | null>(null);

  const inputSignature = $derived(
    JSON.stringify({
      alpha,
      targetPower,
      meanDifference,
      plannedStandardDeviation,
      blindedInterimStandardDeviation,
      interimFraction,
      allocationRatio,
      maxSampleSizeMultiplier,
      alternative,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const alternativeLabel = $derived(
    alternative === "two_sided"
      ? "Two-sided"
      : alternative === "greater"
        ? "Greater"
        : "Less",
  );

  const resultItems = $derived(
    result
      ? [
          {
            label: "Planned per-arm N",
            value: `${result.plannedNControl} / ${result.plannedNTreatment}`,
          },
          { label: "Planned total N", value: String(result.plannedTotalN) },
          {
            label: "Interim per-arm N",
            value: `${result.interimNControl} / ${result.interimNTreatment}`,
          },
          { label: "Variance ratio (s_b/σ₀)²", value: result.varianceRatio.toFixed(4) },
          {
            label: "Re-estimated per-arm N",
            value: `${result.reEstimatedNControl} / ${result.reEstimatedNTreatment}`,
          },
          {
            label: "Inflation factor",
            value: result.sampleSizeInflationFactor.toFixed(4),
          },
          {
            label: "Capped per-arm N",
            value: `${result.cappedNControl} / ${result.cappedNTreatment}`,
          },
          { label: "Capped total N", value: String(result.cappedTotalN) },
          {
            label: "Capped inflation factor",
            value: result.cappedInflationFactor.toFixed(4),
          },
          {
            label: "Power at capped N (planned SD)",
            value: result.achievedPowerAtCapped.toFixed(4),
          },
          {
            label: "Power at capped N (interim SD)",
            value: result.achievedPowerAtCappedInterimSd.toFixed(4),
            highlight: result.wasCapped,
          },
        ]
      : [],
  );

  function buildInput(): BlindedSsreInput {
    const input: BlindedSsreInput = {
      alpha: Number(alpha),
      targetPower: Number(targetPower),
      meanDifference: Number(meanDifference),
      plannedStandardDeviation: Number(plannedStandardDeviation),
      interimFraction: Number(interimFraction),
      allocationRatio: Number(allocationRatio),
      maxSampleSizeMultiplier: Number(maxSampleSizeMultiplier),
      alternative,
    };

    const interimSd = blindedInterimStandardDeviation.trim();
    if (interimSd !== "") {
      input.blindedInterimStandardDeviation = Number(interimSd);
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await calculateMethod<BlindedSsreInput, BlindedSsreResult>(
        "design.blinded_ssre",
        input,
      );
      exportMarkdown = await exportMethodMarkdown<BlindedSsreInput, BlindedSsreResult>(
        "design.blinded_ssre",
        input,
        result,
      );
      rationale = await fetchCalculationRationale("design.blinded_ssre", input, result);
      protocolText = await fetchProtocolText("design.blinded_ssre", input, result);
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "design.blinded_ssre",
        methodName: "Blinded sample size re-estimation",
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
      title="Blinded sample size re-estimation"
      description="Plan a two-sample t-test with blinded variance re-estimation at an interim look. Uses the Friede-Kieser rule: update sample size from the blinded pooled interim SD while holding the planned treatment effect fixed."
      category="Design"
      badges={[alternativeLabel, "Friede-Kieser"]}
    />
  {/snippet}

  {#snippet parameters()}
    <Panel title="Parameters">
      <Section title="Design">
        <Field label="Alpha">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.001" bind:value={alpha} />
          {/snippet}
        </Field>

        <Field label="Target power">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.01" bind:value={targetPower} />
          {/snippet}
        </Field>

        <Field label="Mean difference (treatment − control)">
          {#snippet control()}
            <input type="number" step="0.1" bind:value={meanDifference} />
          {/snippet}
        </Field>

        <Field label="Planned standard deviation (σ₀)">
          {#snippet control()}
            <input type="number" min="0" step="0.1" bind:value={plannedStandardDeviation} />
          {/snippet}
        </Field>

        <Field label="Alternative">
          {#snippet control()}
            <select bind:value={alternative}>
              <option value="two_sided">Two-sided</option>
              <option value="greater">Greater (one-sided)</option>
              <option value="less">Less (one-sided)</option>
            </select>
          {/snippet}
        </Field>
      </Section>

      <Section title="Interim re-estimation" collapsible defaultCollapsed={false}>
        <Field label="Blinded interim SD (s_b, optional)" hint="Defaults to planned SD when blank.">
          {#snippet control()}
            <input
              type="number"
              min="0"
              step="0.1"
              bind:value={blindedInterimStandardDeviation}
              placeholder="Defaults to planned SD"
            />
          {/snippet}
        </Field>

        <Field label="Interim fraction (τ)">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.05" bind:value={interimFraction} />
          {/snippet}
        </Field>

        <Field label="Allocation ratio (treatment / control)">
          {#snippet control()}
            <input type="number" min="0" step="0.1" bind:value={allocationRatio} />
          {/snippet}
        </Field>

        <Field label="Maximum sample size multiplier">
          {#snippet control()}
            <input type="number" min="1" step="0.1" bind:value={maxSampleSizeMultiplier} />
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
        <ResultHero label="Capped total N" value={String(result.cappedTotalN)} />
        <ResultGrid items={resultItems} />
        {#if rationale}
          <RationaleCard text={rationale} />
        {/if}
        {#if protocolText}
          <ProtocolTextCard text={protocolText} />
        {/if}

        {#if result.wasCapped}
          <p class="hint">
            Re-estimated sample size exceeded the maximum multiplier and was reduced to the
            pre-specified cap.
          </p>
        {/if}

        <WarningList warnings={result.warnings} />
        <AssumptionsCard
          items={[
            "Blinded pooled interim SD used for variance re-estimation (Friede-Kieser).",
            "Treatment effect held fixed at the planned value.",
            "Two-sample t-test with equal variance and pre-specified maximum inflation cap.",
          ]}
        />
        <ExportMenu
          title="Blinded sample size re-estimation"
          markdown={exportMarkdown}
          disabled={calculating}
        />
      {:else}
        <p class="empty text-muted">Run a calculation to see re-estimated sample sizes.</p>
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
