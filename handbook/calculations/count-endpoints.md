# Count Endpoints

## Methods

**Implemented** in `clinsize-core`:

- `count.negative_binomial` — two-sample negative binomial for recurrent event counts.

## Negative Binomial (Recurrent Events)

Uses Zhu & Lakkis (2014) Wald test for the log rate ratio (gsDesignNB Method 3 default).

**Effect:** `θ = log(λ₂/λ₁)` where λᵢ are event rates per unit exposure.

**Expected counts:** `μᵢ = λᵢ × exposure_time`

**Variance factor:** `Ṽ = (1/μ₁ + k) + (1/r)(1/μ₂ + k)` where `r` is the treatment-to-control allocation ratio and `k` is the common NB2 dispersion parameter.

**Sample size:**

`n_control = ⌈(z_{α/s} + z_β)² × Ṽ / θ²⌉`

`n_treatment = ⌈n_control × r⌉`

Power is verified by inverting the sizing formula after integer rounding.

## Validation Reference Cases

| λ₁ | λ₂ | k | exposure | ratio | α | power | n_control | total |
|----|----|---|----------|-------|---|-------|-----------|-------|
| 2  | 1  | 1 | 1        | 1     | 0.05 | 0.8 | 58 | 116 |
| 5  | 10 | 0.5 | 1      | 1     | 0.05 | 0.8 | 22 | 44  |

## Inputs

- Control and treatment event rates.
- Dispersion k (NB2 variance `Var(Y) = μ + kμ²`).
- Exposure time (default 1).
- Alpha, target power or control N.
- Allocation ratio.
- Alternative hypothesis.
- Optional dropout rate.

## Outputs

- Per-group and total sample sizes (with dropout-adjusted sizes).
- Achieved power.
- Rate ratio λ₂/λ₁.
- Warnings and assumptions.

## Assumptions

- NB2 negative binomial counts with common dispersion.
- Fixed exposure per subject.
- Wald test on log rate ratio; normal approximation for power.

## Limitations

- Does not model time-varying exposure or frailty beyond the stated dispersion.
- Detectable-effect solve mode is not implemented.
