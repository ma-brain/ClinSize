# Poisson reference output

Independent reference: **Signorini, M. C. (1991), On the Analysis of Poisson
Count Data in Longitudinal Studies**, PhD thesis, Harvard School of Public
Health — the Wald-test sample-size formula for comparing two Poisson rates via
the log rate ratio. Equivalently, Zhu & Lakkis (2014) with dispersion k = 0.

ClinSize applies the Wald test on `log(λ₂/λ₁)` with Poisson variance
`Var(Y) = μ` (no overdispersion):

```
ṽ = 1/μ₁ + (1/r)(1/μ₂)        where μᵢ = λᵢ·t, r = n_treatment/n_control
n_control = (z_{1−α/2} + z_{1−β})² · ṽ / (log(λ₂/λ₁))²
```

## Case: halving_rate

`controlRate = 2`, `treatmentRate = 1` (rate ratio 0.5), exposure 1, two-sided
α 0.05, power 0.8.

- `ṽ = 1/2 + 1/1 = 1.5`, `θ = |ln(0.5)| = 0.6931`
- `(z_{0.975} + z_{0.8})² = (1.9600 + 0.8416)² = 7.849`
- `n = 7.849 × 1.5 / 0.4805 = 24.49 → 25` per arm
- Total `N = 50`; achieved power ≈ 0.807

## Case: doubling_rate

`controlRate = 5`, `treatmentRate = 10` (rate ratio 2.0), exposure 1, two-sided
α 0.05, power 0.8.

- `ṽ = 1/5 + 1/10 = 0.3`
- `n = 7.849 × 0.3 / 0.4805 = 4.90 → 5` per arm
- Total `N = 10`; achieved power ≈ 0.82

## Case: power_round_trip

Feed `controlN = 25` back in with the halving-rate inputs; achieved power
round-trips to ≈ 0.81.

## Tolerance

`rateRatio` is closed-form exact (±1e-12). `achievedPower` uses ±0.02 — the
Wald formula is a large-sample normal approximation, and rounding the
continuous `n` up to an integer moves achieved power slightly off the target.
Per-arm sample sizes are exact integers.

## Relationship to negative binomial

This is the limiting case of the NB2 Wald test (Zhu & Lakkis 2014) as the
dispersion parameter `k → 0`. Setting `k = 0` in the NB variance factor
`(1/μ₁ + k) + (1/r)(1/μ₂ + k)` removes the `kμ²` overdispersion terms, leaving
the Poisson variance `Var(Y) = μ` and the formula above.
