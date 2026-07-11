<script lang="ts">
  import "../app.css";
  import NavRail from "$lib/components/NavRail.svelte";
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
  <NavRail {methods} />
  <main class="content">
    {@render children()}
  </main>
</div>

<style>
  .workbench {
    display: grid;
    grid-template-columns: var(--rail-width) 1fr;
    min-height: 100vh;
  }

  .content {
    min-width: 0;
    background: var(--bg-canvas);
  }
</style>
