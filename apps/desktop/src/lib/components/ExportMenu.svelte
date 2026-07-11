<script lang="ts">
  import { exportCalculationSummary } from "$lib/workflow/export";

  interface Props {
    title: string;
    markdown: string | null;
    disabled?: boolean;
  }

  let { title, markdown, disabled = false }: Props = $props();
  let exporting = $state(false);
  let message = $state<string | null>(null);

  async function runExport(format: "markdown" | "html" | "word" | "pdf") {
    if (!markdown) return;
    exporting = true;
    message = null;
    try {
      await exportCalculationSummary({ markdown, title, format });
      message =
        format === "pdf"
          ? "Saved printable HTML. Open it and use Print to PDF."
          : "Export saved.";
    } catch (error) {
      message = String(error);
    } finally {
      exporting = false;
    }
  }
</script>

<div class="export-menu">
  <button class="secondary" disabled={disabled || exporting || !markdown} onclick={() => runExport("markdown")}>
    Export Markdown
  </button>
  <button class="secondary" disabled={disabled || exporting || !markdown} onclick={() => runExport("html")}>
    Export HTML
  </button>
  <button class="secondary" disabled={disabled || exporting || !markdown} onclick={() => runExport("word")}>
    Export Word
  </button>
  <button class="secondary" disabled={disabled || exporting || !markdown} onclick={() => runExport("pdf")}>
    Export PDF
  </button>
  {#if message}
    <p class="message">{message}</p>
  {/if}
</div>

<style>
  .export-menu {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  button.secondary {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.45rem 0.75rem;
    background: var(--background);
    cursor: pointer;
    font-size: 0.875rem;
  }

  button.secondary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .message {
    width: 100%;
    margin: 0.35rem 0 0;
    font-size: 0.8125rem;
    color: var(--muted);
  }
</style>
