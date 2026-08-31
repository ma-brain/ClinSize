<script lang="ts">
  let { text }: { text: string } = $props();

  const paragraphs = $derived(text.split(/\n\n+/).filter((part) => part.trim() !== ""));
</script>

{#each paragraphs as paragraph}
  <p>{paragraph}</p>
{/each}

<style>
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
