<script lang="ts">
  let { items }: { items: string[] } = $props();
</script>

<div class="assumptions">
  <h4 class="assumptions-title">Assumptions</h4>
  <ul>
    {#each items as item}
      <li>{item}</li>
    {/each}
  </ul>
</div>

<style>
  .assumptions {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .assumptions-title {
    margin: 0 0 0.5rem;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-muted);
  }

  ul {
    margin: 0;
    padding-left: 1.1rem;
    color: var(--text-muted);
    font-size: 0.8125rem;
    line-height: 1.5;
  }
</style>
