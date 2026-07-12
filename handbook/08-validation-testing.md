# Validation and Testing

## Validation Philosophy

ClinSize should be developed with a validation mindset even if early versions are not formally validated for regulated use. Statistical correctness is a product feature, not an implementation detail.

## Test Layers

Use several test layers:

- Rust unit tests for formulas and validation.
- Rust integration tests for method workflows.
- Reference case tests for known values.
- TypeScript tests for UI helpers and DTO handling.
- Component tests for calculation forms.
- End-to-end tests for key desktop workflows when practical.

## Acceptance Criteria For A Method

A calculation method is acceptable when:

- Inputs and outputs are typed.
- Validation rejects invalid parameters.
- Formula or numerical algorithm is documented.
- Unit tests cover normal and invalid cases.
- Reference tests match independent values within documented tolerances.
- UI displays inputs, outputs, assumptions, and warnings.
- Exported report includes enough detail to reproduce the calculation.

## Floating Point Tolerances

Use explicit tolerances. Example:

```rust
assert_relative_eq!(result.achieved_power, 0.9001, epsilon = 1e-4);
```

Choose tolerances based on method and reference precision. Do not hide large deviations by using broad tolerances.

## Red-Green Regression Rule

For every fixed defect:

1. Add a test that fails with the old behavior.
2. Implement the fix.
3. Confirm the test passes.
4. Keep the test permanently.

## Validation Evidence

For each method, store validation evidence under:

```text
validation/
  continuous/
    two-sample-ttest/
      cases.json
      reference-output.md
      validation-report.md
```

`cases.json` (format documented in `validation/README.md`) is machine-checked:
every case runs through the engine's JSON dispatch in CI, and the same files
are embedded into the binaries at compile time so the app's Validation page
and `clinsize validation-report` work outside the repository checkout.

Each validation report should include:

- Method version.
- Reference source.
- Test cases.
- Expected values.
- Actual values.
- Tolerances.
- Pass/fail status.
- Reviewer notes.

## CI Requirements

CI should run:

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
pnpm check
pnpm test
```

Release CI should also build platform installers. See
`handbook/extended-platforms.md` and `.github/workflows/release.yml`.

