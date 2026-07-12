# Validation report: Two-sample t-test

- Engine version: 0.1.0
- Method: `continuous.two_sample_ttest`
- Cases: 4 (4 passed, 0 failed)

## Results

| Case | Field | Expected | Actual | Status |
| --- | --- | --- | --- | --- |
| equal_two_sided_d1 | achievedPower | 0.80704 ± 1e-4 | 0.8070367151474108 | pass |
| equal_two_sided_d1 | nControl | 17 | 17 | pass |
| equal_two_sided_d1 | nTreatment | 17 | 17 | pass |
| equal_two_sided_d05 | achievedPower | 0.80146 ± 1e-4 | 0.8014595579222842 | pass |
| equal_two_sided_d05 | nControl | 64 | 64 | pass |
| equal_two_sided_d05 | nTreatment | 64 | 64 | pass |
| one_sided_d1 | achievedPower | 0.82409 ± 1e-4 | 0.8240858545704081 | pass |
| one_sided_d1 | nControl | 14 | 14 | pass |
| one_sided_d1 | nTreatment | 14 | 14 | pass |
| dropout_20pct | achievedPower | 0.80704 ± 1e-4 | 0.8070367151474108 | pass |
| dropout_20pct | nControl | 17 | 17 | pass |
| dropout_20pct | nControlAdjusted | 22 | 22 | pass |
| dropout_20pct | nTreatment | 17 | 17 | pass |
| dropout_20pct | nTreatmentAdjusted | 22 | 22 | pass |

## Reference sources

- `equal_two_sided_d1` — R power.t.test(delta = 1, sd = 1, sig.level = 0.05, power = 0.8)
- `equal_two_sided_d05` — R power.t.test(delta = 0.5, sd = 1, sig.level = 0.05, power = 0.8)
- `one_sided_d1` — R power.t.test(delta = 1, sd = 1, sig.level = 0.05, power = 0.8, alternative = "one.sided")
- `dropout_20pct` — ClinSize rounding policy: enrolled n = ceil(randomized n / (1 - dropout)) = ceil(17 / 0.8) = 22

## Reviewer notes

Formal independent review is pending. This report documents automated reference
tests only and does not constitute regulatory validation.
