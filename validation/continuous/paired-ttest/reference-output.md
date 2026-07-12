# Paired t-test reference output

Reference source: R 4.x `stats::power.t.test`, `type = "paired"`. The paired
test analyzes within-subject differences with the exact noncentral-t power
distribution; `df = n_pairs − 1`, noncentrality parameter
`δ = mean_difference / (sd / √n_pairs)`.

```r
# two_sided_d1
power.t.test(delta = 1, sd = 1, sig.level = 0.05, power = 0.8,
             type = "paired", alternative = "two.sided")
# n per pair = 9.94 -> rounded to 10; achieved power at n = 10 is 0.8074

# one_sided_d1
power.t.test(delta = 1, sd = 1, sig.level = 0.05, power = 0.8,
             type = "paired", alternative = "one.sided")
# n = 7.7 -> rounded to 8; achieved power at n = 8 is 0.8150

# power_round_trip
power.t.test(n = 10, delta = 1, sd = 1, sig.level = 0.05,
             type = "paired", alternative = "two.sided")
# power = 0.8074
```

## Tolerance

ClinSize uses the exact noncentral-t distribution (same as R), so achieved-power
comparisons use absolute tolerance `1e-4`. Pair counts are exact integers.
