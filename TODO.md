TODO — what's left to fix

  High value
  1. Wire the validation report generator to the new evidence. validation_report/mod.rs only supports two-sample
  t-test and log-rank; the app's Validation page and clinsize validation-report can't see the new
  MMRM/GSD/binomial/SSRE cases. Generalize it to read any cases.csv.
  2. Validation evidence still missing for 7 methods: Mann-Whitney, Wilcoxon signed-rank, paired t-test,
  change-from-baseline, negative binomial, proportional odds, multiplicity (Dunnett/Holm/Hochberg/graphical).
  All have R-checkable references (Hmisc, gsDesign, mvtnorm, power.t.test).
  3. Tauri hardening: "csp": null in tauri.conf.json; read_project_file/write_project_file accept arbitrary
  paths from the renderer; generate_validation_report builds its path from compile-time CARGO_MANIFEST_DIR, so
  that feature silently breaks in any packaged install.
  4. Collapse the dispatch duplication (~1,500 lines): the 19-method match is hand-written 4× in dispatch.rs,
  plus 2 Tauri commands per method. A macro or function-pointer table removes the "add a method, forget a site"
  bug class that caused the stale CLI test.

  Medium
  5. Group sequential / Dunnett numerical precision: fixed 12,000-point Halton QMC gives ~0.02 boundary accuracy
  vs gsDesign's ~1e-6. Upgrade the MVN integration (or at least surface the tolerance in exports — currently
  only documented in validation/).
  6. find_minimum_integer monotonicity guard — binary search over power assumes a monotone predicate; add a
  walk-down check after the search.
  7. Frontend tests: zero unit/component tests (svelte-check only). Start with vitest on buildInput()
  round-trips and summarizeResult.
  8. Project history shows "—" for one-sample, paired, multiplicity, and group-sequential records
  (summarizeResult only knows totalN).

  Low / cleanup
  9. odds_ratio/risk_ratio validate() don't reject control_n in sample-size mode or power in power mode (every
  other method does).
  10. iso_timestamp() returns raw Unix seconds despite the name (project files show "createdAt": "1752…");
  newId() can collide within one millisecond.
  11. InflationHeuristic one-impl trait in group_sequential.rs (violates the repo's own "no premature
  abstraction" rule); design methods list empty solve modes in clinsize list.
  12. Soften the "validated Rust statistical engine" claim in tauri.conf.json until item 2 closes the gap.

  Process
  13. Merge PR #3 once you've reviewed it — CI is green and branch protection now requires it.
  14. Recompute any previously exported MMRM results — the old outputs are undersized by ~`(1+(k−1)ρ)/(1−ρ)`;
  group-sequential exports at "two-sided 0.05" should be re-read as one-sided 0.05 designs. (Both documented in
  the validation history notes, but worth acting on if any numbers left the building.)