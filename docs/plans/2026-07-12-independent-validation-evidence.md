# Independent Validation Evidence Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Replace stale validation TODO items with independently reproducible external evidence for every currently unsupported method.

**Architecture:** Keep the existing generic `cases.json` runner and compile-time embedding unchanged. Each added method directory supplies machine-checked external expectations in `cases.json` and a human-auditable R transcript or formula derivation in `reference-output.md`; expected values are never copied from Rust calculations.

**Tech Stack:** Rust test runner, JSON evidence, R reference packages, maintained statistical package documentation.

---

### Task 1: Remove the stale validation-report TODO

**Files:**
- Modify: `TODO.md`

1. Delete the item claiming the validation-report generator only supports two methods.
2. Renumber the remaining TODO entries.
3. Confirm `TODO.md` retains the missing-evidence item until all seven evidence files are added.

### Task 2: Capture external reference outputs

**Files:**
- Create: `validation/continuous/mann-whitney/reference-output.md`
- Create: `validation/continuous/wilcoxon-signed-rank/reference-output.md`
- Create: `validation/continuous/paired-ttest/reference-output.md`
- Create: `validation/continuous/change-from-baseline/reference-output.md`
- Create: `validation/count/negative-binomial/reference-output.md`
- Create: `validation/ordinal/proportional-odds/reference-output.md`
- Create: `validation/design/multiplicity/reference-output.md`

1. Check the installed R package versions and select an external reference whose assumptions match each ClinSize method.
2. Run every reference command outside the Rust engine and record its command, package/version, raw output, interpretation, and integer-rounding rule in the matching Markdown file.
3. Where no matching maintained package exists, document the published formula, independent calculation, and why it matches the ClinSize model.
4. Do not create a `cases.json` value until it has a corresponding independent reference output.

### Task 3: Add paired and rank-test evidence

**Files:**
- Create: `validation/continuous/paired-ttest/cases.json`
- Create: `validation/continuous/mann-whitney/cases.json`
- Create: `validation/continuous/wilcoxon-signed-rank/cases.json`
- Test: `crates/clinsize-core/src/validation_report/mod.rs`

1. Add a temporary failing assertion that the embedded evidence count is at least 15.
2. Run `cargo test -p clinsize-core validation_report::tests::every_embedded_case_passes -- --nocapture` and confirm it fails before the new files exist.
3. Add independently sourced sample-size and achieved-power cases for paired t test, Mann-Whitney, and Wilcoxon signed-rank. Match each method's JSON input shape and pin integer allocations exactly; use explicit tolerances for floating outputs.
4. Re-run the focused validation-report test and confirm every new case passes.

### Task 4: Add continuous, count, and ordinal evidence

**Files:**
- Create: `validation/continuous/change-from-baseline/cases.json`
- Create: `validation/count/negative-binomial/cases.json`
- Create: `validation/ordinal/proportional-odds/cases.json`
- Test: `crates/clinsize-core/src/validation_report/mod.rs`

1. Raise the failing embedded-evidence-count assertion from 15 to 18.
2. Add externally derived cases for change-from-baseline, negative binomial, and proportional odds, including the reference model's covariance/dispersion/category assumptions.
3. Run the focused validation-report test and inspect generated reports for a pass row per case.

### Task 5: Add multiplicity evidence and close the TODO

**Files:**
- Create: `validation/design/multiplicity/cases.json`
- Modify: `TODO.md`
- Test: `crates/clinsize-core/src/validation_report/mod.rs`

1. Raise the failing embedded-evidence-count assertion from 18 to 19.
2. Add independent cases for Bonferroni, Sidak, Dunnett, Holm, Hochberg, and graphical gatekeeping. Record the external base-R or `mvtnorm` procedure and the fixed-order/weight assumptions.
3. Remove the missing-validation-evidence TODO item only after all seven directories are present.
4. Run `cargo test -p clinsize-core validation_report -- --nocapture` and `cargo test --workspace`.

### Task 6: Final audit

**Files:**
- Modify: `validation/README.md`

1. Update the evidence inventory if it names a supported-method count.
2. Run `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, and `git diff --check`.
3. Verify each new `cases.json` source is represented verbatim in its corresponding `reference-output.md`.
