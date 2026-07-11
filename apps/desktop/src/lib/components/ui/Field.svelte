<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    hint,
    error,
    control,
  }: {
    label: string;
    hint?: string;
    error?: string | null;
    control: Snippet;
  } = $props();
</script>

<label class="field">
  <span class="field-label">{label}</span>
  {#if hint}
    <span class="field-hint">{hint}</span>
  {/if}
  <span class="field-control">
    {@render control()}
  </span>
  {#if error}
    <span class="field-error" role="alert">{error}</span>
  {/if}
</label>

<style>
  .field {
    display: grid;
    gap: 0.35rem;
    margin-bottom: 0.85rem;
  }

  .field-label {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .field-hint {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-top: -0.15rem;
  }

  .field-control :global(input),
  .field-control :global(select),
  .field-control :global(textarea) {
    width: 100%;
    padding: 0.5rem 0.6rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font: inherit;
    font-size: 0.875rem;
    color: var(--text-primary);
    background: var(--bg-canvas);
    transition: border-color var(--transition-fast);
  }

  .field-control :global(input:hover),
  .field-control :global(select:hover),
  .field-control :global(textarea:hover) {
    border-color: #bcc4d0;
  }

  .field-control :global(input:focus),
  .field-control :global(select:focus),
  .field-control :global(textarea:focus) {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  .field-error {
    font-size: 0.75rem;
    color: var(--danger);
  }
</style>
