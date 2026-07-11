# Professional Workflow

## Calculation History

Each successful method calculation is appended to the active project history with:

- Method identifier and display name.
- Input and result payloads.
- A compact summary for tables and scenario comparison.
- Timestamp metadata.

History is visible on the **Project / history** page.

## Project Files

Projects are stored as JSON (`.clinsize.json`) containing:

- Project metadata.
- Calculation history.
- Named scenarios referencing history entries.

Use **New project**, **Open project**, and **Save project** on the Project page.

## Scenario Comparison

On the **Scenarios** page, select history entries and group them into named scenarios. ClinSize renders a comparison table of primary and secondary summary outputs.

## Export Formats

Method pages expose export actions for:

- Markdown (native calculation summary).
- HTML (printable document).
- Word (HTML document with `.doc` extension).
- PDF (printable HTML; open and print to PDF from the system viewer).

Exports use the native save dialog.

## Validation Reports

The **Validation reports** page generates Markdown reports from `validation/` evidence. Automated pass/fail execution is implemented for:

- `continuous.two_sample_ttest`
- `survival.log_rank`

Other methods return an unsupported-method error until runners are added.
