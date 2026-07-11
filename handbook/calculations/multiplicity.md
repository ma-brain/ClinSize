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
- `adjustmentMethod`: `bonferroni` or `sidak`.

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

## Assumptions

- Bonferroni: valid under arbitrary dependence; conservative with positive correlation.
- Šidák: assumes independent comparisons.
- Does not model correlation structure between endpoints.

## Validation Rules

- `familyWiseAlpha` must be in (0, 1).
- `numberOfComparisons` must be at least 1.
- Adjusted alpha must remain positive.

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

## Known Limitations

- Dunnett adjustment for multiple arms vs control is not implemented.
- Holm, Hochberg, and gatekeeping procedures are not implemented.
- FDR methods are not implemented.
- Does not automatically propagate adjusted alpha into endpoint methods; users
  apply the result manually.

## UI Requirements

- Inputs: family-wise alpha, number of comparisons, adjustment method.
- Results: adjusted alpha, reduction factor, warnings.
- Export via standard Markdown/HTML/Word workflow.
- Guidance to use adjusted alpha in endpoint calculations.
