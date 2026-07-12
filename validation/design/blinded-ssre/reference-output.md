# Blinded sample size re-estimation reference output

The Friede & Kieser (2006) re-estimation arithmetic is deterministic
(`n_re = ceil(n₀ × (s_b/σ₀)²)`, capped at `ceil(n₀ × multiplier)`); the power
values are validated against R `stats::power.t.test`:

```r
# planned design: n = 17 per arm
power.t.test(delta = 1, sd = 1, sig.level = 0.05, power = 0.8)

# power at the re-estimated / capped allocation
power.t.test(n = 25, delta = 1, sd = 1.2, sig.level = 0.05)  # 0.8230090
power.t.test(n = 26, delta = 1, sd = 1.5, sig.level = 0.05)  # 0.6544534
power.t.test(n = 25, delta = 1, sd = 1.0, sig.level = 0.05)  # 0.9337100 (planned-SD view)
power.t.test(n = 26, delta = 1, sd = 1.0, sig.level = 0.05)  # 0.9424303 (planned-SD view)
```

## History

Before the 2026-07 fix, the result reported only
`achievedPowerAtCapped` computed under the **planned** SD σ₀. When the cap
binds precisely because the blinded interim SD is higher, that number is
optimistic: in the 50%-SD-increase case above it reads 94.2% while power
under the interim SD is 65.4% against an 80% target. The result now carries
both values, and the `cap_applied` warning states the interim-SD power and
the target so the shortfall is visible in the UI and all exports.
