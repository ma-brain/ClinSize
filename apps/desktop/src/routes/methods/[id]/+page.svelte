<script lang="ts">
  import BinaryEffectMeasureView from "$lib/methods/BinaryEffectMeasureView.svelte";
  import BlindedSsreView from "$lib/methods/BlindedSsreView.svelte";
  import GroupSequentialView from "$lib/methods/GroupSequentialView.svelte";
  import LogRankView from "$lib/methods/LogRankView.svelte";
  import MultiplicityView from "$lib/methods/MultiplicityView.svelte";
  import MannWhitneyView from "$lib/methods/MannWhitneyView.svelte";
  import OneSampleBinomialView from "$lib/methods/OneSampleBinomialView.svelte";
  import TwoProportionDifferenceView from "$lib/methods/TwoProportionDifferenceView.svelte";
  import WilcoxonSignedRankView from "$lib/methods/WilcoxonSignedRankView.svelte";
  import AncovaTwoSampleView from "$lib/methods/AncovaTwoSampleView.svelte";
  import ChangeFromBaselineView from "$lib/methods/ChangeFromBaselineView.svelte";
  import MmrmView from "$lib/methods/MmrmView.svelte";
  import NegativeBinomialView from "$lib/methods/NegativeBinomialView.svelte";
  import PoissonView from "$lib/methods/PoissonView.svelte";
  import ProportionalOddsView from "$lib/methods/ProportionalOddsView.svelte";
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
{:else if methodId === "continuous.change_from_baseline"}
  <ChangeFromBaselineView />
{:else if methodId === "continuous.mmrm"}
  <MmrmView />
{:else if methodId === "continuous.mann_whitney"}
  <MannWhitneyView />
{:else if methodId === "continuous.wilcoxon_signed_rank"}
  <WilcoxonSignedRankView />
{:else if methodId === "binary.two_proportion_difference"}
  <TwoProportionDifferenceView />
{:else if methodId === "binary.odds_ratio"}
  <BinaryEffectMeasureView
    title="Odds ratio"
    description="Superiority design based on a log odds-ratio normal approximation."
    variant="odds_ratio"
    exportFilename="clinsize-odds-ratio.md"
    effectLabel="Odds ratio"
  />
{:else if methodId === "binary.one_sample_binomial"}
  <OneSampleBinomialView />
{:else if methodId === "binary.risk_ratio"}
  <BinaryEffectMeasureView
    title="Risk ratio"
    description="Superiority design based on a log risk-ratio normal approximation."
    variant="risk_ratio"
    exportFilename="clinsize-risk-ratio.md"
    effectLabel="Risk ratio"
  />
{:else if methodId === "count.negative_binomial"}
  <NegativeBinomialView />
{:else if methodId === "count.poisson"}
  <PoissonView />
{:else if methodId === "ordinal.proportional_odds"}
  <ProportionalOddsView />
{:else if methodId === "survival.log_rank"}
  <LogRankView />
{:else if methodId === "design.multiplicity"}
  <MultiplicityView />
{:else if methodId === "design.group_sequential"}
  <GroupSequentialView />
{:else if methodId === "design.blinded_ssre"}
  <BlindedSsreView />
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
