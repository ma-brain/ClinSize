# Binary Endpoints

## Methods

**Implemented** in `clinsize-core`:

- `binary.two_proportion_difference` — difference in proportions (superiority and basic non-inferiority).
- `binary.odds_ratio` — log odds-ratio superiority.
- `binary.risk_ratio` — log risk-ratio superiority.
- `binary.one_sample_binomial` — single-arm response rate versus a reference proportion.

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

## One-Sample Binomial

Compares a single-arm response rate to a reference (null) proportion using a normal approximation to the binomial distribution.

**Formula:** For sample size, finds the smallest integer `N` such that achieved power meets the target. Power uses separate variance estimates at the reference and hypothesized rates:

`z = (|p_alt − p_ref| × √N − z_{α/2} × √(p_ref(1 − p_ref))) / √(p_alt(1 − p_alt))`

One-sided alternatives use `z_α` and the corresponding tail probability.

**Inputs:** reference rate, hypothesized response rate, alpha, target power or sample size, alternative, optional dropout rate.

**Outputs:** sample size N, dropout-adjusted N, achieved power, rate difference, warnings.

**Assumptions:**

- Independent Bernoulli outcomes on one arm.
- Normal approximation; continuity correction and exact binomial power are not implemented.
- Rates must differ and be consistent with the chosen alternative.

**Limitations:** Small sample sizes or rates near 0 or 1 may make the normal approximation unreliable. Detectable-effect solve mode is not implemented.

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
