# Tauri Generic Dispatch Design

## Goal

Remove the per-method calculation and Markdown-export commands from the Tauri
shell while preserving the typed input and result models used by each Svelte
view.

## Options considered

1. Keep the typed Tauri handlers and generate them with a macro. This removes
   some Rust repetition but keeps every command name and frontend call site.
2. Use two generic Tauri commands backed by the existing core JSON dispatcher.
   A small typed TypeScript client keeps individual views type-safe. **Chosen.**
3. Generate the full UI from registry schemas. This would remove more wiring,
   but it is a separate product/UI architecture project and would weaken the
   purpose-built method screens.

## Architecture

`calculate_method(methodId, input)` and `export_method_markdown(methodId,
input, result)` will be the only calculation/report Tauri commands. They
accept `serde_json::Value`, serialize to the JSON representation already used
by `clinsize_core::dispatch`, then deserialize calculation results back to a
`Value` for Tauri.

The frontend will expose generic `calculateMethod<Input, Result>` and
`exportMethodMarkdown<Input, Result>` helpers. Each view supplies its stable
method id and retains its existing `Input`/`Result` types. `SensitivityPanel`
will likewise receive a method id rather than a Tauri command name.

## Error handling

Core `Error` values continue to map to the existing structured `AppError`.
JSON serialization failures at the Tauri boundary are returned as internal
errors with the existing error shape. Unsupported ids remain
`unsupported_method` errors from the core dispatcher.

## Verification

Add Rust boundary tests for a representative calculation, report generation,
and unsupported ids. Keep the core registry-dispatch guard. Run Rust tests,
format/lint checks, and Svelte type checking after migrating all views.

