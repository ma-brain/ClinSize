<script lang="ts">
  let {
    label,
    value,
  }: {
    label: string;
    value: string;
  } = $props();
</script>

<div class="result-hero">
  <span class="label">{label}</span>
  <span class="value tabular">{value}</span>
</div>

<style>
  .result-hero {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1.25rem;
    padding: 1rem 1.1rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: linear-gradient(180deg, var(--bg-panel) 0%, var(--bg-subtle) 100%);
  }

  .label {
    font-size: 1rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  .value {
    flex-shrink: 0;
    font-family: var(--font-display);
    font-size: 1.2rem;
    font-weight: 600;
    line-height: 1.1;
    color: var(--text-primary);
    text-align: right;
  }
</style>
