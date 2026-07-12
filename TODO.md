TODO — what's left to fix

  High value
  1. Collapse the Tauri-side dispatch duplication: 38 per-method command handlers in
  apps/desktop/src-tauri/src/lib.rs (calculate_* + export_*_markdown) plus 38 literal invoke()
  call sites across 17 *View.svelte files. The core dispatch.rs side is now macro-driven; the
  Tauri+frontend side is the remaining Layer 2 of the old "add a method, forget a site" risk.

  Medium
  2. Group sequential / Dunnett numerical precision: fixed 12,000-point Halton QMC gives ~0.02 boundary accuracy
  vs gsDesign's ~1e-6. Upgrade the MVN integration (or at least surface the tolerance in exports — currently
  only documented in validation/).
  3. Frontend tests: zero unit/component tests (svelte-check only). Start with vitest on buildInput()
  round-trips and summarizeResult.

  Process
  4. Recompute any previously exported MMRM results — the old outputs are undersized by ~`(1+(k−1)ρ)/(1−ρ)`;
  group-sequential exports at "two-sided 0.05" should be re-read as one-sided 0.05 designs. (Both documented in
  the validation history notes, but worth acting on if any numbers left the building.)
