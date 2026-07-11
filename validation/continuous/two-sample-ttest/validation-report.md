# Two-sample t-test validation report

## Method version

- ClinSize engine: 0.1.0
- Method identifier: `continuous.two_sample_ttest`

## Reference source

- R `stats::power.t.test` (equal allocation, two-sample)
- Handbook: `handbook/calculations/continuous-ttest.md`

## Test cases

See `cases.csv` and `reference-output.md`.

## Results

| Case | Expected control N | Actual control N | Expected power | Actual power | Status |
|------|-------------------|------------------|----------------|--------------|--------|
| equal_two_sided_d1 | 17 | 17 | 0.80704 | 0.80704 | pass |
| equal_two_sided_d05 | 64 | 64 | 0.80146 | 0.80146 | pass |
| one_sided_d1 | 14 | 14 | 0.82409 | 0.82409 | pass |
| dropout_20pct adjusted N | 22 | 22 | — | — | pass |

Automated coverage: `cargo test -p clinsize-core two_sample_ttest`

## Tolerances

- Achieved power: absolute `1e-4` against R at rounded integer sample sizes
- Sample size integers: exact match required

## Reviewer notes

Formal independent review is pending. This report documents automated reference
tests only and does not constitute regulatory validation.
