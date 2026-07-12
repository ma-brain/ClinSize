# Tauri Export Security Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Remove renderer access to native filesystem paths while preserving project, report, calculation-export, and chart-export workflows.

**Architecture:** Rust commands own every native save dialog and write the selected file after receiving only content and export metadata from the Svelte renderer. The renderer never receives a native path and only treats cancellation as a no-op. Once all export flows use those commands, remove the filesystem and dialog JavaScript plugins and apply the restrictive CSP.

**Tech Stack:** Rust, Tauri v2, `tauri-plugin-dialog`, Svelte 5, SvelteKit.

---

### Task 1: Add tested export persistence helpers

**Files:**
- Create: `apps/desktop/src-tauri/src/export_file.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

1. Write focused tests for writing text and binary export bytes and returning only the selected filename.
2. Run `cargo test -p desktop export_file::tests -- --nocapture` and confirm the helpers are missing.
3. Add typed export errors, byte-writing helpers, and filename-only responses.
4. Re-run the focused test command and confirm it passes.

### Task 2: Move all save dialogs to Rust commands

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/lib/workflow/export.ts`
- Modify: `apps/desktop/src/lib/workflow/chartExport.ts`
- Modify: `apps/desktop/src/routes/validation/+page.svelte`

1. Add command helpers that select a save location through `DialogExt`, write renderer-provided bytes, and return `None` on cancellation.
2. Change calculation exports, chart exports, and validation-report saves to invoke those commands without importing Tauri JavaScript dialog or filesystem APIs.
3. Run `cargo test -p desktop` and `pnpm check`.

### Task 3: Remove renderer filesystem authority

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/capabilities/default.json`
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/package.json`
- Modify: `apps/desktop/pnpm-lock.yaml`
- Modify: `Cargo.lock`

1. Add a static test asserting no filesystem plugin registration, filesystem capability permission, or raw project path command remains.
2. Confirm the test fails while the old plugin configuration exists.
3. Remove the filesystem plugin, JavaScript dialog plugin, and their capability permissions; retain only the Rust dialog plugin.
4. Regenerate lockfiles and re-run the focused tests and Svelte check.

### Task 4: Enable and verify CSP

**Files:**
- Modify: `apps/desktop/src-tauri/tauri.conf.json`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

1. Add a static configuration test for non-null production CSP and a local-only development CSP.
2. Confirm it fails against the null CSP.
3. Add a restrictive application CSP with no external network, frames, objects, or forms, plus only the Vite/HMR development origins.
4. Run `pnpm tauri build --debug` to validate the Tauri schema and bundle configuration.

### Task 5: Verify and align status documentation

**Files:**
- Modify: `TODO.md`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`

1. Align validation-evidence wording with the current implementation state.
2. Run `cargo test --workspace`, `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `pnpm check`, and `git diff --check`.
