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
  import { fetchCalculationRationale, fetchProtocolText } from "$lib/workflow/rationale";
  import type {
    GroupSequentialInput,
    GroupSequentialResult,
    SpendingFunction,
  } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let alpha = $state("0.05");
  let targetPower = $state("0.8");
  let numberOfLooks = $state("3");
  let spendingFunction = $state<SpendingFunction>("obrien_fleming");

  let result = $state<GroupSequentialResult | null>(null);
  let exportMarkdown = $state<string | null>(null);
  let rationale = $state<string | null>(null);
  let protocolText = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let calculating = $state(false);
  let lastCalculatedSignature = $state<string | null>(null);

  const inputSignature = $derived(
    JSON.stringify({ alpha, targetPower, numberOfLooks, spendingFunction }),
  );

  const resultsStale = $derived(
    result !== null &&
      lastCalculatedSignature !== null &&
      lastCalculatedSignature !== inputSignature,
  );

  const spendingLabel = $derived(
    spendingFunction === "obrien_fleming" ? "O'Brien-Fleming" : "Pocock",
  );

  const resultItems = $derived(
    result
      ? [
          {
            label: "Sample size inflation factor",
            value: result.sampleSizeInflationFactor.toFixed(4),
          },
          { label: "Achieved power", value: result.achievedPower.toFixed(4) },
          { label: "Fixed-design drift", value: result.fixedDesignDrift.toFixed(4) },
        ]
      : [],
  );

  function buildInput(): GroupSequentialInput {
    return {
      alpha: Number(alpha),
      targetPower: Number(targetPower),
      numberOfLooks: Number(numberOfLooks),
      spendingFunction,
    };
  }

  async function calculate() {
    calculating = true;
    errorMessage = null;

    try {
      const input = buildInput();
      result = await invoke<GroupSequentialResult>("calculate_group_sequential", { input });
      exportMarkdown = await invoke<string>("export_group_sequential_markdown", {
        input,
        result,
      });
      rationale = await fetchCalculationRationale("design.group_sequential", input, result);
      protocolText = await fetchProtocolText("design.group_sequential", input, result);
      lastCalculatedSignature = inputSignature;
      persistCalculation({
        methodId: "design.group_sequential",
        methodName: "Group sequential design",
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
      title="Group sequential design"
      description="Plan interim efficacy boundaries and sample size inflation for equally spaced looks using Lan-DeMets alpha spending."
      category="Design"
      badges={[spendingLabel, `${numberOfLooks} looks`]}
    />
  {/snippet}

  {#snippet parameters()}
    <Panel title="Parameters">
      <Section title="Design">
        <Field label="Two-sided alpha">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.001" bind:value={alpha} />
          {/snippet}
        </Field>

        <Field label="Target power">
          {#snippet control()}
            <input type="number" min="0" max="1" step="0.01" bind:value={targetPower} />
          {/snippet}
        </Field>

        <Field label="Number of looks">
          {#snippet control()}
            <input type="number" min="2" max="10" step="1" bind:value={numberOfLooks} />
          {/snippet}
        </Field>

        <Field label="Spending function">
          {#snippet control()}
            <select bind:value={spendingFunction}>
              <option value="obrien_fleming">O'Brien-Fleming</option>
              <option value="pocock">Pocock</option>
            </select>
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
        <ResultHero
          label="Sample size inflation factor"
          value={result.sampleSizeInflationFactor.toFixed(4)}
        />
        <ResultGrid items={resultItems} />
        {#if rationale}
          <RationaleCard text={rationale} />
        {/if}
        {#if protocolText}
          <ProtocolTextCard text={protocolText} />
        {/if}

        <table class="looks">
          <thead>
            <tr>
              <th>Look</th>
              <th>Info %</th>
              <th>Upper Z</th>
              <th>Cum. α</th>
            </tr>
          </thead>
          <tbody>
            {#each result.looks as look}
              <tr>
                <td>{look.look}</td>
                <td>{(look.informationFraction * 100).toFixed(0)}</td>
                <td>{look.upperZBoundary.toFixed(3)}</td>
                <td>{look.cumulativeAlphaSpent.toFixed(4)}</td>
              </tr>
            {/each}
          </tbody>
        </table>

        <p class="hint">
          Multiply the fixed-design sample size by the inflation factor to obtain the maximum
          sample size under this group sequential plan.
        </p>

        <WarningList warnings={result.warnings} />
        <AssumptionsCard
          items={[
            "Equally spaced interim looks with Lan-DeMets spending approximation.",
            "Fixed treatment effect and known variance for inflation factor planning.",
            "Boundaries apply to a single primary efficacy comparison.",
          ]}
        />
        <ExportMenu
          title="Group sequential design"
          markdown={exportMarkdown}
          disabled={calculating}
        />
      {:else}
        <p class="empty text-muted">Run a calculation to see interim boundaries.</p>
      {/if}
    </Panel>
  {/snippet}
</MethodPage>

<style>
  .looks {
    width: 100%;
    border-collapse: collapse;
    margin-top: 1rem;
    font-size: 0.8125rem;
  }

  .looks th,
  .looks td {
    padding: 0.35rem 0.5rem;
    border-bottom: 1px solid var(--border);
    text-align: left;
  }

  .looks th {
    color: var(--text-muted);
    font-weight: 500;
  }

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
