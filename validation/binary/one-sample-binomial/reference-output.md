# One-sample binomial reference output

Independent reference values generated with R `EnvStats` (Millard,
*EnvStats: An R Package for Environmental Statistics*), which implements the
classical normal-approximation power for the one-sample proportion z-test:
the critical region uses the null standard error `√(p₀(1−p₀)/n)` and power is
evaluated under the alternative standard error `√(p₁(1−p₁)/n)`, summing both
rejection tails for two-sided tests (Fleiss, Levin & Paik 2003, §2.5).

```r
library(EnvStats)

run <- function(p0, p1, alpha, power, alt) {
  n <- propTestN(p.or.p1 = p1, p0.or.p2 = p0, alpha = alpha, power = power,
                 sample.type = "one.sample", alternative = alt,
                 approx = TRUE, correct = FALSE)
  achieved <- propTestPower(n.or.n1 = n, p.or.p1 = p1, p0.or.p2 = p0,
                            alpha = alpha, sample.type = "one.sample",
                            alternative = alt, approx = TRUE, correct = FALSE)
  c(n = n, achieved = achieved)
}

run(0.2, 0.4, 0.05, 0.8, "two.sided")   # n = 36,  achieved = 0.8021367
run(0.3, 0.5, 0.05, 0.8, "two.sided")   # n = 44,  achieved = 0.8042717
run(0.2, 0.4, 0.05, 0.9, "two.sided")   # n = 50,  achieved = 0.9008601
run(0.5, 0.65, 0.025, 0.9, "greater")   # n = 113, achieved = 0.9012011
run(0.4, 0.25, 0.05, 0.8, "less")       # n = 61,  achieved = 0.8008358
```

## History

Prior to the 2026-07 fix, the two-sided implementation computed power as
`2Φ(z) − 1` (an interval probability, not a rejection probability), which
oversized two-sided designs by roughly 40% (e.g. n = 50 instead of 36 for
p₀ = 0.2, p₁ = 0.4, α = 0.05, power 0.8) and understated achieved power at a
given n. One-sided calculations were unaffected. Previously exported two-sided
results were conservative (larger than necessary), never underpowered.
