# Tauri Security Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prevent renderer-controlled native filesystem access and enable a
restrictive CSP without removing user-selected project locations.

**Architecture:** The Rust backend opens native project dialogs, validates and
persists selected project files, and retains the active path in managed state.
The Svelte page only requests open/save actions and receives project content
plus a display name. The frontend filesystem plugin is removed, while the
dialog plugin stays available only to backend commands.

**Tech Stack:** Rust, Tauri v2, `tauri-plugin-dialog`, Svelte 5, SvelteKit.

---

### Task 1: Add project persistence behavior tests

**Files:**
- Modify: `apps/desktop/src-tauri/src/project.rs`

**Step 1: Write failing tests**

Add tests for a `.clinsize.json` path being accepted, a non-project extension
being rejected, and serialization round-tripping through the file helper.

**Step 2: Run the targeted tests to verify failure**

Run: `cargo test -p desktop project::tests -- --nocapture`

Expected: FAIL because project-path validation and file helpers do not exist.

**Step 3: Implement minimal persistence helpers**

Add an internal project-path validator, typed read/write helpers returning
`AppError`-compatible failures, and a `ProjectFileState` wrapper that stores
the active native path behind a mutex. Preserve existing JSON serialization.

**Step 4: Re-run targeted tests**

Run: `cargo test -p desktop project::tests -- --nocapture`

Expected: PASS.

### Task 2: Replace path IPC with backend-owned dialogs

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/project.rs`
- Modify: `apps/desktop/src/routes/project/+page.svelte`

**Step 1: Write failing command-helper tests**

Add tests that saving uses the state-owned path and that creating/opening a
project clears or sets that state without exposing a path to the renderer.

**Step 2: Run the targeted tests to verify failure**

Run: `cargo test -p desktop project::tests -- --nocapture`

Expected: FAIL because no state-owned command helpers exist.

**Step 3: Implement minimal backend command flow**

Replace `read_project_file(path)` and `write_project_file(path, project)` with
commands that use `DialogExt` and backend state. Open returns `None` on
cancellation or a project plus file name after a user selection. Save writes
the active native path or opens a save dialog, then returns a display name.
Register the state in `run()` and remove the raw-path commands.

**Step 4: Update the Svelte workflow**

Remove frontend `open`, `save`, and raw-path handling. Invoke the backend
commands, display only the returned file name, and reset backend state when
creating a new project.

**Step 5: Re-run targeted tests and frontend check**

Run: `cargo test -p desktop project::tests -- --nocapture && cd apps/desktop && pnpm check`

Expected: PASS with no TypeScript errors.

### Task 3: Remove broad frontend filesystem access

**Files:**
- Modify: `apps/desktop/src-tauri/capabilities/default.json`
- Modify: `apps/desktop/src-tauri/Cargo.toml`
- Modify: `apps/desktop/package.json`
- Modify: `apps/desktop/pnpm-lock.yaml`
- Modify: `Cargo.lock`

**Step 1: Write a failing configuration assertion**

Add a Rust test or static configuration test asserting that no project command
accepts a raw path and that filesystem plugin permissions are absent.

**Step 2: Run it to verify failure**

Run the focused test command selected in Task 1.

Expected: FAIL while `fs:allow-write-*` remains configured.

**Step 3: Remove unneeded access**

Remove the frontend filesystem plugin and its permissive capability entries;
keep `dialog:default` only as needed by backend-owned dialogs. Regenerate lock
files using the repository package manager.

**Step 4: Re-run focused checks**

Run: `cargo test -p desktop && cd apps/desktop && pnpm check`

Expected: PASS.

### Task 4: Enable production and development CSPs

**Files:**
- Modify: `apps/desktop/src-tauri/tauri.conf.json`

**Step 1: Write a failing configuration test**

Add a lightweight Rust/static assertion that `csp` is not null and includes
only the bundled app, Tauri IPC, and local asset sources; verify development
includes only the Vite server and HMR websocket.

**Step 2: Run it to verify failure**

Run the focused desktop test command.

Expected: FAIL because `csp` is currently null.

**Step 3: Add restrictive CSP configuration**

Configure `csp` with application, asset, and IPC-only sources; disable frames,
objects, forms, and external network access. Add `devCsp` only for the local
Vite origin and HMR websocket so development remains usable.

**Step 4: Verify build-time compatibility**

Run: `cd apps/desktop && pnpm tauri build --debug`

Expected: successful Tauri build with the CSP accepted by the schema.

### Task 5: Full verification and documentation alignment

**Files:**
- Modify: `TODO.md`
- Modify: `apps/desktop/src-tauri/tauri.conf.json`

**Step 1: Update status wording**

Remove the completed validation-report packaging item from `TODO.md`; retain
the remaining validation-evidence gap and soften the bundle description until
all methods have independent evidence.

**Step 2: Run full verification**

Run: `cargo test --workspace && cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cd apps/desktop && pnpm check`

Expected: all Rust tests, formatting, Clippy, and Svelte checks pass.

**Step 3: Review the final diff**

Run: `git diff --check && git status --short`

Expected: no whitespace errors and only intentional changes.
