<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    children,
    padding = true,
  }: {
    title?: string;
    children: Snippet;
    padding?: boolean;
  } = $props();
</script>

<section class="panel" class:no-padding={!padding}>
  {#if title}
    <h3 class="panel-title">{title}</h3>
  {/if}
  {@render children()}
</section>

<style>
  .panel {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-panel);
    box-shadow: var(--shadow-panel);
    padding: 1.25rem;
  }

  .panel.no-padding {
    padding: 0;
  }

  .panel-title {
    margin: 0 0 1rem;
    font-size: 0.8125rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--text-muted);
  }
</style>
