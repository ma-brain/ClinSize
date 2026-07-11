# New Calculation Method Template

## Method Name

Write the formal method name.

## Method Identifier

Use a stable identifier such as `continuous.two_sample_ttest`.

## Purpose

Describe the clinical design question this method answers.

## Endpoint Type

Continuous, binary, survival, count, ordinal, or other.

## Hypotheses

Define null and alternative hypotheses. Specify sign conventions and whether higher or lower values are favorable.

## Inputs

List each input:

- Name.
- Type.
- Units.
- Valid range.
- Default value, if any.
- Clinical interpretation.

## Outputs

List each output:

- Name.
- Type.
- Rounding.
- Interpretation.

## Formula Or Algorithm

Document the formula or numerical method. Include enough detail for independent implementation.

## Assumptions

List statistical and practical assumptions.

## Validation Rules

Define invalid inputs and expected error messages.

## Rounding Policy

Describe how sample sizes are rounded and how achieved power is recalculated.

## Reference Sources

List textbooks, papers, R packages, SAS examples, or commercial software used for validation.

## Test Cases

Define normal cases, edge cases, invalid cases, and reference cases.

## Known Limitations

State what the method does not support.

## UI Requirements

Define required form fields, result fields, warnings, sensitivity parameters, and export content.

