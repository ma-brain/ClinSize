# ClinSize

Cross-platform clinical trial sample size and power desktop app, built with
SvelteKit, Tauri, and Rust. macOS first; Windows and Linux follow.

Engineering, statistical, and AI-collaboration standards live in
[`handbook/`](handbook/README.md) — read that before making architectural or
statistical decisions. Cursor rules derived from it live in `.cursor/rules/`.

## Layout

```text
apps/desktop/       SvelteKit + Tauri application
crates/clinsize-core/  Pure Rust statistical engine (no Tauri/UI deps)
crates/clinsize-cli/   Thin CLI wrapper around clinsize-core
handbook/            Engineering handbook (source of truth)
validation/          Independent validation evidence, per method
examples/            Example inputs, outputs, and reports
```

## Getting started

Requires Rust (via `rustup`), Node.js, and `pnpm`. See
[`handbook/03-development-environment.md`](handbook/03-development-environment.md)
for the full list.

```bash
just setup   # cargo fetch + pnpm install
just dev     # pnpm tauri dev
just test    # cargo test --workspace + pnpm check
just lint    # cargo fmt --check + cargo clippy -D warnings + pnpm check
just build   # release build (Rust + Tauri installer)
just cli -- --help   # run the CLI wrapper
```

## Status

Phase 2 (continuous endpoints): t-tests, one-way ANOVA, and two-sample ANCOVA are implemented.
Sensitivity analysis views are next. See [`handbook/11-roadmap.md`](handbook/11-roadmap.md).
