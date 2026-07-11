<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    header,
    parameters,
    results,
    resultsStale = false,
  }: {
    header: Snippet;
    parameters: Snippet;
    results: Snippet;
    resultsStale?: boolean;
  } = $props();
</script>

<div class="method-page">
  {@render header()}

  <div class="workspace">
    <div class="parameters-column">
      {@render parameters()}
    </div>

    <aside class="results-column" class:stale={resultsStale} aria-live="polite">
      {#if resultsStale}
        <p class="stale-banner">Inputs changed — recalculate to refresh results.</p>
      {/if}
      {@render results()}
    </aside>
  </div>
</div>

<style>
  .method-page {
    padding: 1.5rem 1.75rem 2rem;
    min-height: 100%;
  }

  .workspace {
    display: grid;
    grid-template-columns: minmax(18rem, 26rem) minmax(22rem, 1fr);
    gap: 1.25rem;
    align-items: start;
  }

  .parameters-column {
    min-width: 0;
  }

  .results-column {
    position: sticky;
    top: 1.25rem;
    min-width: 0;
    transition: opacity var(--transition-fast);
  }

  .results-column.stale {
    opacity: 0.58;
  }

  .stale-banner {
    margin: 0 0 0.75rem;
    padding: 0.45rem 0.65rem;
    border-radius: var(--radius-sm);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 0.75rem;
    font-weight: 500;
  }

  @media (max-width: 960px) {
    .workspace {
      grid-template-columns: 1fr;
    }

    .results-column {
      position: static;
    }
  }
</style>
