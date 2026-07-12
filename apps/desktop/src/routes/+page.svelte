<script lang="ts">
  import type { MethodDescriptor } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let methods = $state<MethodDescriptor[]>([]);

  onMount(async () => {
    methods = await invoke<MethodDescriptor[]>("list_methods");
  });
</script>

<main class="home">
  <section class="panel">
    <h2>ClinSize</h2>
    <p>
      Clinical trial sample size and power workbench — {methods.length} validated
      methods across continuous, binary, count, ordinal, survival, and design
      endpoints, each independently checked against published R references.
    </p>
    <dl class="status">
      <dt>Registered methods</dt>
      <dd>{methods.length}</dd>
    </dl>
    {#if methods.length > 0}
      <ul>
        {#each methods as method}
          <li><a href="/methods/{method.id}">{method.displayName}</a></li>
        {/each}
      </ul>
    {/if}
  </section>
</main>

<style>
  .home {
    padding: 1.5rem;
  }

  .panel {
    max-width: 36rem;
    padding: 1.25rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
  }

  h2 {
    margin: 0 0 0.75rem;
    font-size: 1rem;
    font-weight: 600;
  }

  p {
    margin: 0 0 1rem;
    color: var(--muted);
    line-height: 1.5;
  }

  .status {
    margin: 0 0 1rem;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.25rem 1rem;
    font-size: 0.875rem;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    font-weight: 500;
  }

  ul {
    margin: 0;
    padding-left: 1.1rem;
    font-size: 0.875rem;
  }

  a:hover {
    color: var(--accent);
  }
</style>
