# Continuous Endpoints: t-tests

## Methods

Initial methods:

- One-sample t-test. **Implemented** (`continuous.one_sample_ttest`).
- Paired t-test. **Implemented** (`continuous.paired_ttest`).
- Two-sample t-test with equal variance assumption. **Implemented** (`continuous.two_sample_ttest`).

Later methods:

- Welch two-sample t-test.
- Non-parametric approximations.

## Two-Sample t-test Scope

The first implementation should support superiority testing for a continuous endpoint with:

- Treatment and control groups.
- Mean difference.
- Common standard deviation.
- Allocation ratio.
- One-sided or two-sided alpha.
- Target power.

## Inputs

- `alpha`: Type I error rate.
- `power`: Target power when solving for sample size.
- `sample_size`: Group sizes when solving for power.
- `mean_difference`: Expected treatment effect.
- `standard_deviation`: Common standard deviation.
- `allocation_ratio`: Treatment-to-control allocation.
- `alternative`: Two-sided, greater, or less.
- `dropout_rate`: Optional inflation factor.

## Outputs

- Control group N.
- Treatment group N.
- Total N.
- Dropout-adjusted N, if applicable.
- Achieved power.
- Standardized effect size.
- Warnings.

## Assumptions

- Independent observations.
- Continuous endpoint.
- Approximately normal endpoint distribution or adequate large-sample behavior.
- Common standard deviation for equal-variance method.
- Fixed allocation ratio.

## Validation

Compare against:

- R `power.t.test`.
- Published textbook examples.
- Independently reviewed spreadsheet calculations.

Tests should cover equal allocation, unequal allocation, one-sided alpha, two-sided alpha, and dropout inflation.

## Formula Or Algorithm

Equal-variance two-sample t-test power uses the noncentral t distribution with
degrees of freedom `ν = n_control + n_treatment − 2` and noncentrality parameter:

`δ = (μ_treatment − μ_control) / (σ × √(1/n_treatment + 1/n_control))`

Two-sided power at significance `α`:

`P(T > t_{α/2, ν} | δ) + P(T < −t_{α/2, ν} | δ)`

where `T` follows a noncentral t distribution with `ν` df and noncentrality `δ`.
One-sided greater/less alternatives use `t_{α, ν}` with the corresponding single
tail.

Sample size is the smallest integer control-group size such that the rounded
allocation (`n_treatment = ⌈n_control × allocation_ratio⌉`) achieves at least the
target power. Achieved power is recalculated after rounding. Dropout inflation
multiplies rounded group sizes by `1/(1 − dropout_rate)` and rounds up.

Implementation uses R's `pnt` algorithm via the `r_mathlib` crate. Validation
reference: R `power.t.test` (stats package).

## Known Limitations

- Equal variance only; Welch's test is not implemented yet.
- Detectable-effect solve mode is not implemented.
- Unequal allocation uses integer-rounded treatment sizes; validation against R
  `power.t.test` covers equal allocation only (base R does not accept a ratio
  argument).
- Dropout inflation applies a simple uniform inflation factor; it does not model
  differential dropout between arms.

