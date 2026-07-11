<script lang="ts">
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
