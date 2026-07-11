# Binary Endpoints

## Methods

Initial methods:

- Difference in proportions.
- Risk ratio.
- Odds ratio.

Priority should be given to difference in proportions because it is common and easier to explain.

## Inputs

- Control event rate.
- Treatment event rate or effect measure.
- Alpha.
- Target power or sample size.
- Allocation ratio.
- Superiority, non-inferiority, or equivalence objective.
- Continuity correction option, if supported.

## Outputs

- Group sample sizes.
- Total sample size.
- Achieved power.
- Assumptions.
- Warnings.

## Assumptions

- Independent Bernoulli outcomes.
- Fixed allocation ratio.
- Approximation method is documented.

## Method Choices

Binary endpoint sample size has several valid approaches. The app must state which method is used:

- Normal approximation.
- Arcsine approximation.
- Score method.
- Exact method.

Do not present a binary calculation without naming the approximation.

## Validation

Compare against:

- R packages such as `pwr`, `TrialSize`, or other reviewed packages.
- Published examples.
- SAS or commercial software outputs when available.

Tolerance may vary by method because different tools use different approximations.

