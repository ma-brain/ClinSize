<script lang="ts">
  let { children } = $props();
</script>

<div class="workbench">
  <nav class="rail" aria-label="Method navigation">
    <header class="brand">
      <h1>ClinSize</h1>
      <p>Sample size and power</p>
    </header>
    <p class="rail-note">Methods appear here as they are implemented.</p>
  </nav>

  <div class="content">
    {@render children()}
  </div>
</div>

<style>
  :global(:root) {
    --background: #f6f7f9;
    --panel: #ffffff;
    --border: #d8dce3;
    --text: #1a1d21;
    --muted: #5c6570;
    --accent: #2f6fed;
    --rail-width: 14rem;
  }

  :global(body) {
    margin: 0;
    color: var(--text);
    font-family:
      Inter,
      system-ui,
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      sans-serif;
    background: var(--background);
  }

  .workbench {
    display: grid;
    grid-template-columns: var(--rail-width) 1fr;
    min-height: 100vh;
  }

  .rail {
    border-right: 1px solid var(--border);
    background: var(--panel);
    padding: 1.25rem 1rem;
  }

  .brand h1 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .brand p {
    margin: 0.25rem 0 0;
    font-size: 0.75rem;
    color: var(--muted);
  }

  .rail-note {
    margin: 1.5rem 0 0;
    font-size: 0.8125rem;
    color: var(--muted);
    line-height: 1.4;
  }

  .content {
    min-width: 0;
  }
</style>
