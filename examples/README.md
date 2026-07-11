# Examples

Example inputs, outputs, and exported reports for calculation methods. Use
these with the CLI and for manual review:

```bash
just cli calculate --method continuous.two_sample_ttest \
  --input examples/continuous/two-sample-ttest/sample-size.json
```

Each method folder should contain at least one JSON input file. Run
`clinsize calculate` to produce result JSON, then `clinsize report` to
render a Markdown summary.
