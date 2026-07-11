# Validation Evidence

Independent validation evidence for each calculation method, per
`handbook/08-validation-testing.md`. One subfolder per method:

```text
validation/
  <endpoint-category>/
    <method-name>/
      cases.csv
      reference-output.md
      validation-report.md
```

Use `handbook/templates/validation-test-plan.md` as the starting point for
`validation-report.md`. No method is implemented yet, so this directory is
otherwise empty.
