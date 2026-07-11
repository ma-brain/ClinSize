# Cursor Project Rules

Use these rules as persistent project instructions in Cursor.

## Project Role

You are helping build ClinSize, a cross-platform clinical trial sample size and power application using SvelteKit, Tauri, and Rust.

## Non-Negotiable Rules

- Keep statistical calculations in the Rust core crate.
- Do not implement formulas in Svelte or TypeScript.
- Do not add a method without tests and method documentation.
- Do not silently change statistical behavior.
- Do not claim a method is validated unless validation evidence exists.
- Prefer small, reviewable changes.
- Update handbook files when architectural or statistical decisions change.

## Architecture Rules

- `clinsize-core` contains pure Rust methods.
- Tauri commands are thin wrappers.
- Svelte renders forms, results, warnings, and charts.
- TypeScript DTOs must match Rust DTOs.
- UI validation is helpful but Rust validation is authoritative.

## Coding Style

- Use explicit names.
- Avoid broad abstractions before there are at least two real users of the abstraction.
- Keep modules focused.
- Add tests near the code being tested.
- Include edge cases for numerical code.

## Statistical Review Rule

For statistical methods, explain:

- Formula.
- Assumptions.
- Validation source.
- Tolerance.
- Known limitations.

If any of these are missing, stop and ask for the missing information or mark the implementation incomplete.

