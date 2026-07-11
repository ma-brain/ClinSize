# Survival Endpoints

## Initial Scope

Initial survival support should focus on log-rank test designs with proportional hazards assumptions.

## Inputs

- Hazard ratio or median survival assumptions.
- Alpha.
- Target power.
- Allocation ratio.
- Accrual duration.
- Follow-up duration.
- Dropout or loss-to-follow-up assumptions, if supported.
- Event target, if solving event-driven designs.

## Outputs

- Required number of events.
- Required sample size, if accrual assumptions are provided.
- Achieved power.
- Assumptions.
- Warnings.

## Assumptions

- Proportional hazards.
- Independent censoring.
- Accrual pattern is documented.
- Event rates are interpreted consistently.

## Validation

Compare against:

- Schoenfeld-style event formulas.
- Freedman-style approximations.
- R survival design packages.
- Published oncology or time-to-event examples.

## Implementation Caution

Survival sample size is often misunderstood because event count and patient count are not the same. The UI must separate required events from required enrolled subjects.

