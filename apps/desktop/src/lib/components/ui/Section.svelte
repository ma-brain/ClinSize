<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    collapsible = false,
    defaultCollapsed = false,
    children,
  }: {
    title: string;
    collapsible?: boolean;
    defaultCollapsed?: boolean;
    children: Snippet;
  } = $props();

  const initialCollapsed = defaultCollapsed;
  let collapsed = $state(initialCollapsed);
</script>

<div class="section">
  {#if collapsible}
    <button
      type="button"
      class="section-toggle"
      aria-expanded={!collapsed}
      onclick={() => {
        collapsed = !collapsed;
      }}
    >
      <span class="section-title">{title}</span>
      <span class="chevron" class:open={!collapsed} aria-hidden="true">▾</span>
    </button>
  {:else}
    <h4 class="section-title static">{title}</h4>
  {/if}

  {#if !collapsible || !collapsed}
    <div class="section-body">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .section {
    margin-bottom: 1rem;
  }

  .section-toggle {
    display: flex;
    width: 100%;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0;
    border: none;
    background: none;
    cursor: pointer;
    text-align: left;
  }

  .section-title {
    margin: 0;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .section-title.static {
    margin-bottom: 0.75rem;
  }

  .chevron {
    color: var(--text-muted);
    font-size: 0.75rem;
    transition: transform var(--transition-fast);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .section-body {
    margin-top: 0.75rem;
  }
</style>
