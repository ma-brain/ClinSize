# Statistical Methodology

## Method Documentation Rule

Every statistical method must have a dedicated method note before implementation is considered complete.

The method note must define:

- Purpose.
- Clinical use cases.
- Endpoint type.
- Hypotheses.
- Inputs.
- Outputs.
- Formula or algorithm.
- Assumptions.
- Parameter validation.
- Rounding rules.
- References.
- Validation examples.
- Known limitations.

## Source Of Truth

The Rust implementation and method documentation must agree. If they diverge, the method is not ready for release.

## Solve Modes

Common solve modes:

- Solve for sample size given target power.
- Solve for power given sample size.
- Solve for detectable effect given sample size and power.

Do not implement all solve modes for every method by default. Add modes when there is a clear use case and validation evidence.

## Statistical Assumptions

Each method should explicitly state assumptions such as:

- Endpoint distribution.
- Equal or unequal variances.
- Equal or unequal allocation.
- One-sided or two-sided alpha.
- Approximation type.
- Independence assumptions.
- Censoring assumptions for time-to-event methods.

## Numerical Methods

Some calculations are closed form; others require root finding. For numerical methods:

- Define convergence tolerance.
- Define maximum iterations.
- Return a convergence error if the solver fails.
- Test monotonicity where applicable.
- Avoid unbounded searches without safeguards.

## Independent Validation

Reference values should come from at least one of:

- Published textbook examples.
- Peer-reviewed articles.
- Established R packages.
- SAS procedures.
- PASS, nQuery, East, or other validated software when available.
- Independently reviewed spreadsheet calculations.

Document the source, version, parameters, and expected result.

## Reporting Requirements

Every result should be reportable as text:

- Method name.
- Endpoint type.
- Hypothesis.
- Formula or approximation family.
- Input parameters.
- Result.
- Achieved power after rounding.
- Warnings and limitations.
- Date/time and software version.

Do not claim regulatory validation unless a formal validation process has been completed.

