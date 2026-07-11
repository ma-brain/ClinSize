# Two-sample t-test reference output

Reference source: R 4.x `stats::power.t.test`, type = `"two.sample"`, equal allocation.

## Case: equal_two_sided_d1

```r
power.t.test(delta = 1, sd = 1, sig.level = 0.05, power = 0.8, type = "two.sample")
```

- Continuous n per group: 16.714768
- Rounded control N: 17
- Achieved power at N = 17: 0.8070359

## Case: equal_two_sided_d05

```r
power.t.test(delta = 0.5, sd = 1, sig.level = 0.05, power = 0.8, type = "two.sample")
```

- Continuous n per group: 63.765764
- Rounded control N: 64
- Achieved power at N = 64: 0.8014596

## Case: one_sided_d1

```r
power.t.test(delta = 1, sd = 1, sig.level = 0.05, power = 0.8,
             type = "two.sample", alternative = "one.sided")
```

- Continuous n per group: 13.097773
- Rounded control N: 14
- Achieved power at N = 14: 0.8240859

## Tolerance

Achieved power comparisons use absolute tolerance `1e-4` unless R rounds sample
size differently; sample-size integers must match exactly.
