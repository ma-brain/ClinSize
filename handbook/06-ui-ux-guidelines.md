# UI and UX Guidelines

## Design Goal

ClinSize should look like a professional statistical workbench. The interface should support repeated use, fast scanning, and careful review of assumptions.

## Primary Layout

Recommended desktop layout:

```text
Navigation rail | Parameter panel | Results and plots
```

The left side should help users choose endpoint category and method. The center should hold inputs. The right side should show results, warnings, assumptions, and charts.

## Interaction Principles

- Users should be able to calculate without navigating through modal dialogs.
- Changing an input should clearly mark results as stale or recalculate when safe.
- Errors should appear next to the relevant field.
- Warnings should be visible in the result area and exported report.
- Sensitivity analysis should be one click away from the base calculation.

## Visual Style

Use a restrained professional style:

- Neutral background with a shared design token layer (`apps/desktop/src/app.css`).
- Source Serif 4 for method titles and primary results; IBM Plex Sans for UI text.
- Compact panels with a dedicated results column on method screens.
- Tables where comparison matters.
- Charts with readable axes and labels; a single accent gradient on sensitivity lines only.
- Minimal decoration on chrome; no marketing hero pages.

Avoid marketing-style hero pages, oversized cards, decorative gradients on navigation or panels, and layouts that waste space.

## Calculation Form Pattern

Each method screen should include:

- Method selector.
- Endpoint category.
- Hypothesis type.
- Solve mode.
- Inputs grouped by clinical meaning.
- Advanced options collapsed by default.
- Result summary.
- Assumptions and warnings.
- Sensitivity chart.
- Export controls.

## Result Display

Results should show:

- Required N or achieved power as the primary output.
- Group-level sample sizes.
- Total sample size.
- Achieved power after rounding.
- Effect size.
- Dropout-adjusted N when applicable.
- Assumptions.
- Warnings.
- Method reference.

## Sensitivity Analysis

At minimum, support one-way sensitivity analysis over:

- Effect size.
- Standard deviation.
- Alpha.
- Power.
- Allocation ratio.
- Dropout rate.

**Implemented** in the desktop app as a collapsible sensitivity panel on each method
screen. The UI varies one parameter at a time across a fixed sweep range and
re-runs the existing Rust `calculate_*` command for each point. No statistical
logic is duplicated in TypeScript.

Scenario comparison is available on the **Scenarios** page using saved project
history entries.

## Accessibility

Use semantic form controls, visible focus states, readable contrast, keyboard navigation, and labels for all inputs. Do not rely on color alone for warnings or invalid states.

## Clinical Trial Language

Use terms familiar to clinical trial users:

- Treatment group.
- Control group.
- Allocation ratio.
- Superiority.
- Non-inferiority.
- Equivalence.
- Dropout.
- Achieved power.
- Type I error.

Avoid casual language in outputs and reports.

