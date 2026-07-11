# Multiplicity Adjustments

## Method Identifier

`design.multiplicity`

## Purpose

Convert a family-wise Type I error rate into a per-comparison alpha for sample
size planning when a trial tests multiple hypotheses in a single family.

## Endpoint Type

Design utility (not tied to a specific endpoint distribution).

## Hypotheses

This method does not test a clinical endpoint directly. It supplies an adjusted
per-comparison alpha to use in downstream endpoint calculations.

## Inputs

- `familyWiseAlpha`: Family-wise Type I error rate to control across all comparisons.
- `numberOfComparisons`: Number of comparisons in the family (integer ≥ 1).
- `adjustmentMethod`: `bonferroni`, `sidak`, `dunnett`, or `holm`.
- `gatePosition`: Position in the gatekeeping sequence (1 = first), required for Holm.

## Outputs

- `adjustedAlpha`: Per-comparison alpha to use in endpoint sample size formulas.
- `alphaReductionFactor`: Ratio of adjusted alpha to the family-wise alpha.
- `warnings`: Assumption notes and edge-case guidance.

## Formula Or Algorithm

**Bonferroni**

```text
alpha_adj = alpha_family / m
```

Controls the family-wise error rate (FWER) at `alpha_family` under any
dependence structure (Boole's inequality). Conservative when comparisons are
positively correlated.

**Šidák (Sidak)**

```text
alpha_adj = 1 - (1 - alpha_family)^(1/m)
```

Controls FWER exactly at `alpha_family` when the `m` comparisons are
independent. Slightly less conservative than Bonferroni under independence.

**Dunnett (many arms vs control)**

For `m` active treatment arms each compared with a common control using equal
per-group sample sizes, solve the two-sided Dunnett critical value `c` from

```text
P(|Z_i| <= c for all i = 1..m) = 1 - alpha_family
```

where `(Z_1, ..., Z_m)` is multivariate standard normal with pairwise
correlation 0.5 between treatment-vs-control contrasts. The equivalent
per-comparison alpha is

```text
alpha_adj = 2 * Phi(-c)
```

Computed via the equicorrelated-normal integral representation and validated
against R `mvtnorm::pmvnorm`.

**Holm gatekeeping (fixed order)**

For hypothesis `k` in a pre-specified family of `m` ordered comparisons, the
local alpha used when gates `1..k-1` have passed is

```text
alpha_adj = alpha_family / (m - k + 1)
```

Gate `k = 1` matches Bonferroni (`alpha_family / m`). Gate `k = m` uses the
full family-wise alpha. Holm step-down controls FWER under arbitrary dependence.

## Assumptions

- Bonferroni: valid under arbitrary dependence; conservative with positive correlation.
- Šidák: assumes independent comparisons.
- Dunnett: equal per-group sample sizes; one control; two-sided contrasts.
- Holm: fixed pre-specified testing order; planning assumes prior gates pass.

## Validation Rules

- `familyWiseAlpha` must be in (0, 1).
- `numberOfComparisons` must be at least 1.
- `gatePosition` must be between 1 and `numberOfComparisons` when using Holm.

## Rounding Policy

No rounding; adjusted alpha is reported to six decimal places in exports.

## Reference Sources

- Julious, S. A. *Sample Sizes for Clinical Trials* (2010), multiplicity discussion.
- Chow, S.-C. et al. *Sample Size Calculations in Clinical Research*.

Validated against closed-form arithmetic reference values.

## Test Cases

| Family-wise α | m | Method | Expected α_adj |
| --- | --- | --- | --- |
| 0.05 | 2 | Bonferroni | 0.025 |
| 0.05 | 5 | Bonferroni | 0.01 |
| 0.05 | 2 | Šidák | 0.025321 |
| 0.05 | 5 | Šidák | 0.010206 |
| 0.05 | 2 | Dunnett | 0.026958 |
| 0.05 | 3 | Dunnett | 0.018824 |
| 0.05 | 5 | Dunnett | 0.012023 |
| 0.05 | 5 | Holm (gate 1) | 0.01 |
| 0.05 | 5 | Holm (gate 3) | 0.016667 |
| 0.05 | 5 | Holm (gate 5) | 0.05 |

## Known Limitations

- Hochberg step-up and graphical gatekeeping are not implemented.
- FDR methods are not implemented.
- Does not automatically propagate adjusted alpha into endpoint methods; users
  apply the result manually.

## UI Requirements

- Inputs: family-wise alpha, number of comparisons, adjustment method.
- Results: adjusted alpha, reduction factor, warnings.
- Export via standard Markdown/HTML/Word workflow.
- Guidance to use adjusted alpha in endpoint calculations.
