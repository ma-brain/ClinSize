# Multiplicity reference output

Each case compares ClinSize's adjusted per-comparison alpha against the
closed-form value from the adjustment method's original publication. All
inputs use `familyWiseAlpha = 0.05`.

## Bonferroni (1936)

```text
alpha_adjusted = alpha / m
```

- `bonferroni_m2`: `0.05 / 2 = 0.025`, reduction factor `0.5`
- `bonferroni_m5`: `0.05 / 5 = 0.01`

Reference: Bonferroni, C. E. (1936), *Teoria statistica delle classi e calcolo
delle probabilità*.

## Šidák (1967)

```text
alpha_adjusted = 1 - (1 - alpha)^(1/m)
```

- `sidak_m2`: `1 - 0.95^0.5 = 0.025320566`
- `sidak_m5`: `1 - 0.95^0.2 = 0.010206212`

Reference: Šidák, Z. (1967), Rectangular confidence regions for the means of
multivariate normal distributions, *JASA* 62(318).

## Dunnett (1955)

Equicorrelated (`rho = 0.5`) many-to-one normal contrast. The adjusted alpha
is the two-sided tail probability at the Dunnett quantile, computed by
inverting the multivariate normal CDF over the common-contrast correlation.

- `dunnett_m2`: `adjustedAlpha ≈ 0.026958` (looser than Bonferroni's 0.025,
  reflecting the correlation gain)
- `dunnett_m3`: `adjustedAlpha ≈ 0.018825`

Reference: Dunnett, C. W. (1955), A multiple comparison procedure for
comparing several treatments with a control, *JASA* 50(272).

## Holm (1979) — step-down

```text
alpha_adjusted(k) = alpha / (m - k + 1)
```

- `holm_gate1_of_5`: `0.05 / 5 = 0.01`
- `holm_gate5_of_5`: `0.05 / 1 = 0.05`

Reference: Holm, S. (1979), A simple sequentially rejective multiple test
procedure, *Scand. J. Statist.* 6(2).

## Hochberg (1988) — step-up

```text
alpha_adjusted(k) = alpha / k
```

- `hochberg_gate1_of_5`: `0.05 / 1 = 0.05`
- `hochberg_gate5_of_5`: `0.05 / 5 = 0.01`

Reference: Hochberg, Y. (1988), A sharper Bonferroni procedure for multiple
tests of significance, *Biometrika* 75(4).

## Graphical procedure (Bretz et al. 2011)

```text
alpha_adjusted(k) = alpha * w_k / sum(w)
```

- `graphical_equal_weights_gate2`: equal weights normalize to `w = 0.2`;
  `0.05 × 0.2 = 0.01`, `comparisonWeight = 0.2`
- `graphical_weighted_gate2`: weights `[0.5, 0.3, 0.2]` (already sum to 1);
  `0.05 × 0.3 = 0.015`, `comparisonWeight = 0.3`

Reference: Bretz, F., Maurer, W., Brannath, W. & Posch, M. (2011), A graphical
approach to sequentially rejective multiple test procedures, *Statistics in
Medicine* 28(4) [original 2009; clarified 2011].

## Tolerance

Bonferroni, Šidák, Holm, Hochberg, and graphical are closed-form exact —
tolerance ±1e-6 is purely for floating-point representation. Dunnett relies on
the MVN quantile, pinned to ±1e-4.
