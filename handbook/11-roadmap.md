# Roadmap

## Phase 0: Project Foundation

- Create Rust workspace.
- Create SvelteKit + Tauri desktop app.
- Add core crate.
- Add command boundary.
- Add test and lint commands.
- Add handbook and AI rules.

## Phase 1: First Validated Method

- Implement two-sample t-test sample size.
- Add input validation.
- Add achieved power calculation.
- Add reference tests.
- Build first Svelte method page.
- Export calculation summary as Markdown or HTML.

## Phase 2: Continuous Endpoints

- One-sample t-test.
- Paired t-test.
- One-way ANOVA.
- Basic ANCOVA adjustment.
- Sensitivity analysis views. **Done** — collapsible one-parameter sweeps on each method screen.

## Phase 3: Binary Endpoints

- Difference in proportions. **Done** — `binary.two_proportion_difference` with superiority and basic non-inferiority.
- Risk ratio. **Done** — `binary.risk_ratio`.
- Odds ratio. **Done** — `binary.odds_ratio`.
- Superiority designs. **Done** — supported on all three binary methods.
- Basic non-inferiority designs. **Done** — proportion difference only.

## Phase 4: Survival Endpoints

- Log-rank test. **Done** — `survival.log_rank` with desktop UI, sensitivity analysis, and accrual-based enrollment sizing.
- Accrual and follow-up assumptions. **Done** — optional inputs on `survival.log_rank`.
- Event-driven sample size. **Done** — events-to-enrollment translation on `survival.log_rank`.
- Dropout and censoring assumptions. **Done (basic)** — optional exponential dropout hazard on `survival.log_rank`.

## Phase 5: Professional Workflow

- Calculation history. **Done** — auto-recorded on each method calculation into the active project.
- Project files. **Done** — JSON `.clinsize.json` save/open from the Project page.
- Scenario comparison. **Done** — group history entries and compare primary outputs.
- PDF and Word export. **Done** — HTML and Word-compatible HTML exports; PDF via printable HTML.
- Validation report generation. **Done** — automated reports for two-sample t-test and log-rank.

## Phase 6: Extended Platforms

- Windows installer. **Done** — NSIS bundle via Tauri; release workflow on `windows-latest`.
- Linux package. **Done** — `.deb` and AppImage via Tauri; release workflow on `ubuntu-22.04`.
- CLI tool. **Done** — `clinsize` binary with `list`, `calculate`, `report`, and `validation-report`.
- Potential mobile or web prototype using the same core engine. **Deferred** — documented in `handbook/extended-platforms.md`.

## Backlog

- Multiplicity adjustments. **In progress** — Bonferroni, Šidák, Dunnett, and Holm gatekeeping via `design.multiplicity`; Hochberg and graphical gatekeeping remain.
- Group sequential designs.
- Adaptive design utilities.
- Blinded sample size re-estimation.
- R or Python bindings.
- Quarto report generation.

