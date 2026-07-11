# ANOVA and ANCOVA

## ANOVA Scope

Initial ANOVA support should focus on one-way fixed-effect ANOVA for comparing means across multiple groups.

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

ANCOVA should not be rushed. The first ANCOVA version may support a simple variance reduction adjustment based on correlation between baseline and outcome.

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

