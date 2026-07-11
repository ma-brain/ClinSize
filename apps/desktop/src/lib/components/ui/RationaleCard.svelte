<script lang="ts">
  let { text }: { text: string } = $props();

  const paragraphs = $derived(text.split(/\n\n+/).filter((part) => part.trim() !== ""));
</script>

<div class="rationale">
  <h4 class="rationale-title">Sample size calculation rationale</h4>
  {#each paragraphs as paragraph}
    <p>{paragraph}</p>
  {/each}
</div>

<style>
  .rationale {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .rationale-title {
    margin: 0 0 0.5rem;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  p {
    margin: 0 0 0.65rem;
    color: var(--text-primary);
    font-size: 0.8125rem;
    line-height: 1.55;
  }

  p:last-child {
    margin-bottom: 0;
  }
</style>
