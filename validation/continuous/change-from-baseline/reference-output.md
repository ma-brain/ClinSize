# Change from baseline reference output

Independent reference: **Frison, L. & Pocock, S. J. (1992), Repeated measures
in clinical trials: analysis using mean summary statistics and its
implications for design**, *Statistics in Medicine*, 11(13): 1685–1704 — for
the change-score standard deviation `σ_cfb = σ √(2(1 − ρ))` — combined with
R `stats::power.t.test` (`type = "two.sample"`) on the change-score SD using
exact noncentral-t power.

ClinSize derives the within-subject change-score SD, then applies the standard
two-sample t-test sample-size formula with that SD.

## Case: two_sided_rho05

`meanDifference = 3`, `standardDeviation = 10`, `baselineOutcomeCorrelation = 0.5`,
two-sided α 0.05, power 0.8.

- `σ_cfb = 10 × √(2 × 0.5) = 10.0`
- Effect size on the change scale `= 3 / 10 = 0.3`

```r
power.t.test(delta = 3, sd = 10, sig.level = 0.05, power = 0.8,
             type = "two.sample", alternative = "two.sided")
# n per arm = 175.39 -> rounded to 176; achieved power at n = 176 is 0.8014
```

## Case: one_sided_rho05

Same inputs, one-sided α 0.05:

```r
power.t.test(delta = 3, sd = 10, sig.level = 0.05, power = 0.8,
             type = "two.sample", alternative = "one.sided")
# n per arm = 138.5 -> rounded to 139; achieved power ≈ 0.8023
```

## Case: power_round_trip

Feed `controlN = 176` back in; achieved power round-trips to ≈ 0.8014.

## Tolerance

`changeScoreStandardDeviation` pinned to ±1e-5, `effectSize` to ±1e-12 (both
closed-form), achieved power to ±1e-4 (exact noncentral-t, matches R).
Per-arm sample sizes are exact integers.
