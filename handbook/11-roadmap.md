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

- Log-rank test.
- Accrual and follow-up assumptions.
- Event-driven sample size.
- Dropout and censoring assumptions.

## Phase 5: Professional Workflow

- Calculation history.
- Project files.
- Scenario comparison.
- PDF and Word export.
- Validation report generation.

## Phase 6: Extended Platforms

- Windows installer.
- Linux package.
- CLI tool.
- Potential mobile or web prototype using the same core engine.

## Backlog

- Multiplicity adjustments.
- Group sequential designs.
- Adaptive design utilities.
- Blinded sample size re-estimation.
- R or Python bindings.
- Quarto report generation.

