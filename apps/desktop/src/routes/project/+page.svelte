<script lang="ts">
  import { createProject, projectState, removeCalculation, renameCalculation, setProject, touchProject } from "$lib/stores/project.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import type { ProjectFile } from "$lib/types/project";

  let projectName = $state(projectState.project.name);
  let statusMessage = $state<string | null>(null);

  async function saveProject() {
    const path =
      projectState.filePath ??
      (await save({
        defaultPath: `${projectState.project.name || "clinsize-project"}.clinsize.json`,
        filters: [{ name: "ClinSize project", extensions: ["clinsize.json", "json"] }],
      }));
    if (!path) return;

    await invoke("write_project_file", { path, project: projectState.project });
    projectState.filePath = path;
    projectState.dirty = false;
    statusMessage = `Saved ${path}`;
  }

  async function openProject() {
    const path = await open({
      multiple: false,
      filters: [{ name: "ClinSize project", extensions: ["clinsize.json", "json"] }],
    });
    if (!path || Array.isArray(path)) return;
    const project = await invoke<ProjectFile>("read_project_file", { path });
    setProject(project, path);
    projectName = project.name;
    statusMessage = `Opened ${path}`;
  }

  function newProject() {
    setProject(createProject(projectName.trim() || "Untitled project"));
    statusMessage = "Started a new project.";
  }
</script>

<div class="project-page">
  <header class="page-header">
    <h2>Project and calculation history</h2>
    <p>Review saved calculations, manage the active project file, and prepare scenarios.</p>
  </header>

  <section class="panel toolbar">
    <label>
      Project name
      <input
        bind:value={projectName}
        onchange={() => {
          projectState.project.name = projectName;
          touchProject();
        }}
      />
    </label>
    <div class="actions">
      <button onclick={newProject}>New project</button>
      <button onclick={openProject}>Open project</button>
      <button onclick={saveProject}>Save project</button>
    </div>
    {#if projectState.filePath}
      <p class="meta">File: {projectState.filePath}</p>
    {/if}
    {#if projectState.dirty}
      <p class="meta">Unsaved changes</p>
    {/if}
    {#if statusMessage}
      <p class="meta">{statusMessage}</p>
    {/if}
  </section>

  <section class="panel">
    <h3>Calculation history</h3>
    {#if projectState.project.calculations.length === 0}
      <p class="muted">Run a method calculation to populate history.</p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Label</th>
            <th>Method</th>
            <th>Primary result</th>
            <th>Secondary result</th>
            <th>Created</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each projectState.project.calculations as entry}
            <tr>
              <td>
                <input
                  value={entry.label ?? ""}
                  placeholder={entry.methodName}
                  onchange={(event) =>
                    renameCalculation(entry.id, (event.currentTarget as HTMLInputElement).value)}
                />
              </td>
              <td>{entry.methodName}</td>
              <td>{entry.summary.primaryValue}</td>
              <td>{entry.summary.secondaryValue ?? "—"}</td>
              <td>{entry.createdAt}</td>
              <td>
                <button class="link" onclick={() => removeCalculation(entry.id)}>Remove</button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>
</div>

<style>
  .project-page {
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

  .toolbar {
    display: grid;
    gap: 0.75rem;
  }

  label {
    display: grid;
    gap: 0.25rem;
    font-size: 0.8125rem;
  }

  input {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.35rem 0.5rem;
    background: var(--background);
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  button {
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.45rem 0.75rem;
    background: var(--background);
    cursor: pointer;
    font-size: 0.875rem;
  }

  button.link {
    border: none;
    padding: 0;
    color: var(--accent);
    background: transparent;
  }

  h3 {
    margin: 0 0 0.75rem;
    font-size: 0.9375rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }

  th,
  td {
    border-bottom: 1px solid var(--border);
    padding: 0.5rem 0.35rem;
    text-align: left;
    vertical-align: top;
  }

  .meta,
  .muted {
    color: var(--muted);
    font-size: 0.8125rem;
    margin: 0;
  }
</style>
