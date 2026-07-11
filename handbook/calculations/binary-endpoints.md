# Binary Endpoints

## Methods

**Implemented** in `clinsize-core`:

- `binary.two_proportion_difference` — difference in proportions (superiority and basic non-inferiority).
- `binary.odds_ratio` — log odds-ratio superiority.
- `binary.risk_ratio` — log risk-ratio superiority.

## Difference In Proportions

Uses a normal approximation to the binomial distribution.

**Superiority:** matches R `stats::power.prop.test` for equal allocation; pooled normal approximation for unequal allocation.

**Non-inferiority (higher-is-better):** Chow et al. (2003) formula validated against TrialSize `TwoSampleProportion.NIS`. The margin is the maximum acceptable deficit (control minus treatment).

## Odds Ratio

Log odds-ratio normal approximation (Chow et al. 2003). Validated against TrialSize `RelativeRisk.Equality` (which parameterizes odds ratio despite the function name).

`OR = p_t(1 − p_c) / (p_c(1 − p_t))`

## Risk Ratio

Log risk-ratio normal approximation (Chow et al. 2003).

`RR = p_t / p_c`

## Inputs

- Control event rate.
- Treatment event rate.
- Alpha.
- Target power or sample size.
- Allocation ratio.
- Study objective (superiority or non-inferiority for proportion difference).
- Non-inferiority margin when applicable.
- Dropout rate (optional).

## Outputs

- Group sample sizes.
- Total sample size.
- Achieved power.
- Effect measure (rate difference, odds ratio, or risk ratio).
- Assumptions.
- Warnings.

## Assumptions

- Independent Bernoulli outcomes.
- Fixed allocation ratio.
- Higher event rate is favorable.
- Approximation method is documented in warnings and exports.

## Method Choices

Binary endpoint sample size has several valid approaches. ClinSize documents which approximation is used per method. Continuity correction, score, and exact methods are not yet implemented.

## Validation

Compare against:

- R `power.prop.test` (proportion difference superiority).
- TrialSize `TwoSampleProportion.NIS` (proportion difference non-inferiority).
- TrialSize `RelativeRisk.Equality` (odds ratio).
- Chow et al. 2003 published formulas (risk ratio).

Tolerance may vary by method because different tools use different approximations.
