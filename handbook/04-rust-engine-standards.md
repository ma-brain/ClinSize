# Rust Engine Standards

## Scope

The Rust engine is responsible for statistical correctness, numerical algorithms, validation, and report data. It should be deterministic, testable, and independent of the UI.

## Design Rules

- Put all statistical methods in `clinsize-core`.
- Do not use Tauri types in `clinsize-core`.
- Use typed structs for all public inputs and outputs.
- Use enums for finite option sets such as alternative hypothesis or allocation mode.
- Return `Result<T, Error>` for operations that can fail.
- Keep formulas and numerical algorithms close to the relevant method module.
- Use `serde` for boundary DTOs only when needed.

## Numeric Rules

- Use `f64` for statistical calculations unless there is a documented reason not to.
- Validate all inputs before calculation.
- Document tolerances for every test that compares floating point values.
- Avoid silent rounding except for final sample size decisions.
- Always report achieved power when sample size is rounded upward.
- Document whether formulas use normal, t, chi-square, F, exact, or asymptotic approximations.

## Input Validation

Every method should validate:

- Probability parameters are in valid ranges.
- Alpha is greater than 0 and less than 1.
- Power is greater than 0 and less than 1 when supplied.
- Standard deviations and variances are positive.
- Allocation ratios are positive.
- Dropout rates are at least 0 and less than 1.
- Mutually exclusive solve modes are not both supplied.

Validation errors should identify the field and the rule.

## Rounding Policy

Sample size outputs should round up to the next integer. If group allocation is unequal, round group sizes conservatively and report total N and achieved power.

Do not hide rounding decisions. The result should expose:

- Raw continuous estimate, if useful.
- Rounded group sizes.
- Total sample size.
- Achieved power after rounding.

## Testing Rules

Each method requires:

- Unit tests for validation errors.
- Unit tests for normal input cases.
- Boundary tests for common edge values.
- Reference tests against published or independently generated values.
- Regression tests for every corrected statistical or numerical defect.

## Dependencies

Acceptable core dependencies may include:

- `serde` for serialization.
- `thiserror` for structured errors.
- `statrs`, `libm`, or equivalent numerical libraries after review.
- `approx` for floating point test assertions.

Avoid adding large dependencies to the core crate unless they directly support statistical correctness or maintainability.

## Module Pattern

Use a consistent module shape:

```text
src/methods/continuous/two_sample_ttest.rs
src/methods/continuous/mod.rs
src/methods/mod.rs
```

Each method module should contain:

- Input type.
- Result type.
- Validation logic.
- Calculation logic.
- Method-specific tests.

