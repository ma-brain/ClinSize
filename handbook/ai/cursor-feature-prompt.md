# Cursor Feature Prompt

Use this prompt when asking Cursor to build a feature.

```text
We are building ClinSize, a cross-platform clinical trial sample size and power desktop app using SvelteKit, Tauri, and Rust.

Relevant handbook rules:
- Statistical formulas belong in the Rust core crate.
- Svelte handles UI only.
- Tauri commands are thin wrappers.
- Inputs and outputs must be typed.
- Every method needs validation, tests, assumptions, and warnings.

Task:
[Describe the feature.]

Expected behavior:
[List user-facing behavior.]

Files likely involved:
[List likely files or ask Cursor to inspect.]

Testing:
- Add or update Rust tests for calculation behavior.
- Add UI tests if UI behavior changes.
- Run relevant checks.

Constraints:
- Keep the change small and reviewable.
- Do not introduce unrelated refactors.
- Do not change statistical behavior outside this task.
```

