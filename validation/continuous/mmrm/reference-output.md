# MMRM reference output

Independent reference values generated with R `longpower::power.mmrm`
(version 1.0.27), which implements Lu, Luo & Chen (2008), *Sample size
estimation for repeated measures analysis in randomized clinical trials with
missing data*, The International Journal of Biostatistics 4(1).

ClinSize builds the correlation matrix from a single ρ (compound symmetry or
AR(1)) and geometric retention `(1 − d)^j` from the per-visit dropout rate,
then applies the same information accumulation as `power.mmrm`.

```r
library(longpower)

cs  <- function(k, rho) { m <- matrix(rho, k, k); diag(m) <- 1; m }
ar1 <- function(k, rho) outer(1:k, 1:k, function(i, j) rho^abs(i - j))
ret <- function(k, d) (1 - d)^(1:k)

# cs_k3_dropout5: n/arm = 17.786522 (ClinSize rounds to 18)
power.mmrm(Ra = cs(3, 0.5), ra = ret(3, 0.05), sigmaa = 2, lambda = 1,
           delta = 2, sig.level = 0.05, power = 0.8, alternative = "two.sided")
power.mmrm(N = 36, Ra = cs(3, 0.5), ra = ret(3, 0.05), sigmaa = 2, lambda = 1,
           delta = 2, sig.level = 0.05, alternative = "two.sided")   # power = 0.8046597

# cs_k3_complete: n/arm = 15.697759 -> 16; power at N = 32 is 0.8074296
power.mmrm(Ra = cs(3, 0.5), ra = rep(1, 3), sigmaa = 2, lambda = 1,
           delta = 2, sig.level = 0.05, power = 0.8, alternative = "two.sided")

# ar1_k4_dropout10: n/arm = 121.528892 -> 122; power at N = 244 is 0.9010971
power.mmrm(Ra = ar1(4, 0.6), ra = ret(4, 0.1), sigmaa = 3, lambda = 1,
           delta = 1.5, sig.level = 0.025, power = 0.9, alternative = "one.sided")

# cs_k5_lambda2: n1 = 263.762110, n2 = 131.881055; power at N = 396 is 0.9002562
power.mmrm(Ra = cs(5, 0.3), ra = ret(5, 0.08), sigmaa = 1.2, lambda = 2,
           delta = 0.5, sig.level = 0.05, power = 0.9, alternative = "two.sided")

# ar1_k4_power_at_74: power at N = 74 is 0.4319623
power.mmrm(N = 74, Ra = ar1(4, 0.6), ra = ret(4, 0.1), sigmaa = 3, lambda = 1,
           delta = 1.5, sig.level = 0.025, alternative = "one.sided")
```

The variance factor φ column is the `[K, K]` element of the inverse
information matrix (`solve(Ia)[K, K]` in the `power.mmrm` source), computed
with the same correlation and retention inputs.

Two-sided achieved powers differ from longpower by the opposite rejection
tail (< 1e-6 at these operating points); tolerances in `cases.csv` absorb it.

## History

Before the 2026-07 fix, this method used a fabricated formula attributed to a
nonexistent "Lu–Skellam (1988)" reference:
`V_eff = 2σ²(1−ρ_final)/(1+(k−1)ρ_final)`. With complete data an MMRM
final-visit contrast cannot beat the final-visit two-sample test (φ = 1), but
the old formula claimed a 4× variance reduction at ρ=0.5, k=3, producing 4
subjects per arm where the t-test alone requires 17. Results exported before
the fix understate the required sample size by roughly
`(1+(k−1)ρ)/(1−ρ)` and must be recomputed. The old outputs
(`glsVarianceEfficiencyFactor`, `vEff`, `rhoFinal`, enrollable-N inflation)
were removed together with the formula.
