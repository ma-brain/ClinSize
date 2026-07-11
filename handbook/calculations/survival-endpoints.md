# Survival Endpoints

## Methods

**Implemented** in `clinsize-core`:

- `survival.log_rank` — Schoenfeld (1981) two-arm log-rank required events and achieved power.

Planned next: accrual/follow-up translation to enrolled subjects, event-driven designs with dropout.

## Log-Rank Test

Uses the Schoenfeld (1981) normal approximation for the two-sample log-rank test under proportional hazards.

**Total events:**

`D = (z_α + z_β)² / [p(1 − p)(ln HR)²]`

where `p = 1 / (1 + r)` is the control-group proportion, `r` is the treatment:control allocation ratio, and `HR = λ_treatment / λ_control`.

**Achieved power** for a fixed event count `D`:

`power = Φ(√(D · p(1 − p)) · |ln HR| − z_α)`

Two-sided tests use `z_{α/2}`; one-sided tests use `z_α`. Required events are rounded up to the smallest integer meeting the target power after rounding.

Validated against R `gsDesign::nEvents`.

## Inputs

- Hazard ratio (treatment / control).
- Alpha.
- Target power or total events.
- Allocation ratio (treatment:control).
- Alternative hypothesis (two-sided or one-sided).

## Outputs

- Required total events.
- Expected events per arm.
- Achieved power.
- Assumptions and warnings.

## Assumptions

- Proportional hazards.
- Independent censoring.
- Two-arm comparison via the log-rank test.
- Lower hazard in the treatment arm (`HR < 1`) indicates benefit when longer survival is favorable.

## Validation

Compare against:

- Schoenfeld-style event formulas.
- R `gsDesign::nEvents`.
- Published oncology or time-to-event examples.

## Implementation Caution

Survival sample size is often misunderstood because event count and patient count are not the same. The UI must separate required events from required enrolled subjects. The current log-rank method reports events only; accrual and follow-up inputs are required to convert events into enrolled sample size.
