# Validation Evidence

Independent validation evidence for each calculation method, per
`handbook/08-validation-testing.md`. One subfolder per method:

```text
validation/
  <endpoint-category>/
    <method-name>/
      cases.json
      reference-output.md
      validation-report.md   (optional stored snapshot)
```

The directory name maps to the method identifier: `binary/one-sample-binomial`
holds evidence for `binary.one_sample_binomial`.

## cases.json format

`cases.json` is machine-checked: every case runs through the engine's JSON
dispatch in CI (`cargo test -p clinsize-core validation_report`), is embedded
into the binaries at compile time, and feeds the app's Validation page and
`clinsize validation-report --method <id>`.

```json
{
  "methodId": "continuous.two_sample_ttest",
  "cases": [
    {
      "caseId": "equal_two_sided_d1",
      "source": "R power.t.test(delta = 1, sd = 1, sig.level = 0.05, power = 0.8)",
      "input": { "solveMode": "sample_size", "alpha": 0.05, "...": "..." },
      "expect": {
        "nControl": 17,
        "achievedPower": { "value": 0.80704, "tol": 1e-4 }
      }
    }
  ]
}
```

- `input` is the method's JSON input document, exactly as the CLI's
  `calculate --method <id> --input …` accepts it.
- `expect` keys are result fields; use `/` for nested paths
  (`looks/0/upperZBoundary` indexes into arrays).
- A bare number is compared at absolute tolerance `1e-9` (use this for
  integers); pinned floats should give `{ "value": …, "tol": … }`.
- `source` names the external reference (R command, published table) that
  produced the expected values. Record the full commands and outputs in the
  method's `reference-output.md`.

Expected values must come from the external reference, never from the engine
itself — a case that pins engine output to engine output validates nothing.
