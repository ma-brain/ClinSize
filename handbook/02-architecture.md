# Architecture

## Architectural Goal

The architecture must keep the statistical engine independent from the desktop shell. The calculation engine should be usable from:

- A Tauri desktop app.
- A command line interface.
- Tests and validation scripts.
- Future WebAssembly, mobile, R, or Python bindings.

## Recommended Workspace Layout

```text
clinsize/
  Cargo.toml
  apps/
    desktop/
      package.json
      src/
      src-tauri/
  crates/
    clinsize-core/
      src/
        methods/
        distributions/
        validation/
        reports/
    clinsize-cli/
      src/
  handbook/
  validation/
  examples/
```

## Main Components

`clinsize-core` contains pure statistical logic. It should not depend on Svelte, Tauri, operating system APIs, or UI concerns.

`apps/desktop` contains the SvelteKit frontend and Tauri command bindings. It can depend on `clinsize-core`.

`clinsize-cli` is optional but recommended. It gives a simple way to run calculations in scripts and makes validation easier.

`validation` contains independent reference cases, comparison outputs, and validation reports.

## Data Flow

```text
Svelte form
  -> TypeScript input model
  -> Tauri command
  -> Rust request DTO
  -> clinsize-core calculation
  -> Rust result DTO
  -> Svelte result view
  -> report/export layer
```

The UI should never implement statistical formulas. It may perform light client-side validation for usability, but the Rust core remains authoritative.

## Core Crate Design

Each calculation method should expose:

- A typed input struct.
- A typed output struct.
- A validation function.
- A calculation function.
- Unit tests.
- Reference examples.
- Method documentation.

Example shape:

```rust
pub struct TwoSampleTTestInput {
    pub alpha: f64,
    pub power: Option<f64>,
    pub sample_size_per_group: Option<u32>,
    pub mean_difference: f64,
    pub standard_deviation: f64,
    pub allocation_ratio: f64,
    pub alternative: Alternative,
}

pub struct TwoSampleTTestResult {
    pub n_treatment: u32,
    pub n_control: u32,
    pub total_n: u32,
    pub achieved_power: f64,
    pub effect_size: f64,
    pub warnings: Vec<CalculationWarning>,
}
```

## Method Registry

Use a registry so the UI can list available methods without hard-coding every method in multiple places.

The registry should provide:

- Method identifier.
- Display name.
- Endpoint category.
- Supported solving modes.
- Input schema metadata.
- Documentation link.

Do not let the registry become the calculation engine. It should describe methods and route requests only.

## Error Handling

Use structured errors in Rust and map them to UI-friendly messages at the boundary.

Error categories:

- Invalid input.
- Unsupported method.
- Numerical convergence failure.
- Internal calculation error.
- File or export error.

Avoid returning raw strings from the core engine except at outermost application boundaries.

## Extensibility Rule

Adding a new statistical method should require changes in predictable places:

1. Method documentation.
2. Input and result types.
3. Calculation implementation.
4. Unit tests.
5. Validation cases.
6. Registry entry.
7. Svelte form and result renderer.

If adding a method requires broad unrelated changes, the architecture is too coupled.

