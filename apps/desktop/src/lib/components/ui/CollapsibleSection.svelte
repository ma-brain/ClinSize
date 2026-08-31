<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    defaultCollapsed = false,
    children,
  }: {
    title: string;
    defaultCollapsed?: boolean;
    children: Snippet;
  } = $props();

  let collapsed = $state(defaultCollapsed);
</script>

<section class="collapsible-panel">
  <button
    type="button"
    class="panel-toggle"
    aria-expanded={!collapsed}
    onclick={() => {
      collapsed = !collapsed;
    }}
  >
    <h3 class="panel-title">{title}</h3>
    <span class="chevron" class:open={!collapsed} aria-hidden="true">▾</span>
  </button>

  {#if !collapsed}
    <div class="panel-body">
      {@render children()}
    </div>
  {/if}
</section>

<style>
  .collapsible-panel {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-panel);
    box-shadow: var(--shadow-panel);
    padding: 1.25rem;
  }

  .panel-toggle {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0;
    border: none;
    background: none;
    cursor: pointer;
    text-align: left;
  }

  .panel-title {
    margin: 0;
    font-size: 0.8125rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .chevron {
    flex-shrink: 0;
    color: var(--text-muted);
    font-size: 0.75rem;
    transition: transform var(--transition-fast);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .panel-body {
    margin-top: 1rem;
  }
</style>
