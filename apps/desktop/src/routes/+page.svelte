<script lang="ts">
  import type { MethodDescriptor } from "$lib/types";
  import { invoke } from "@tauri-apps/api/core";

  let engineVersion = $state<string | null>(null);
  let methods = $state<MethodDescriptor[]>([]);

  async function loadFoundation() {
    engineVersion = await invoke<string>("engine_info");
    methods = await invoke<MethodDescriptor[]>("list_methods");
  }
</script>

<main class="home">
  <section class="panel">
    <h2>Foundation</h2>
    <p>
      Phase 0 is complete: Rust workspace, statistical engine crate, Tauri command
      boundary, and project tooling are in place. Calculation methods begin in
      Phase 1.
    </p>
    <button onclick={loadFoundation}>Verify engine boundary</button>
    {#if engineVersion}
      <dl class="status">
        <dt>Engine version</dt>
        <dd>{engineVersion}</dd>
        <dt>Registered methods</dt>
        <dd>{methods.length}</dd>
      </dl>
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

  button {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.4rem 0.75rem;
    background: var(--background);
    cursor: pointer;
    font-size: 0.875rem;
  }

  button:hover {
    border-color: var(--accent);
  }

  .status {
    margin: 1rem 0 0;
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
</style>
