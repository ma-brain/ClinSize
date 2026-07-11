# Change From Baseline

## Scope

**Implemented** as `continuous.change_from_baseline` for parallel two-group designs comparing mean change-from-baseline (CFB) scores.

## Formula Or Algorithm

Under equal baseline and follow-up standard deviation σ and baseline-outcome correlation ρ:

`σ_cfb = σ × √(2(1 − ρ))`

Sample size and power delegate to the equal-variance two-sample t-test using `σ_cfb` as the common within-group standard deviation. Cohen's d in outputs uses the unadjusted outcome SD.

Validation reference: R `power.t.test(delta=Δ, sd=σ×√(2×(1−ρ)), sig.level=α, power=1−β, type="two.sample")`.

Reference case: Δ=3, σ=10, ρ=0.5, α=0.05, power=0.8 → **176 per arm**.

## Inputs

- Mean CFB difference (treatment minus control).
- Common outcome standard deviation.
- Baseline-outcome correlation.
- Alpha, target power or control N.
- Allocation ratio.
- Optional dropout rate.

## Outputs

- Per-group and total sample sizes (with dropout-adjusted sizes).
- Achieved power.
- Cohen's d (unadjusted SD).
- Change-score standard deviation σ_cfb.
- Warnings and assumptions.

## Assumptions

- Parallel groups; independent subjects.
- Normality of CFB scores (or adequate approximation).
- Common σ at baseline and follow-up.
- Equal baseline-outcome correlation across arms.

## Limitations

- Does not model baseline imbalance or ANCOVA adjustment (see `continuous.ancova_two_sample`).
- Detectable-effect solve mode is not implemented.
