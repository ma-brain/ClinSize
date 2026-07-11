<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    onclick,
    disabled = false,
    type = "button",
    fullWidth = false,
    children,
  }: {
    onclick?: (event: MouseEvent) => void;
    disabled?: boolean;
    type?: "button" | "submit";
    fullWidth?: boolean;
    children: Snippet;
  } = $props();
</script>

<button {type} class="primary-button" class:full-width={fullWidth} {disabled} {onclick}>
  {@render children()}
</button>

<style>
  .primary-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    padding: 0.55rem 1rem;
    border: none;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: #ffffff;
    font: inherit;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      background var(--transition-fast),
      opacity var(--transition-fast);
  }

  .primary-button:hover:not(:disabled) {
    background: #334fd4;
  }

  .primary-button:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  .primary-button.full-width {
    width: 100%;
  }
</style>
