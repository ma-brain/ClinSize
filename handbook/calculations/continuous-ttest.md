# Continuous Endpoints: t-tests

## Methods

Initial methods:

- One-sample t-test.
- Paired t-test.
- Two-sample t-test with equal variance assumption.

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

## Known Limitations

Early versions may use normal approximations for planning. If exact t-based power is implemented, document the degrees of freedom and numerical integration or distribution functions used.

