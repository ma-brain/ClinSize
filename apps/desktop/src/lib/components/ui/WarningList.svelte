<script lang="ts">
  import type { CalculationWarning } from "$lib/types";

  let { warnings }: { warnings: CalculationWarning[] } = $props();
</script>

{#if warnings.length > 0}
  <div class="warning-list" role="status">
    <h4 class="warning-title">Warnings</h4>
    <ul>
      {#each warnings as warning}
        <li>
          <span class="warning-icon" aria-hidden="true">!</span>
          <span>
            <strong>{warning.code}:</strong>
            {warning.message}
          </span>
        </li>
      {/each}
    </ul>
  </div>
{/if}

<style>
  .warning-list {
    margin-top: 1rem;
    padding: 0.85rem 1rem;
    border: 1px solid #f0d9a8;
    border-radius: var(--radius-sm);
    background: #fffaf0;
  }

  .warning-title {
    margin: 0 0 0.5rem;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--warning);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: 0.45rem;
  }

  li {
    display: flex;
    gap: 0.5rem;
    font-size: 0.8125rem;
    color: var(--text-primary);
    line-height: 1.45;
  }

  .warning-icon {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 1.1rem;
    height: 1.1rem;
    border-radius: 999px;
    background: var(--warning);
    color: #ffffff;
    font-size: 0.6875rem;
    font-weight: 700;
  }
</style>
