<script lang="ts">
  import type { CalculationWarning } from "$lib/types";

  let { warnings }: { warnings: CalculationWarning[] } = $props();
</script>

{#if warnings.length > 0}
  <ul class="notes-list">
    {#each warnings as warning}
      <li>
        <strong>{warning.code}:</strong>
        {warning.message}
      </li>
    {/each}
  </ul>
{/if}

<style>
  .notes-list {
    margin: 0;
    padding-left: 1.1rem;
    display: grid;
    gap: 0.45rem;
    color: var(--text-muted);
    font-size: 0.8125rem;
    line-height: 1.5;
  }
</style>
