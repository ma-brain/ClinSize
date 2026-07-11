# ANOVA and ANCOVA

## ANOVA Scope

Initial ANOVA support should focus on one-way fixed-effect ANOVA for comparing means across multiple groups. **Implemented** as `continuous.one_way_anova` (balanced groups).

## Formula Or Algorithm

Uses R `stats::power.anova.test` parameterization with between-group variance `σ²_b` and within-group variance `σ²_w`:

`λ = (k − 1) × n × (σ²_b / σ²_w)`

`df₁ = k − 1`, `df₂ = k × (n − 1)`

Power = `P(F > F_{α, df₁, df₂} | λ)` via noncentral F (`r_mathlib` / R `pnf`).

Cohen's `f = √(σ²_b / σ²_w)`. Per-group sample size is the smallest integer `n ≥ 2` achieving the target power after rounding.

Validation reference: R `power.anova.test`.

## ANOVA Inputs

- Number of groups.
- Group allocation pattern.
- Group means or effect size.
- Common standard deviation.
- Alpha.
- Target power or sample size.

## ANOVA Outputs

- Sample size per group.
- Total sample size.
- Achieved power.
- Effect size.
- Warnings and assumptions.

## ANOVA Assumptions

- Independent observations.
- Normally distributed residuals or adequate approximation.
- Common variance across groups.
- Fixed group design.

## ANCOVA Scope

**Implemented** as `continuous.ancova_two_sample` for parallel two-group designs with one baseline covariate.

The first version uses an approximate variance reduction formula rather than a full model-based ANCOVA calculation.

## ANCOVA Formula Or Algorithm

Adjusted residual standard deviation:

`σ_adj = σ_y × √(1 − ρ²)`

where `σ_y` is the unadjusted outcome standard deviation and `ρ` is the baseline-outcome correlation.

Sample size and power then follow the equal-variance two-sample t-test using `σ_adj`. Cohen's d in outputs uses the unadjusted `σ_y`.

Validation reference: R `power.t.test` with `sd = σ_y × √(1 − ρ²)`.

## ANCOVA Inputs

- Unadjusted endpoint standard deviation.
- Baseline-outcome correlation or variance reduction factor.
- Mean difference.
- Alpha.
- Target power.
- Allocation ratio.

## ANCOVA Validation

Compare against:

- Published formulas.
- R or SAS examples.
- Independently reviewed calculations.

## Implementation Caution

ANCOVA assumptions are easy to oversimplify. Clearly state whether the method uses an approximate variance reduction formula or a full model-based calculation.

