# Nonparametric Endpoints

## Methods

**Implemented** in `clinsize-core`:

- `continuous.mann_whitney` — Mann-Whitney U (Wilcoxon rank-sum) two-group comparison.
- `continuous.wilcoxon_signed_rank` — Wilcoxon signed-rank paired comparison.

Both methods use the Noether (1987) normal approximation for sample size and power.

## Mann-Whitney U

Nonparametric superiority design for two independent groups on a continuous endpoint.

**Planning parameter:** probability of superiority `P(Y_treatment > Y_control)`, derived from the mean difference and common standard deviation under equal-variance normality:

`P_superiority = Φ(d / √2)`

where `d` is Cohen's d (mean difference divided by standard deviation) and `Φ` is the standard normal CDF.

**Sample size (Noether 1987):** total sample size follows

`N = (z_{α/2} + z_β)² / G`

with `G = 12 × t × (1 − t) × (P_superiority − 0.5)²`, where `t` is the treatment fraction from the allocation ratio. One-sided alternatives use `z_α` instead of `z_{α/2}`.

Group sizes are split from total N using the allocation ratio; achieved power is recalculated after rounding.

## Wilcoxon Signed-Rank

Nonparametric superiority design for paired continuous differences.

**Planning parameter:** `P(difference > 0)`, derived from the expected mean and SD of within-subject differences under normality:

`P_positive = Φ(μ_diff / σ_diff)`

**Sample size (Noether 1987):** number of pairs

`N = (z_{α/2} + z_β)² / (12 × (P_positive − 0.5)²)`

One-sided alternatives use `z_α`. Achieved power is recalculated after rounding up to integer pairs.

## Inputs

**Mann-Whitney:**

- Mean difference (treatment minus control).
- Common standard deviation.
- Allocation ratio.
- Alpha, target power or control group N.
- Alternative (two-sided, greater, less).
- Dropout rate (optional).

**Wilcoxon signed-rank:**

- Expected mean paired difference.
- Standard deviation of differences.
- Alpha, target power or number of pairs.
- Alternative.
- Dropout rate (optional).

## Outputs

**Mann-Whitney:** control N, treatment N, total N, dropout-adjusted sizes, achieved power, P(treatment > control), Cohen's d, warnings.

**Wilcoxon signed-rank:** number of pairs, dropout-adjusted pairs, achieved power, P(difference > 0), Cohen's d, warnings.

## Assumptions

- Continuous endpoints without ties.
- Location shift mapped to a probability parameter (superiority or positive difference) under normality for planning.
- Mann-Whitney assumes equal within-group variance when translating mean difference to P(superiority).
- Noether (1987) normal approximation; exact rank-based power and tie corrections are not implemented.

## Limitations

- The normal mapping from mean/SD to probability parameters is an approximation; skewed or heavy-tailed data may deviate.
- Ties, clustering, and stratification are not modeled.
- Detectable-effect solve mode is not implemented for either method.

## Validation

Compare against:

- Noether (1987) published sample size tables and formulas.
- Reference shifts yielding P(superiority) = 0.6 or P(positive) = 0.6 at 80% power (unit tests in `mann_whitney.rs` and `wilcoxon_signed_rank.rs`).

Tolerance: achieved power within ±0.02 after integer rounding.

## Reference

Noether GE. Sample size determination for some common nonparametric tests. *Journal of the American Statistical Association*. 1987;82(398):645–647.
