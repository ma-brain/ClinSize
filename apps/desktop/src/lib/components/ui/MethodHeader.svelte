<script lang="ts">
  let {
    title,
    description,
    category,
    badges = [],
  }: {
    title: string;
    description: string;
    category?: string;
    badges?: string[];
  } = $props();
</script>

<header class="method-header">
  <div class="title-row">
    <h2 class="title">{title}</h2>
    {#if category}
      <span class="category-chip">{category}</span>
    {/if}
  </div>
  {#if badges.length > 0}
    <ul class="badges" aria-label="Method attributes">
      {#each badges as badge}
        <li>{badge}</li>
      {/each}
    </ul>
  {/if}
  <p class="description">{description}</p>
</header>

<style>
  .method-header {
    margin-bottom: 1.25rem;
  }

  .title-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.6rem;
  }

  .title {
    margin: 0;
    font-family: var(--font-display);
    font-size: 1.625rem;
    font-weight: 600;
    line-height: 1.2;
    color: var(--text-primary);
  }

  .category-chip {
    display: inline-flex;
    align-items: center;
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 0.6875rem;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-transform: uppercase;
  }

  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin: 0.6rem 0 0;
    padding: 0;
    list-style: none;
  }

  .badges li {
    padding: 0.2rem 0.5rem;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--bg-panel);
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .description {
    margin: 0.5rem 0 0;
    max-width: 42rem;
    font-size: 0.875rem;
    color: var(--text-muted);
    line-height: 1.55;
  }
</style>
