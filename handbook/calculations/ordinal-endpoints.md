# Ordinal Endpoints

## Methods

**Implemented** in `clinsize-core`:

- `ordinal.proportional_odds` — two-group ordinal comparison under proportional odds (Whitehead 1993 / Hmisc `posamsize`).

## Proportional Odds

**Efficiency factor:** `ps = 1 − Σpᵢ³` where `pᵢ` are control-group category probabilities ordered best → worst.

**Allocation:** `A = (1 − f)/f` where `f` is the treatment fraction.

**Sample size (continuous total N):**

`n_total = 3 × (A+1)² × (z_{α/2} + z_β)² / A / (log OR)² / ps`

Total N is rounded up; treatment and control sizes follow the treatment fraction.

**Power (Hmisc `popower` approximation):**

`V = n₁ × n₂ × n / (3(n+1)²) × ps`

`power = Φ(|log OR| × √V − z_{α/2})`

## Validation Reference Case

Control probabilities `[0.2, 0.5, 0.2, 0.1]`, OR=2, treatment fraction=0.5, α=0.05, power=0.8:

- Continuous total N ≈ **228.5**; smallest integer total achieving 80% power is **231** (after allocation rounding)
- Efficiency ps ≈ **0.858**

## Inputs

- Category probabilities for the control group (best to worst).
- Target odds ratio (> 1).
- Treatment fraction (0, 1).
- Alpha, target power or control N.
- Optional dropout rate.

## Outputs

- Per-group and total sample sizes (with dropout-adjusted sizes).
- Achieved power.
- Efficiency factor ps.
- Warnings and assumptions.

## Assumptions

- Proportional odds holds across cumulative logits.
- Two-group comparison with specified allocation.
- Category probabilities describe the control arm; treatment shifts odds uniformly.

## Limitations

- Two-sided test only in the current UI (formula uses z_{α/2}).
- Detectable-effect solve mode is not implemented.
- Does not accept subject-level ordinal data — planning uses assumed control category proportions.
