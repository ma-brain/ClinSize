# MMRM (Longitudinal Repeated Measures)

## Scope

**Implemented** as `continuous.mmrm` for parallel two-group designs with repeated continuous measures analyzed via mixed model for repeated measures (MMRM) at the final post-baseline visit.

## Formula Or Algorithm

Reference: Lu K, Skellam JS (1988) — analytical GLS variance efficiency under MMRM.

Within-subject correlation uses a simplified single-ρ parameterization:

| Structure | ρ_final |
|-----------|---------|
| AR(1), Toeplitz | ρ^k |
| Unstructured, compound symmetry, CSH | ρ |

where k = number of post-baseline visits.

**GLS variance efficiency factor:** `1 + (k − 1) × ρ_final`

**GLS factor:** `1 / (1 + (k − 1) × ρ_final)`

**Effective variance:** `V_eff = 2 × σ² × (1 − ρ_final) × GLS_factor`

**Evaluable per arm (normal approximation):**

`n_arm = (z_{α/s} + z_β)² × V_eff / δ²`

ClinSize finds the smallest integer control-group evaluable size such that the rounded treatment-group size achieves the target power, then recomputes achieved power at that allocation.

**Visit dropout:**

`d_cum = 1 − (1 − d_visit)^k`

`N_enr per arm = ⌈n_arm / (1 − d_cum)⌉`

Reference case: δ=2, σ=2, unstructured, ρ=0.5, k=3, α=0.05 two-sided, power=0.8, per-visit dropout 5%, allocation 1:1 → GLS efficiency **2.0**, cumulative dropout **≈14%**, **4 evaluable per arm**, **5 enrollable per arm (10 total)**.

## Inputs

- Treatment effect δ at the final post-baseline visit.
- Residual standard deviation σ.
- Correlation structure and ρ.
- Number of post-baseline visits k (integer ≥ 1).
- Alpha, target power or control N.
- Allocation ratio.
- Optional per-visit dropout rate.

## Outputs

- Evaluable and enrollable per-group and total sample sizes.
- Achieved power.
- ρ_final, GLS factor, GLS variance efficiency factor, V_eff.
- Cumulative dropout.
- Warnings and assumptions.

## Assumptions

- MMRM with visit as a categorical factor.
- Simplified single-ρ within-subject correlation (not a full correlation matrix).
- Equal residual variance across arms; effect at the final post-baseline visit.
- Visit-level dropout independent with constant per-visit rate.

## Limitations

- Normal approximation for sample size (not exact t or Kenward–Roger).
- Does not model baseline as a covariate or unstructured correlation matrices with multiple distinct parameters.
- Detectable-effect solve mode is not implemented.
- Not validated against external reference software beyond the Lu–Skellam reference case in unit tests.
