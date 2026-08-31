<script lang="ts">
  import { exportCalculationSummary } from "$lib/workflow/export";

  interface Props {
    title: string;
    markdown: string | null;
    disabled?: boolean;
  }

  let { title, markdown, disabled = false }: Props = $props();

  let open = $state(false);
  let exporting = $state(false);
  let message = $state<string | null>(null);
  let containerEl: HTMLDivElement | null = null;
  let messageTimeout: ReturnType<typeof setTimeout> | null = null;

  const formats = [
    { id: "markdown", label: "Markdown" },
    { id: "html", label: "HTML" },
    { id: "word", label: "Word" },
    { id: "pdf", label: "PDF" },
  ] as const;

  function showMessage(text: string) {
    message = text;
    if (messageTimeout) clearTimeout(messageTimeout);
    messageTimeout = setTimeout(() => {
      message = null;
    }, 4000);
  }

  async function runExport(format: "markdown" | "html" | "word" | "pdf") {
    if (!markdown) return;
    open = false;
    exporting = true;
    message = null;
    try {
      await exportCalculationSummary({ markdown, title, format });
      showMessage(
        format === "pdf"
          ? "PDF saved."
          : format === "word"
            ? "Word document saved."
            : "Export saved.",
      );
    } catch (error) {
      showMessage(String(error));
    } finally {
      exporting = false;
    }
  }

  function handleWindowClick(event: MouseEvent) {
    if (open && containerEl && !containerEl.contains(event.target as Node)) {
      open = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") open = false;
  }
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleKeydown} />

<div class="export-menu" bind:this={containerEl}>
  <button
    type="button"
    class="trigger"
    disabled={disabled || exporting || !markdown}
    aria-haspopup="true"
    aria-expanded={open}
    onclick={() => {
      open = !open;
    }}
  >
    {exporting ? "Exporting…" : "Export"}
    <span class="chevron" class:open aria-hidden="true">▾</span>
  </button>

  {#if open}
    <ul class="menu" role="menu">
      {#each formats as format}
        <li role="none">
          <button type="button" role="menuitem" onclick={() => runExport(format.id)}>
            {format.label}
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if message}
    <p class="message" role="status">{message}</p>
  {/if}
</div>

<style>
  .export-menu {
    position: relative;
    flex-shrink: 0;
  }

  .trigger {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    border: 1px solid var(--accent-soft);
    border-radius: var(--radius-sm);
    padding: 0.4rem 0.7rem;
    background: var(--accent-soft);
    color: var(--accent);
    cursor: pointer;
    font: inherit;
    font-size: 0.8125rem;
    font-weight: 600;
    white-space: nowrap;
  }

  .trigger:hover:not(:disabled) {
    border-color: var(--accent);
  }

  .trigger:disabled {
    border-color: var(--border);
    background: var(--bg-panel);
    color: var(--text-primary);
    opacity: 0.55;
    cursor: not-allowed;
  }

  .chevron {
    font-size: 0.7rem;
    color: var(--text-muted);
    transition: transform var(--transition-fast);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .menu {
    position: absolute;
    z-index: 20;
    top: calc(100% + 0.35rem);
    right: 0;
    display: flex;
    flex-direction: column;
    min-width: 9rem;
    margin: 0;
    padding: 0.3rem;
    list-style: none;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-panel);
    box-shadow: var(--shadow-panel);
  }

  .menu button {
    display: block;
    width: 100%;
    border: none;
    border-radius: var(--radius-sm);
    padding: 0.4rem 0.55rem;
    background: none;
    color: var(--text-primary);
    cursor: pointer;
    font: inherit;
    font-size: 0.8125rem;
    text-align: left;
  }

  .menu button:hover {
    background: var(--accent-soft);
    color: var(--accent);
  }

  .message {
    position: absolute;
    z-index: 20;
    top: calc(100% + 0.35rem);
    right: 0;
    margin: 0;
    padding: 0.4rem 0.6rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-panel);
    box-shadow: var(--shadow-panel);
    color: var(--text-muted);
    font-size: 0.75rem;
    white-space: nowrap;
  }
</style>
