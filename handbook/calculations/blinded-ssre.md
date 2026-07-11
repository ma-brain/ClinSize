# Blinded Sample Size Re-estimation

## Method Identifier

`design.blinded_ssre`

## Purpose

Plan a continuous two-sample superiority trial that includes one blinded interim
look for variance re-estimation. Computes the initial planned sample size, the
re-estimated sample size from a blinded pooled interim standard deviation, and
the capped final enrollment after applying a pre-specified maximum inflation.

## Endpoint Type

Continuous (equal-variance two-sample t-test).

## Hypotheses

Superiority test of treatment minus control mean difference `Δ` with Type I
error `α` and target power `1 − β` at the planning stage.

## Inputs

- `alpha`: Two-sided or one-sided Type I error rate.
- `targetPower`: Target power for the initial sample size calculation.
- `meanDifference`: Planned treatment minus control mean difference (`Δ`).
- `plannedStandardDeviation`: Assumed common within-group SD at design (`σ₀`).
- `blindedInterimStandardDeviation` (optional): Hypothetical blinded pooled
  interim SD (`s_b`) for what-if re-estimation. Defaults to `σ₀`.
- `interimFraction`: Fraction `τ` of planned per-arm enrollment at the interim
  look (0 < τ < 1).
- `allocationRatio`: Treatment-to-control allocation ratio.
- `maxSampleSizeMultiplier`: Maximum allowed inflation relative to planned
  per-arm sample size (≥ 1).
- `alternative`: `two_sided`, `greater`, or `less`.

## Outputs

- `plannedNControl`, `plannedNTreatment`, `plannedTotalN`: Initial design.
- `interimNControl`, `interimNTreatment`, `interimTotalN`: Subjects at interim.
- `reEstimatedNControl`, `reEstimatedNTreatment`, `reEstimatedTotalN`: After
  blinded variance update (before cap).
- `cappedNControl`, `cappedNTreatment`, `cappedTotalN`: Final enrollment cap.
- `sampleSizeInflationFactor`: Re-estimated total N / planned total N.
- `cappedInflationFactor`: Capped total N / planned total N.
- `varianceRatio`: `(s_b / σ₀)²`.
- `achievedPowerAtCapped`: Power at capped allocation using planned `Δ` and
  `σ₀` (optimistic when the interim SD is higher).
- `achievedPowerAtCappedInterimSd`: Power at capped allocation using planned
  `Δ` and the blinded interim SD `s_b` — the realistic estimate when the cap
  binds.
- `wasCapped`: Whether the cap reduced the re-estimated sample size.
- `warnings`: Assumption notes.

## Formula Or Algorithm

**Step 1 — Planned sample size**

Solve for the smallest integer per-arm allocation achieving `targetPower` with
the equal-variance two-sample t-test using `σ₀` (same engine as
`continuous.two_sample_ttest`).

**Step 2 — Blinded variance re-estimation (Friede-Kieser)**

At interim fraction `τ`, observe blinded pooled SD `s_b` from all subjects
pooled across arms. Re-estimate per-arm sample size:

```text
n_re = ceil(n₀ × (s_b / σ₀)²)
```

Treatment allocation follows the same ratio as the planned design.

**Step 3 — Cap**

```text
n_cap = min(n_re, ceil(n₀ × max_multiplier))
```

**Step 4 — Interim timing**

```text
n_interim = ceil(n₀ × τ)
```

Achieved power at capped N is recalculated twice: with planned `Δ` and `σ₀`,
and with planned `Δ` and the blinded interim SD `s_b`. When the cap binds,
the interim-SD power is the realistic estimate and the `cap_applied` warning
states the shortfall against the target.

## Assumptions

- Independent observations with a common within-group variance.
- Blinded pooled interim SD updates variance only; `Δ` is held fixed.
- One interim look at fraction `τ` of planned enrollment.
- Friede-Kieser blinded SSR maintains nominal Type I error under the stated
  model; this release reports the deterministic re-estimation rule only (no
  simulation).

## Validation Rules

- `alpha` in (0, 1); `targetPower` in (0, 1).
- `plannedStandardDeviation` and optional `blindedInterimStandardDeviation`
  must be positive.
- `meanDifference` must be non-zero.
- `interimFraction` in (0, 1).
- `maxSampleSizeMultiplier` ≥ 1.

## Rounding Policy

Per-arm sample sizes round up (`ceil`) after variance scaling and cap
application. Interim per-arm N rounds up from `τ × n₀`.

## Reference Sources

- Friede, M., & Kieser, M. (2006). Sample size recalculation for the t-test.
  *Biometrical Journal*, 48(4), 590–599.
- Planned sample size: R `stats::power.t.test`.
- Re-estimation arithmetic: manual R reference scripts in unit tests.

## Test Cases

| Scenario | σ₀ | s_b | n₀ (per arm) | n_re | n_cap (×1.5) |
| --- | --- | --- | --- | --- | --- |
| No change | 1 | 1 | 17 | 17 | 17 |
| 20% SD increase | 1 | 1.2 | 17 | 25 | 25 |
| 50% SD increase, capped | 1 | 1.5 | 17 | 39 | 26 |

Power at capped/re-estimated N (Δ=1, σ₀=1, α=0.05; R `power.t.test`):

| Scenario | n | SD used | Power |
| --- | --- | --- | --- |
| s_b=1.2, not capped | 25 | σ₀=1 | 0.93371 |
| s_b=1.2, not capped | 25 | s_b=1.2 | 0.82301 |
| s_b=1.5, capped | 26 | σ₀=1 | 0.94243 |
| s_b=1.5, capped | 26 | s_b=1.5 | **0.65445** (underpowered vs 0.8 target) |

## Known Limitations

- Continuous two-sample t-test only; binary and survival SSR not implemented.
- Single blinded interim look; no repeated or unblinded re-estimation.
- CHW/CROS/Mehta-Gould exact procedures are out of scope for v1.
- Does not simulate operating characteristics or Type I error inflation.
- Does not combine with group-sequential alpha spending in this release.

## UI Requirements

- Inputs grouped by planning assumptions and interim re-estimation scenario.
- Results: planned, interim, re-estimated, and capped sample sizes; inflation
  factors; achieved power at capped N; cap indicator.
- Warnings for blinded-variance-only rule, single interim look, cap applied.
- Export via standard Markdown/HTML/Word workflow.
