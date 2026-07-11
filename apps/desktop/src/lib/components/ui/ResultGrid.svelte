<script lang="ts">
  export interface ResultItem {
    label: string;
    value: string;
    highlight?: boolean;
  }

  let { items }: { items: ResultItem[] } = $props();
</script>

<dl class="result-grid">
  {#each items as item}
    <dt>{item.label}</dt>
    <dd class:highlight={item.highlight} class="tabular">{item.value}</dd>
  {/each}
</dl>

<style>
  .result-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1.25rem;
    margin: 0;
    font-size: 0.875rem;
  }

  dt {
    color: var(--text-muted);
  }

  dd {
    margin: 0;
    font-weight: 500;
    color: var(--text-primary);
  }

  dd.highlight {
    font-weight: 600;
  }
</style>
