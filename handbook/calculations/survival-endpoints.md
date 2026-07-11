# Survival Endpoints

## Methods

**Implemented** in `clinsize-core`:

- `survival.log_rank` — Schoenfeld (1981) two-arm log-rank required events, achieved power, and optional enrollment sizing from accrual assumptions.

## Log-Rank Test

Uses the Schoenfeld (1981) normal approximation for the two-sample log-rank test under proportional hazards.

**Total events:**

`D = (z_α + z_β)² / [p(1 − p)(ln HR)²]`

where `p = 1 / (1 + r)` is the control-group proportion, `r` is the treatment:control allocation ratio, and `HR = λ_treatment / λ_control`.

**Achieved power** for a fixed event count `D`:

`power = Φ(√(D · p(1 − p)) · |ln HR| − z_α)`

Two-sided tests use `z_{α/2}`; one-sided tests use `z_α`. Required events are rounded up to the smallest integer meeting the target power after rounding.

Validated against R `gsDesign::nEvents`.

## Accrual And Follow-Up

When control hazard rate, accrual duration, and minimum follow-up are provided, ClinSize translates required events into enrolled subjects using the Lachin and Foulkes (1986) uniform-accrual exponential model.

**Per-arm event probability:**

`P = λ / (λ + η) × (1 − exp(−(λ + η)F) × (1 − exp(−(λ + η)A)) / ((λ + η)A))`

where `λ` is the arm-specific event hazard, `η` is the dropout hazard (0 when omitted), `A` is accrual duration, and `F` is minimum follow-up. Study duration is `A + F`.

**Enrollment:**

Solve for the smallest total enrollment `N` such that

`N_c P_c + N_t P_t ≥ D`

with `N_c = N / (1 + r)` and `N_t = N r / (1 + r)`.

Validated against R `gsDesign::nSurv` (Schoenfeld and LachinFoulkes methods).

## Inputs

- Hazard ratio (treatment / control).
- Alpha.
- Target power or total events.
- Allocation ratio (treatment:control).
- Alternative hypothesis (two-sided or one-sided).
- Optional control hazard rate, accrual duration, minimum follow-up, and dropout hazard for enrollment sizing.

## Outputs

- Required total events.
- Expected events per arm.
- Achieved power.
- Enrolled subjects per arm and total, when accrual inputs are provided.
- Per-arm event probabilities when accrual inputs are provided.
- Assumptions and warnings.

## Assumptions

- Proportional hazards.
- Independent censoring.
- Two-arm comparison via the log-rank test.
- Lower hazard in the treatment arm (`HR < 1`) indicates benefit when longer survival is favorable.
- Uniform accrual over the accrual period when enrollment sizing is requested.
- Exponential event and dropout hazards for enrollment sizing.

## Validation

Compare against:

- Schoenfeld-style event formulas.
- R `gsDesign::nEvents` and `gsDesign::nSurv`.
- Published oncology or time-to-event examples.

## Implementation Caution

Survival sample size is often misunderstood because event count and patient count are not the same. The UI separates required events from enrolled subjects. Enrollment sizing requires explicit accrual and follow-up assumptions.
