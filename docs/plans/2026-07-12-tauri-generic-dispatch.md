# Tauri Generic Dispatch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace per-method Tauri calculation and Markdown-export commands with two generic commands backed by the core dispatcher.

**Architecture:** Tauri accepts JSON values and delegates routing to `clinsize_core::dispatch`. A typed Svelte helper hides the generic transport so existing method views retain typed inputs and results. Sensitivity analysis passes a method id through the same helper.

**Tech Stack:** Rust, Tauri 2, serde_json, Svelte 5, TypeScript, pnpm.

---

### Task 1: Generic Rust boundary

**Files:** Modify `apps/desktop/src-tauri/src/lib.rs`.

1. Write failing tests for generic calculation, Markdown export, and an unsupported id.
2. Run `cargo test -p desktop generic_method --lib`; confirm missing generic functions.
3. Add `calculate_method(method_id, input: Value)` and `export_method_markdown(method_id, input: Value, result: Value)`. Serialize values, delegate to `clinsize_core::dispatch::{calculate_json, report_markdown_json}`, deserialize the result, and map serialization failures to `AppError` code `internal`.
4. Re-run the focused test; confirm it passes.
5. Commit the Rust boundary.

### Task 2: Consolidate handlers

**Files:** Modify `apps/desktop/src-tauri/src/lib.rs`.

1. Extend generic tests to another registered method.
2. Remove all typed `calculate_*` and `export_*_markdown` functions, their imports, and registrations.
3. Register only the generic pair; retain rationale/protocol commands.
4. Run `cargo fmt --check`, `cargo test --workspace`, and `cargo clippy --workspace -- -D warnings`.
5. Commit the handler consolidation.

### Task 3: Typed TypeScript transport

**Files:** Create `apps/desktop/src/lib/workflow/methodDispatch.ts`; modify one representative method view.

1. Change the representative view to import the intended helper; run `pnpm --dir apps/desktop check` and confirm the missing-module failure.
2. Add `calculateMethod<Input, Result>(methodId, input)` and `exportMethodMarkdown<Input, Result>(methodId, input, result)` wrapping the generic Tauri commands.
3. Re-run the type check; confirm it passes.
4. Commit the helper.

### Task 4: Migrate all callers

**Files:** Modify `apps/desktop/src/lib/components/SensitivityPanel.svelte` and every `apps/desktop/src/lib/methods/*View.svelte`.

1. Replace direct per-method invocations with the helpers and stable method ids; remove unused `invoke` imports.
2. Rename sensitivity's `command` prop to `methodId`, call `calculateMethod`, and migrate all sensitivity callers.
3. Run `rg 'calculate_[a-z_]+|export_[a-z_]+_markdown' apps/desktop/src apps/desktop/src-tauri/src/lib.rs`; confirm no per-method command references remain.
4. Run `cargo fmt --check`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and `pnpm --dir apps/desktop check`.
5. Commit the frontend migration.

### Task 5: Review

1. Run `git diff main...HEAD --check` and inspect the stat.
2. Confirm calculation results, Markdown exports, rationale/protocol calls, history, and sensitivity sweeps retain their contracts.
3. Commit this plan document.

