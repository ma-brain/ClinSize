# Non-inferiority and Equivalence

## Scope

Non-inferiority and equivalence methods should be added only after the corresponding superiority methods are stable and validated.

## Concepts

Non-inferiority tests whether an investigational treatment is not worse than control by more than a specified margin.

Equivalence tests whether the treatment effect lies within a pre-specified acceptable interval.

## Inputs

- Endpoint type.
- Effect measure.
- Non-inferiority or equivalence margin.
- Alpha.
- Power.
- Allocation ratio.
- Expected control response.
- Expected treatment response.

## Outputs

- Group sample sizes.
- Total sample size.
- Achieved power.
- Margin interpretation.
- Warnings and assumptions.

## Margin Handling

The UI and report must make the margin direction explicit. Ambiguous margin direction is a serious statistical risk.

For example:

- Higher values are better.
- Lower values are better.
- Difference is treatment minus control.
- Difference is control minus treatment.

## Validation

Validate carefully against multiple sources because different packages define signs and margins differently.

Required tests:

- Higher-is-better endpoint.
- Lower-is-better endpoint.
- One-sided non-inferiority test.
- Two one-sided equivalence tests.
- Invalid margin direction.

