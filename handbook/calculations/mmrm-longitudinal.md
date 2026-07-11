# MMRM (Longitudinal Repeated Measures)

## Scope

**Implemented** as `continuous.mmrm` for parallel two-group designs with
repeated continuous measures analyzed via mixed model for repeated measures
(MMRM), testing the treatment difference at the final post-baseline visit.

## Formula Or Algorithm

Reference: Lu, K., Luo, X., & Chen, P.-Y. (2008). Sample size estimation for
repeated measures analysis in randomized clinical trials with missing data.
*The International Journal of Biostatistics*, 4(1). Implemented in R as
`longpower::power.mmrm`.

With `K` post-baseline visits, within-subject correlation matrix `R` built
from a single ρ, and monotone retention `r_j` (probability of still being
observed at visit `j`):

| Structure | R entries |
|-----------|-----------|
| Compound symmetry | `R[i,j] = ρ` for `i ≠ j` |
| AR(1) | `R[i,j] = ρ^|i−j|` |

**Information accumulation** (`r_{K+1} = 0`):

`I = Σ_{j=1..K} (r_j − r_{j+1}) × pad(inv(R[1..j, 1..j]))`

**Variance factor:** `φ = inv(I)[K, K]` — equals 1 with complete data, so
MMRM never claims to beat the final-visit two-sample test; dropout only
increases φ.

**Contrast variance:** `Var(δ̂) = σ² × φ × (1/n_control + 1/n_treatment)`
where `n` are randomized counts and σ is the final-visit SD.

**Sample size (normal approximation):** smallest integer control-group size
such that the rounded treatment-group size (allocation ratio λ) achieves the
target power; achieved power is recomputed at the integer allocation and, for
two-sided tests, sums both rejection tails.

**Visit dropout:** per-visit rate `d` maps to geometric retention
`r_j = (1 − d)^j`. Dropout is modeled *inside* the MMRM variance: subjects
who discontinue contribute their observed visits, so reported sample sizes
are randomized counts and no separate enrollment inflation is applied.

Reference case: δ=2, σ=2, compound symmetry ρ=0.5, k=3, α=0.05 two-sided,
power=0.8, 5%/visit dropout, 1:1 → φ = **1.13306**, **18 randomized per arm
(36 total)**, achieved power 0.80466 (longpower: n/arm = 17.79).

## Inputs

- Treatment effect δ at the final post-baseline visit.
- Standard deviation σ at the final visit.
- Correlation structure (compound symmetry or AR(1)) and ρ.
- Number of post-baseline visits k (integer 1–20).
- Alpha, target power or control N.
- Allocation ratio.
- Optional per-visit dropout rate.

## Outputs

- Randomized per-group and total sample sizes.
- Achieved power.
- Variance factor φ, final-visit retention, cumulative dropout.
- Warnings and assumptions.

## Assumptions

- MMRM with visit as a categorical factor; effect tested at the final visit.
- Single-ρ compound symmetry or AR(1) within-subject correlation.
- Equal variance, correlation, and retention across arms.
- Monotone missingness with constant per-visit dropout rate.

## Limitations

- Normal approximation for sample size (not exact t or Kenward–Roger).
- Unstructured, Toeplitz, and CSH correlation matrices are rejected: they
  need more parameters than the single ρ collected by the UI.
- Does not model baseline as a covariate.
- Detectable-effect solve mode is not implemented.

## Validation

Cross-checked against R `longpower::power.mmrm` (sample size, power, and
variance factor to ≤1e-5); see `validation/continuous/mmrm/`.

## History

Before 2026-07 this method used a fabricated formula attributed to a
nonexistent "Lu–Skellam (1988)" reference, with
`V_eff = 2σ²(1−ρ)/(1+(k−1)ρ)`. That formula understated the required sample
size by roughly a factor of (1+(k−1)ρ)/(1−ρ) — about 4× at ρ=0.5, k=3 (it
claimed 4 evaluable subjects per arm where the final-visit t-test alone needs
17). Any sample size exported by the old MMRM method should be recomputed.
