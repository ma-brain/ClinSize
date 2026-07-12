# Wilcoxon signed-rank reference output

Independent reference: **Noether, G. E. (1987), Sample Size Formula for the
Mann-Whitney (Wilcoxon rank-sum) Test**, *Journal of the American Statistical
Association*, 82(398): 645–647.

ClinSize applies the Noether (1987) asymptotic normal approximation to the
one-sample/paired Wilcoxon signed-rank test. The mean shift is mapped to a
probability of a positive difference `p = P(difference > 0)` under normality,
`p = Φ(δ / σ)`, and the required number of pairs is:

```
n_pairs = (z_{1−α/2} + z_{1−β})² / [3 (p − 0.5)²]      (two-sided)
n_pairs = (z_{1−α}   + z_{1−β})² / [3 (p − 0.5)²]      (one-sided)
```

The factor 3 (rather than 6) reflects the one-sample variance of the signed
rank sign under the null. Achieved power is recomputed at the rounded integer
`n` with a tolerance of ±0.01 to reflect the asymptotic approximation.

## Case: two_sided_d0253

Effect `d = 0.2533`, SD 1.0, two-sided α 0.05, power 0.8.

- `p = Φ(0.2533 / 1.0) = 0.6000`
- `n = (1.959964 + 0.8416212)² / [3 × 0.1²] = 131.34 → 131` pairs
- Achieved power at `n = 131` ≈ 0.8004

## Case: one_sided_d0253

Same effect, one-sided α 0.05 (`z_{0.95} = 1.6448536`).

- `n = (1.6448536 + 0.8416212)² / [3 × 0.1²] = 103.58 → 104` pairs
- Achieved power ≈ 0.8031

## Case: power_round_trip

Feed `nPairs = 131` back in; verifies achieved power round-trips to ≈ 0.80.

## Tolerance

`probabilityPositiveDifference` is pinned to ±1e-3; `achievedPower` to ±0.01
(asymptotic formula, not exact at small `n`). Pair counts are exact integers.
