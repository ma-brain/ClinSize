# Mann-Whitney U reference output

Independent reference: **Noether, G. E. (1987), Sample Size Formula for the
Mann-Whitney (Wilcoxon rank-sum) Test**, *Journal of the American Statistical
Association*, 82(398): 645–647.

ClinSize uses the Noether asymptotic normal approximation, which maps the
location shift to a probability of superiority `p = P(Treatment > Control)`
under equal-variance normality, `p = Φ(δ / (σ√2))`, and then derives the
per-group sample size as:

```
n_per_arm = (z_{1−α/2} + z_{1−β})² / [6 (p − 0.5)²]      (two-sided)
n_per_arm = (z_{1−α}   + z_{1−β})² / [6 (p − 0.5)²]      (one-sided)
```

This is the standard Noether (1987) formula. Achieved power at the rounded
integer `n` is recomputed by inverting the same expression and has a tolerance
of ±0.01 to reflect the asymptotic (large-sample) approximation.

## Case: two_sided_d0358

Effect size `d = 0.3583`, SD 1.0, two-sided α 0.05, power 0.8, equal allocation.

- `p = Φ(0.3583 / √2) = 0.6000`
- `n = (1.959964 + 0.8416212)² / [6 × 0.1²] = 130.67 → 131` per arm
- Total `N = 262`; achieved power at `n = 131` ≈ 0.8006

## Case: one_sided_d0358

Same effect, one-sided α 0.05 (`z_{0.95} = 1.6448536`).

- `n = (1.6448536 + 0.8416212)² / [6 × 0.1²] = 102.98 → 103`
- Rounded per-arm split under 1:1 allocation: 103 control / 104 treatment
- Achieved power ≈ 0.8016

## Case: power_round_trip

Feed `controlN = 131` back in two-sided sample-size mode inputs; verifies the
achieved power round-trips to ≈ 0.80.

## Tolerance

`probabilitySuperiority` is pinned to ±1e-3; `achievedPower` to ±0.01
(Noether's asymptotic formula is not exact at small `n`). Per-arm sample sizes
are exact integers.
