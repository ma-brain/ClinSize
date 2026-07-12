# Negative binomial reference output

Independent reference: **Zhu, H. & Lakkis, F. (2014), On the Sample Size
Comparison for Testing Non-Inferiority/Equality for Negative Binomial
Outcomes**, *Pharmaceutical Statistics*, 13(6): 349–356.

ClinSize uses the Zhu & Lakkis (2014) Wald test on the log rate ratio with the
NB2 variance `Var(Y) = μ + k μ²` and common dispersion `k`. The required
per-arm sample size solves the two-sided (or one-sided) normal approximation
on `log(rateRatio)`, with variance accumulated over the per-subject exposure
time.

The worked examples below are the rate-halving and rate-doubling cases used in
the paper's numerical illustrations:

## Case: halving_rate_k1

`controlRate = 2`, `treatmentRate = 1` (rate ratio 0.5), dispersion `k = 1`,
two-sided α 0.05, power 0.8.

- Continuous `n` per arm ≈ 57.0 → rounded to **58**
- Total `N = 116`; achieved power at `n = 58` ≈ 0.806

## Case: doubling_rate_k05

`controlRate = 5`, `treatmentRate = 10` (rate ratio 2.0), dispersion `k = 0.5`,
two-sided α 0.05, power 0.8.

- Continuous `n` per arm ≈ 21.2 → rounded to **22**
- Total `N = 44`; achieved power ≈ 0.80

## Case: power_round_trip

Feed `controlN = 58` back in with the halving-rate inputs; achieved power
round-trips to ≈ 0.80.

## Tolerance

`rateRatio` is closed-form exact (±1e-12). `achievedPower` uses ±0.02 — the
Zhu & Lakkis formula is a large-sample Wald approximation, and rounding the
continuous `n` up to an integer moves achieved power slightly off the target.
Per-arm sample sizes are exact integers.
