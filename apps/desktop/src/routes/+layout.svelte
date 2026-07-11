<script lang="ts">
  import type { MethodDescriptor } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let { children } = $props();
  let methods = $state<MethodDescriptor[]>([]);

  onMount(async () => {
    methods = await invoke<MethodDescriptor[]>("list_methods");
  });
</script>

<div class="workbench">
  <nav class="rail" aria-label="Method navigation">
    <header class="brand">
      <h1><a href="/">ClinSize</a></h1>
      <p>Sample size and power</p>
    </header>

    <ul class="method-list">
      <li class="workflow-link">
        <a href="/project">Project / history</a>
      </li>
      <li class="workflow-link">
        <a href="/scenarios">Scenarios</a>
      </li>
      <li class="workflow-link">
        <a href="/validation">Validation reports</a>
      </li>
    </ul>

    <ul class="method-list">
      {#each methods as method}
        <li>
          <a href="/methods/{method.id}">{method.displayName}</a>
          <span>{method.endpointCategory}</span>
        </li>
      {/each}
    </ul>
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

  :global(a) {
    color: inherit;
    text-decoration: none;
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

  .method-list {
    list-style: none;
    margin: 1.25rem 0 0;
    padding: 0;
    display: grid;
    gap: 0.5rem;
  }

  .workflow-link a {
    font-size: 0.8125rem;
    font-weight: 600;
  }

  .method-list a {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
  }

  .method-list a:hover {
    color: var(--accent);
  }

  .method-list span {
    display: block;
    font-size: 0.75rem;
    color: var(--muted);
  }

  .content {
    min-width: 0;
  }
</style>
