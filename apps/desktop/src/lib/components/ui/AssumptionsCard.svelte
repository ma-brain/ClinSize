<script lang="ts">
  let { items }: { items: string[] } = $props();
</script>

<ul class="assumptions-list">
  {#each items as item}
    <li>{item}</li>
  {/each}
</ul>

<style>
  .assumptions-list {
    margin: 0;
    padding-left: 1.1rem;
    color: var(--text-muted);
    font-size: 0.8125rem;
    line-height: 1.5;
  }
</style>
