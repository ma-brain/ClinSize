# ClinSize Engineering Handbook

This handbook defines the engineering, statistical, validation, and AI collaboration standards for a cross-platform clinical trial sample size and power application built with Cursor, SvelteKit, Tauri, and Rust.

The working product name used throughout this handbook is **ClinSize**. Rename it later if needed; the architecture and standards do not depend on the name.

## Purpose

ClinSize is intended to be a professional desktop application for clinical trial design calculations. The first product scope is sample size and power calculations for fixed methods such as t-tests, ANOVA, binary endpoints, survival endpoints, non-inferiority, and equivalence.

The handbook has four goals:

1. Give human developers a clear project reference.
2. Give Cursor and other AI assistants stable instructions.
3. Keep statistical methods traceable to formulas, references, assumptions, and tests.
4. Separate the validated calculation engine from the desktop user interface.

## Recommended Repository Shape

```text
clinsize/
  apps/
    desktop/               # Tauri + SvelteKit application
  crates/
    clinsize-core/          # Pure Rust statistical engine
    clinsize-cli/          # Optional command line wrapper
  handbook/                # This handbook
  validation/              # Independent validation evidence
  examples/                # Example inputs, outputs, and reports
```

## Core Principles

- Keep statistical algorithms in a pure Rust core crate with no Tauri dependency.
- Treat every calculation method as a validated module with documented assumptions.
- Prefer explicit, typed inputs and outputs over loosely structured maps.
- Make UI behavior predictable, dense, and suited to repeated professional use.
- Verify numerical results against published examples and independent software.
- Use AI assistants for acceleration, but require human review for formulas, validation, and regulated-use claims.

## How To Use This Handbook In Cursor

The non-negotiable rules, Rust engine standards, Svelte/UI standards, and
statistics-review checklist are wired into `.cursor/rules/*.mdc` at the
project root, so Cursor loads them automatically based on which files are
open (see that directory for the scoping). This handbook is still the
canonical, human-readable version — update it first when a rule changes,
then reflect the change in the matching `.cursor/rules/*.mdc` file.

For work that isn't covered by an auto-attached rule, include the relevant
files in Cursor context manually:

- Project structure: `../README.md`, `02-architecture.md`
- Rust work: `04-rust-engine-standards.md`, `07-statistical-methodology.md`
- UI work: `05-svelte-tauri-standards.md`, `06-ui-ux-guidelines.md`
- Tests: `08-validation-testing.md`
- New methods: `templates/new-calculation-method.md`

Cursor should treat the handbook as the source of truth unless a newer project decision record supersedes it.

