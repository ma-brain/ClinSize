# Group Sequential Designs

## Method Identifier

`design.group_sequential`

## Purpose

Plan interim efficacy boundaries and sample size inflation for a group
sequential trial with equally spaced information fractions. Use the inflation
factor to scale a fixed-design sample size to the maximum sample size required
under the sequential plan.

## Endpoint Type

Design utility (not tied to a specific endpoint distribution).

## Hypotheses

This method does not test a clinical endpoint directly. It supplies interim
efficacy boundaries and a sample size inflation factor for downstream endpoint
planning.

## Inputs

- `alpha`: Two-sided family-wise Type I error rate.
- `targetPower`: Target power for the group sequential design.
- `numberOfLooks`: Total number of looks, including the final analysis (2–10).
- `spendingFunction`: `obrien_fleming` or `pocock`.

## Outputs

- `looks`: Per-look information fraction, incremental and cumulative alpha
  spent, and upper Z boundary.
- `sampleSizeInflationFactor`: Multiplier on fixed-design sample size.
- `requiredDrift` and `fixedDesignDrift`: Standardized effect sizes used in
  boundary solving.
- `achievedPower`: Power under the fixed-design drift at the inflated sample
  size.
- `warnings`: Assumption notes and edge-case guidance.

## Formula Or Algorithm

**Alpha spending (Lan-DeMets)**

O'Brien-Fleming cumulative spend at information fraction `t`:

```text
alpha(t) = 2 * (1 - Phi(z_{alpha/2} / sqrt(t)))
```

Pocock cumulative spend:

```text
alpha(t) = alpha * ln(1 + (e - 1) * t)
```

Incremental spend at look `k` with equally spaced timing `t_k = k/K` is
`alpha(t_k) - alpha(t_{k-1})`.

**Boundary solving**

For each look, find the upper Z boundary such that the probability of first
crossing at or before that look under the null equals the incremental alpha
spent. Uses recursive integration with 32-point Gauss-Legendre quadrature over
the multivariate normal correlation structure of standardized interim
statistics.

**Sample size inflation**

Matches gsDesign `n.I[k]` for one-sided upper-bound efficacy (`test.type = 1`).
find multiplier `w` such that equally spaced information `w * t_k` achieves
target power at the fixed-design drift

```text
theta_fixed = z_{1-alpha} + z_{power}
```

## Assumptions

- Equally spaced information fractions (`t_k = k/K`).
- Symmetric two-sided efficacy boundaries (upper boundary only).
- No futility stopping or non-binding futility bounds.
- Normal standardized test statistic with known variance (planning model).

## Validation Rules

- `alpha` must be in (0, 1).
- `targetPower` must be in (0, 1).
- `numberOfLooks` must be between 2 and 10.

## Rounding Policy

No rounding; boundaries and inflation factor are reported to six decimal
places in exports.

## Reference Sources

- Lan, K. K. G. & DeMets, D. L. (1983). Discrete sequential boundaries.
- Jennison, C. & Turnbull, B. W. *Group Sequential Methods with Applications
  to Clinical Trials*.

Validated against R `gsDesign::gsDesign` with `test.type = 1` (one-sided
upper efficacy). `test.type = 4` asymmetric beta-spending inflation is not
yet implemented.

## Test Cases

| α | Power | K | Spending | Expected inflation |
| --- | --- | --- | --- | --- |
| 0.05 | 0.8 | 3 | O'Brien-Fleming | 1.020305 |
| 0.05 | 0.8 | 5 | Pocock | 1.221578 |
| 0.05 | 0.8 | 3 | Pocock | 1.176743 |

| α | K | Spending | Look 1 Z |
| --- | --- | --- | --- |
| 0.05 | 3 | O'Brien-Fleming | 3.200102 |
| 0.05 | 3 | Pocock | 2.002014 |

## Known Limitations

- Custom information timing is not supported.
- Futility bounds and asymmetric boundaries are not implemented.
- Inflation matches gsDesign `test.type = 1`; `test.type = 4` lower
  beta-spending adjustment is not yet included.
- Spending families beyond O'Brien-Fleming and Pocock are not implemented.
- Does not automatically propagate inflation into endpoint methods; users
  apply the factor manually.

## UI Requirements

- Inputs: alpha, target power, number of looks, spending function.
- Results: per-look boundary table, inflation factor, achieved power, warnings.
- Export via standard Markdown/HTML/Word workflow.
- Guidance to multiply fixed-design sample size by the inflation factor.
