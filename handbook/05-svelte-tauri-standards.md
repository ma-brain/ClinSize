# Svelte and Tauri Standards

## Scope

The desktop app provides forms, navigation, result views, sensitivity plots, exports, settings, and Tauri command bindings. It should not duplicate statistical formulas.

## SvelteKit Role

SvelteKit should handle:

- Routes and pages.
- Layouts.
- Form state.
- Client-side validation hints.
- Result rendering.
- Charts and sensitivity visualizations.
- User preferences.

## Tauri Role

Tauri should handle:

- Native desktop shell.
- Calls from TypeScript to Rust.
- File dialogs.
- Export paths.
- Application settings storage.
- Logging.
- Auto-update infrastructure, if enabled.

## Boundary Rule

TypeScript can validate for user experience, but Rust validation is authoritative. A calculation request must still be validated in Rust even if the UI appears to prevent invalid input.

## TypeScript Models

Define TypeScript models matching Rust DTOs:

```ts
export type Alternative = "two_sided" | "less" | "greater";

export interface TwoSampleTTestInput {
  alpha: number;
  power?: number;
  sampleSizePerGroup?: number;
  meanDifference: number;
  standardDeviation: number;
  allocationRatio: number;
  alternative: Alternative;
}
```

Keep DTO names stable. If a Rust DTO changes, update TypeScript types and tests in the same change.

## Form Pattern

Every calculation page should provide:

- Method title.
- Short method context.
- Parameter form.
- Solve mode selector, such as sample size or power.
- Inline validation messages.
- Calculate action.
- Result summary.
- Assumptions and warnings.
- Sensitivity view.
- Export action.

## State Management

Use local component state for simple forms. Use Svelte stores only for shared state such as:

- Selected method.
- User preferences.
- Recent calculations.
- Theme.
- Project/session state.

Avoid global stores for data that belongs to a single page.

## Tauri Commands

Tauri commands should be thin:

```rust
#[tauri::command]
fn calculate_two_sample_ttest(input: TwoSampleTTestInput) -> Result<TwoSampleTTestResult, AppError> {
    clinsize_core::methods::continuous::two_sample_ttest::calculate(input).map_err(AppError::from)
}
```

Do not place calculation formulas in command handlers.

## Cross-Platform Notes

Avoid platform-specific assumptions in UI code:

- Do not assume path separators.
- Do not assume a default documents folder exists.
- Do not hard-code macOS-only keyboard labels.
- Use Tauri APIs for dialogs and filesystem interactions.

