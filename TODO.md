TODO — what's left to fix

  High value
  1. Collapse the dispatch duplication (~1,500 lines): the 19-method match is hand-written 4× in dispatch.rs,
  plus 2 Tauri commands per method. A macro or function-pointer table removes the "add a method, forget a site"
  bug class that caused the stale CLI test.

  Medium
  2. Group sequential / Dunnett numerical precision: fixed 12,000-point Halton QMC gives ~0.02 boundary accuracy
  vs gsDesign's ~1e-6. Upgrade the MVN integration (or at least surface the tolerance in exports — currently
  only documented in validation/).
  3. find_minimum_integer monotonicity guard — binary search over power assumes a monotone predicate; add a
  walk-down check after the search.
  4. Frontend tests: zero unit/component tests (svelte-check only). Start with vitest on buildInput()
  round-trips and summarizeResult.
  5. Project history shows "—" for one-sample, paired, multiplicity, and group-sequential records
  (summarizeResult only knows totalN).

  Low / cleanup
  6. odds_ratio/risk_ratio validate() don't reject control_n in sample-size mode or power in power mode (every
  other method does).
  7. iso_timestamp() returns raw Unix seconds despite the name (project files show "createdAt": "1752…");
  newId() can collide within one millisecond.
  8. InflationHeuristic one-impl trait in group_sequential.rs (violates the repo's own "no premature
  abstraction" rule); design methods list empty solve modes in clinsize list.
  Process
  9. Recompute any previously exported MMRM results — the old outputs are undersized by ~`(1+(k−1)ρ)/(1−ρ)`;
  group-sequential exports at "two-sided 0.05" should be re-read as one-sided 0.05 designs. (Both documented in
  the validation history notes, but worth acting on if any numbers left the building.)
