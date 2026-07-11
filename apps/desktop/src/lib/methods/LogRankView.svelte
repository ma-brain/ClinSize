<script lang="ts">
  import ExportMenu from "$lib/components/ExportMenu.svelte";
  import MethodPage from "$lib/components/MethodPage.svelte";
  import SensitivityPanel from "$lib/components/SensitivityPanel.svelte";
  import AssumptionsCard from "$lib/components/ui/AssumptionsCard.svelte";
  import Field from "$lib/components/ui/Field.svelte";
  import MethodHeader from "$lib/components/ui/MethodHeader.svelte";
  import Panel from "$lib/components/ui/Panel.svelte";
  import PrimaryButton from "$lib/components/ui/PrimaryButton.svelte";
  import ResultGrid from "$lib/components/ui/ResultGrid.svelte";
  import ResultHero from "$lib/components/ui/ResultHero.svelte";
  import Section from "$lib/components/ui/Section.svelte";
  import WarningList from "$lib/components/ui/WarningList.svelte";
  import { logRankSensitivityOptions } from "$lib/sensitivity/configs";
  import { persistCalculation } from "$lib/workflow/record";
  import type { Alternative, LogRankInput, LogRankResult, SolveMode } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let solveMode = $state<SolveMode>("sample_size");
  let alpha = $state("0.05");
  let power = $state("0.8");
  let totalEvents = $state("66");
  let hazardRatio = $state("0.5");
  let allocationRatio = $state("1");
  let alternative = $state<Alternative>("two_sided");
  let controlHazardRate = $state("0.1155");
  let accrualDuration = $state("12");
  let minimumFollowUp = $state("18");
  let dropoutHazardRate = $state("");
  let includeAccrual = $state(true);

  let result = $state<LogRankResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);
  let lastCalculatedSignature = $state<string | null>(null);

  const inputSignature = $derived(
    JSON.stringify({
      solveMode,
      alpha,
      power,
      totalEvents,
      hazardRatio,
      allocationRatio,
      alternative,
      includeAccrual,
      controlHazardRate,
      accrualDuration,
      minimumFollowUp,
      dropoutHazardRate,
    }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const sensitivityOptions = $derived(
    logRankSensitivityOptions(
      solveMode,
      hazardRatio,
      alpha,
      power,
      allocationRatio,
      includeAccrual,
      controlHazardRate,
      accrualDuration,
      minimumFollowUp,
      dropoutHazardRate,
    ),
  );

  const solveModeLabel = $derived(
    solveMode === "sample_size" ? "Required events" : "Power",
  );

  const alternativeLabel = $derived(
    alternative === "two_sided" ? "Two-sided" : "One-sided",
  );

  const sensitivityOutputLabel = $derived(
    solveMode === "sample_size"
      ? includeAccrual
        ? "Total enrolled subjects"
        : "Required total events"
      : "Achieved power",
  );

  const heroLabel = $derived(
    solveMode === "sample_size"
      ? includeAccrual
        ? "Total enrolled subjects"
        : "Required total events"
      : "Achieved power",
  );

  const heroValue = $derived(
    result
      ? solveMode === "sample_size"
        ? includeAccrual
          ? String(result.totalN ?? result.requiredEvents)
          : String(result.requiredEvents)
        : result.achievedPower.toFixed(4)
      : "—",
  );

  const resultItems = $derived(
    result
      ? [
          { label: "Required total events", value: String(result.requiredEvents) },
          { label: "Expected control events", value: String(result.eventsControl) },
          { label: "Expected treatment events", value: String(result.eventsTreatment) },
          { label: "Achieved power", value: result.achievedPower.toFixed(4) },
          { label: "Hazard ratio", value: result.hazardRatio.toFixed(4) },
          ...(result.totalN
            ? [
                { label: "Control N", value: String(result.nControl) },
                { label: "Treatment N", value: String(result.nTreatment) },
                { label: "Total enrolled subjects", value: String(result.totalN) },
              ]
            : []),
        ]
      : [],
  );

  function buildInput(): LogRankInput {
    const input: LogRankInput = {
      solveMode,
      alpha: Number(alpha),
      hazardRatio: Number(hazardRatio),
      allocationRatio: Number(allocationRatio),
      alternative,
    };

    if (solveMode === "sample_size") input.power = Number(power);
    else input.totalEvents = Number(totalEvents);

    if (includeAccrual) {
      input.controlHazardRate = Number(controlHazardRate);
      input.accrualDuration = Number(accrualDuration);
      input.minimumFollowUp = Number(minimumFollowUp);
      if (dropoutHazardRate.trim() !== "") {
        input.dropoutHazardRate = Number(dropoutHazardRate);
      }
    }

    return input;
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await invoke<LogRankResult>("calculate_log_rank", { input });
      exportMarkdown = await invoke<string>("export_log_rank_markdown", { input, result });
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "survival.log_rank",
        methodName: "Log-rank test",
        input,
        result,
      });
    } catch (error) {
      result = null;
      exportMarkdown = null;
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
      title="Log-rank test"
      description="Two-arm time-to-event design using the Schoenfeld approximation. Provide accrual assumptions to translate required events into enrolled subjects."
      category="Survival"
      badges={[solveModeLabel, alternativeLabel]}
    />
  {/snippet}

  {#snippet parameters()}
    <Panel title="Parameters">
      <Section title="Design">
        <Field label="Solve mode">
          {#snippet control()}
            <select bind:value={solveMode}>
              <option value="sample_size">Required events</option>
              <option value="power">Power</option>
            </select>
          {/snippet}
        </Field>

        <Field label="Alternative hypothesis">
          {#snippet control()}
            <select bind:value={alternative}>
              <option value="two_sided">Two-sided</option>
              <option value="greater">One-sided (treatment hazard &lt; control)</option>
              <option value="less">One-sided (treatment hazard &gt; control)</option>
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
          <Field label="Total events">
            {#snippet control()}
              <input type="number" min="1" step="1" bind:value={totalEvents} />
            {/snippet}
          </Field>
        {/if}

        <Field label="Hazard ratio (treatment / control)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={hazardRatio} />
          {/snippet}
        </Field>

        <Field label="Allocation ratio (treatment / control)">
          {#snippet control()}
            <input type="number" min="0" step="0.01" bind:value={allocationRatio} />
          {/snippet}
        </Field>

        <label class="checkbox-field">
          <input type="checkbox" bind:checked={includeAccrual} />
          <span>Include accrual and follow-up assumptions</span>
        </label>
      </Section>

      {#if includeAccrual}
        <Section title="Accrual" collapsible defaultCollapsed={false}>
          <Field label="Control hazard rate">
            {#snippet control()}
              <input type="number" min="0" step="0.001" bind:value={controlHazardRate} />
            {/snippet}
          </Field>

          <Field label="Accrual duration">
            {#snippet control()}
              <input type="number" min="0" step="0.1" bind:value={accrualDuration} />
            {/snippet}
          </Field>

          <Field label="Minimum follow-up">
            {#snippet control()}
              <input type="number" min="0" step="0.1" bind:value={minimumFollowUp} />
            {/snippet}
          </Field>

          <Field label="Dropout hazard rate (optional)">
            {#snippet control()}
              <input type="number" min="0" step="0.0001" bind:value={dropoutHazardRate} />
            {/snippet}
          </Field>
        </Section>
      {/if}

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
        <WarningList warnings={result.warnings} />
        <AssumptionsCard
          items={[
            "Proportional hazards with exponentially distributed event and censoring times.",
            "Schoenfeld approximation for log-rank test power.",
            "Accrual model uses uniform entry over the accrual period when enabled.",
          ]}
        />
        <ExportMenu title="Log-rank test" markdown={exportMarkdown} />
        <SensitivityPanel
          ready={true}
          defaultExpanded={true}
          chartFileStem="clinsize-sensitivity-log-rank"
          inputSignature={lastCalculatedSignature ?? inputSignature}
          command="calculate_log_rank"
          buildInput={buildInput}
          options={sensitivityOptions}
          getOutputValue={(value) => {
            const row = value as LogRankResult;
            if (solveMode === "sample_size") {
              return includeAccrual ? (row.totalN ?? row.requiredEvents) : row.requiredEvents;
            }
            return row.achievedPower;
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
  .checkbox-field {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.85rem;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
  }

  .checkbox-field input {
    width: auto;
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
