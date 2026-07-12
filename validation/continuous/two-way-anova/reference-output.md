# Two-way ANOVA reference output

Independent reference: **Cohen, J. (1988), Statistical Power Analysis for the
Behavioral Sciences** (2nd ed.), Chapter 8 — noncentral-F power for fixed-effect
two-way ANOVA. Cross-checkable against G*Power ("ANOVA: Fixed effects, two-way")
and R `pwr.f2.test` via the f² → λ conversion.

ClinSize computes exact power for the user-selected primary effect (main A, main
B, or interaction AB) via the noncentral F distribution:

```
df1   = numerator df for the primary effect
        Main A: a − 1 | Main B: b − 1 | Interaction: (a−1)(b−1)
df2   = ab(n − 1)                         (common denominator df)
λ     = df1 × multiplier × n × (σ²_effect / σ²_error)
        where multiplier = b for Main A, a for Main B, 1 for Interaction
power = P(F'(df1, df2, λ) > F_crit(df1, df2, α))
```

The sample size is the smallest integer `n_per_cell` achieving target power.

## Case: main_a_2x3

`nLevelsA = 2`, `nLevelsB = 3`, primary = main_a,
`varianceA = 0.5`, `withinVariance = 1.0`, α = 0.05, power = 0.8.

- `df1 = a − 1 = 1`, multiplier = b = 3
- Cohen's f = √(0.5/1.0) = 0.7071
- `λ = 1 × 3 × n × 0.5`
- At n = 6: `df2 = 6 × 5 = 30`, `λ = 9.0`, power ≈ 0.827

## Case: main_b_2x3

Same design, primary = main_b.

- `df1 = b − 1 = 2`, multiplier = a = 2
- At n = 6: `λ = 2 × 2 × 6 × 0.5 = 12.0`, power ≈ 0.847 (higher df1 → more power
  at the same n and effect size)

## Case: interaction_2x3

Primary = interaction.

- `df1 = (a−1)(b−1) = 2`, multiplier = 1 (no cross-factor scaling)
- `λ = 2 × 1 × n × 0.5`
- At n = 11: `df2 = 6 × 10 = 60`, `λ = 11.0`, power ≈ 0.833
- The interaction needs nearly twice the replicates of the main effects because
  its noncentrality grows linearly in n (not b·n or a·n).

## Case: power_round_trip

Feed `nPerCell = 6` back with the main_a inputs; achieved power round-trips to
≈ 0.827.

## Tolerance

`effectSize` is closed-form (±1e-6). `achievedPower` is pinned to ±1e-3 — the
noncentral-F CDF is computed via the same `r_mathlib` routines R uses
(`non_central_f_cdf`), so agreement is near-exact. Per-cell sample sizes are
exact integers.
