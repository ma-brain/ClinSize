<script lang="ts">
  import BinaryEffectMeasureView from "$lib/methods/BinaryEffectMeasureView.svelte";
  import LogRankView from "$lib/methods/LogRankView.svelte";
  import MultiplicityView from "$lib/methods/MultiplicityView.svelte";
  import TwoProportionDifferenceView from "$lib/methods/TwoProportionDifferenceView.svelte";
  import AncovaTwoSampleView from "$lib/methods/AncovaTwoSampleView.svelte";
  import OneWayAnovaView from "$lib/methods/OneWayAnovaView.svelte";
  import SingleSampleTTestView from "$lib/methods/SingleSampleTTestView.svelte";
  import TwoSampleTTestView from "$lib/methods/TwoSampleTTestView.svelte";

  let { params } = $props();
  const methodId = $derived(params.id);
</script>

{#if methodId === "continuous.two_sample_ttest"}
  <TwoSampleTTestView />
{:else if methodId === "continuous.one_sample_ttest"}
  <SingleSampleTTestView
    title="One-sample t-test"
    description="Compare one group mean to a known reference value."
    meanDifferenceLabel="Mean difference from reference"
    sizeLabel="Sample size N"
    variant="one_sample"
  />
{:else if methodId === "continuous.paired_ttest"}
  <SingleSampleTTestView
    title="Paired t-test"
    description="Compare paired differences on a continuous endpoint."
    meanDifferenceLabel="Expected mean paired difference"
    sizeLabel="Number of pairs"
    variant="paired"
  />
{:else if methodId === "continuous.one_way_anova"}
  <OneWayAnovaView />
{:else if methodId === "continuous.ancova_two_sample"}
  <AncovaTwoSampleView />
{:else if methodId === "binary.two_proportion_difference"}
  <TwoProportionDifferenceView />
{:else if methodId === "binary.odds_ratio"}
  <BinaryEffectMeasureView
    title="Odds ratio"
    description="Superiority design based on a log odds-ratio normal approximation."
    variant="odds_ratio"
    calculateCommand="calculate_odds_ratio"
    exportCommand="export_odds_ratio_markdown"
    exportFilename="clinsize-odds-ratio.md"
    effectLabel="Odds ratio"
  />
{:else if methodId === "binary.risk_ratio"}
  <BinaryEffectMeasureView
    title="Risk ratio"
    description="Superiority design based on a log risk-ratio normal approximation."
    variant="risk_ratio"
    calculateCommand="calculate_risk_ratio"
    exportCommand="export_risk_ratio_markdown"
    exportFilename="clinsize-risk-ratio.md"
    effectLabel="Risk ratio"
  />
{:else if methodId === "survival.log_rank"}
  <LogRankView />
{:else if methodId === "design.multiplicity"}
  <MultiplicityView />
{:else}
  <p class="muted">Select a method from the navigation rail.</p>
{/if}

<style>
  .muted {
    color: var(--muted);
    font-size: 0.875rem;
    padding: 1.5rem;
  }
</style>
