# Independent Validation Evidence Design

## Goal

Remove stale validation-report work from the TODO and add independently
reproducible evidence for the seven methods that currently lack it.

## Scope

The validation corpus will gain one `cases.json` file and one
`reference-output.md` file for each of these registered methods:

- `continuous.mann_whitney`
- `continuous.wilcoxon_signed_rank`
- `continuous.paired_ttest`
- `continuous.change_from_baseline`
- `count.negative_binomial`
- `ordinal.proportional_odds`
- `design.multiplicity`

Each JSON case remains executable by the existing generic validation-report
runner. Its `source` field records the authoritative R package/function (and
the exact function call) that produced its expected values. The companion
Markdown file captures the R commands, package versions, output, and any
rounding rule used to translate the external result to ClinSize's integer
allocation.

## Evidence Strategy

Prefer an executable, maintained R implementation for every case:

- `power.t.test(type = "paired")` for paired t tests.
- R implementations matching the rank-test and proportional-odds models where
  their assumptions align with ClinSize.
- `mvtnorm` and base R multiplicity procedures for Dunnett and stepwise
  multiplicity calculations.

Where no maintained R implementation accepts the exact ClinSize model,
document a published formula in `reference-output.md`, show the independent
calculation, and identify the assumption match explicitly. This exception is
expected for the selected negative-binomial and multiplicity cases.

Every method will receive at least a sample-size and an achieved-power case
where the endpoint supports both. Cases will include alternate-sided or
allocation coverage when the external reference exposes that dimension.

## Acceptance Criteria

- The generic embedded validation suite includes all seven new files and every
  expectation passes.
- Each new source is independently reproducible from the checked-in R command
  or documented formula; no expected number is derived from the Rust result.
- The stale validation-report TODO item is removed.
- The missing-evidence TODO item is removed only when all seven method files
  are present and the full workspace tests pass.

## Out of Scope

- Reworking numerical methods or changing calculation behavior to force a
  match.
- Running R dynamically in CI; the checked-in external outputs avoid making
  the Rust test suite depend on local R package installation.
