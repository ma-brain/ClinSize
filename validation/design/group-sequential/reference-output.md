# Group sequential design reference output

Reference values generated with R `gsDesign` (Keaven Anderson). ClinSize
implements a **one-sided efficacy-only** design (`test.type = 1`): the full
`alpha` is spent on the upper boundary via Lan-DeMets spending, and the
inflation factor is `max(n.I)` with `n.fix = 1`.

```r
library(gsDesign)

show <- function(k, alpha, sf) {
  x <- gsDesign(k = k, test.type = 1, alpha = alpha, beta = 0.2,
                sfu = sf, n.fix = 1)
  list(bounds = x$upper$bound,
       incremental_spend = x$upper$spend,
       inflation = max(x$n.I))
}

show(3, 0.025, sfLDOF)      # bounds 3.710303 2.511427 1.993048, inflation 1.012795
show(3, 0.025, sfLDPocock)  # bounds 2.279428 2.294910 2.295939, inflation 1.170419
show(5, 0.025, sfLDPocock)  # bounds 2.437977 2.426814 2.410194 2.396649 2.386000, inflation 1.212613
show(4, 0.025, sfLDOF)      # bounds 4.332634 2.963131 2.359044 2.014090, inflation 1.019637
show(3, 0.05,  sfLDOF)      # bounds 3.200102 2.140815 1.694812, inflation 1.020305
show(5, 0.05,  sfLDPocock)  # bounds 2.176211 ... 2.070998, inflation 1.221484
```

Boundary tolerance is 0.02 (absolute Z): ClinSize solves boundaries with a
deterministic 12,000-point Halton quasi-Monte-Carlo integration of the
multivariate normal, which is accurate to roughly two decimal places on
boundary values. gsDesign's recursive numerical integration is more precise;
treat gsDesign as authoritative where they differ.

## History

Prior to the 2026-07 fix, the `alpha` input was documented and labeled in the
UI as a "two-sided family-wise Type I error rate" while the engine computed a
one-sided design spending the full alpha on the upper boundary. A user
entering 0.05 as "two-sided alpha" therefore obtained boundaries appropriate
for one-sided 0.05 (equivalently two-sided 0.10) — actual two-sided Type I
error was double the label. The computation itself was always a correct
one-sided gsDesign `test.type = 1` design; the fix relabels alpha as
one-sided everywhere (engine docs, UI, reports, protocol text) and changes
the UI default to 0.025. Previously exported results computed at
"two-sided alpha = 0.05" should be re-read as one-sided alpha = 0.05 designs.
