# Proportional odds reference output

Independent references:

- **Whitehead, J. (1993), Sample size calculations for ordered categorical
  data**, *Statistics in Medicine*, 12(24): 2257–2271 — the underlying formula.
- R package **Hmisc** function `posamsize`, which implements Whitehead (1993)
  with the efficiency factor `ps = 1 − Σpᵢ³`.

ClinSize applies the Whitehead / `posamsize` formula:

```
N_total = (z_{1−α/2} + z_{1−β})² / [ (log OR)² × (1 − Σ pᵢ³) × θ × (1 − θ) ]
```

where `θ` is the treatment fraction. The per-arm split rounds to satisfy the
allocation fraction, so `nControl` and `nTreatment` may differ by one.

## Case: four_cat_or2_power8

`categoryProbabilities = [0.2, 0.5, 0.2, 0.1]`, `oddsRatio = 2`,
`treatmentFraction = 0.5`, α 0.05, power 0.8.

- Efficiency `1 − (0.2³ + 0.5³ + 0.2³ + 0.1³) = 1 − 0.142 = 0.858`
- Continuous `N` ≈ 228.5 → rounded split **115 / 116**, total **231**
- Achieved power at `N = 231` ≈ 0.801

## Case: four_cat_or2_power9

Same categories, oddsRatio 2, α 0.025, power 0.9.

- Continuous `N` ≈ 363.3 → rounded **182 / 182**, total **364**
- Achieved power ≈ 0.901

## Case: power_round_trip

Feed `controlN = 115` back in; achieved power round-trips to ≈ 0.80.

## Tolerance

`efficiency` is closed-form (±1e-3). `achievedPower` uses ±0.02 — the
continuous `N` is rounded up to an integer, moving achieved power slightly off
the target. Sample sizes are exact integers.
