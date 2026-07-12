<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { MethodDescriptor } from "$lib/types";

  let methods = $state<MethodDescriptor[]>([]);
  let selectedMethodId = $state("continuous.two_sample_ttest");
  let report = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let generating = $state(false);

  onMount(async () => {
    methods = await invoke<MethodDescriptor[]>("list_methods");
  });

  async function generateReport() {
    generating = true;
    errorMessage = null;
    report = null;
    try {
      report = await invoke<string>("generate_validation_report", {
        methodId: selectedMethodId,
      });
    } catch (error) {
      errorMessage = String(error);
    } finally {
      generating = false;
    }
  }

  async function saveReport() {
    if (!report) return;
    await invoke<string | null>("save_export_file", {
      exportType: "markdown",
      fileStem: `validation-report-${selectedMethodId.replaceAll(".", "-")}`,
      contents: Array.from(new TextEncoder().encode(report)),
    });
  }
</script>

<div class="validation-page">
  <header class="page-header">
    <h2>Validation reports</h2>
    <p>Generate method validation reports from repository evidence and automated checks.</p>
  </header>

  <section class="panel">
    <label>
      Method
      <select bind:value={selectedMethodId}>
        {#each methods as method}
          <option value={method.id}>{method.displayName}</option>
        {/each}
      </select>
    </label>

    <div class="actions">
      <button onclick={generateReport} disabled={generating}>
        {generating ? "Generating…" : "Generate report"}
      </button>
      <button onclick={saveReport} disabled={!report}>Save report</button>
    </div>

    {#if errorMessage}
      <p class="error">{errorMessage}</p>
    {/if}
  </section>

  {#if report}
    <section class="panel">
      <pre>{report}</pre>
    </section>
  {/if}
</div>

<style>
  .validation-page {
    padding: 1.5rem;
  }

  .page-header h2 {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 600;
  }

  .page-header p {
    margin: 0.35rem 0 0;
    color: var(--muted);
    font-size: 0.875rem;
  }

  .panel {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
    padding: 1rem;
    margin-top: 1rem;
  }

  label {
    display: grid;
    gap: 0.25rem;
    font-size: 0.8125rem;
  }

  select {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    background: var(--background);
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
  }

  button {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.45rem 0.75rem;
    background: var(--background);
    cursor: pointer;
    font-size: 0.875rem;
  }

  pre {
    margin: 0;
    white-space: pre-wrap;
    font-size: 0.8125rem;
  }

  .error {
    color: #9b1c1c;
    font-size: 0.8125rem;
  }
</style>
