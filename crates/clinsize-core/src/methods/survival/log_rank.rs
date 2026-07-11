//! Two-arm log-rank test event count (Schoenfeld approximation).

use serde::{Deserialize, Serialize};

use super::shared::{
    event_probability_uniform_accrual, schoenfeld_events, schoenfeld_power,
    solve_enrolled_subjects, split_enrollment, split_events,
};
use crate::error::{Error, Result};
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for a two-arm log-rank superiority design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRankInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    /// Target power when solving for required events.
    pub power: Option<f64>,
    /// Observed or planned total events when solving for power.
    pub total_events: Option<u32>,
    /// Treatment hazard divided by control hazard (`lambda_t / lambda_c`).
    pub hazard_ratio: f64,
    /// Treatment-to-control allocation ratio.
    pub allocation_ratio: f64,
    pub alternative: Alternative,
    /// Control-group event hazard (`lambda_c`) for accrual-based enrollment sizing.
    pub control_hazard_rate: Option<f64>,
    /// Uniform accrual duration.
    pub accrual_duration: Option<f64>,
    /// Minimum follow-up after the last enrolled subject.
    pub minimum_follow_up: Option<f64>,
    /// Exponential loss-to-follow-up hazard applied to both arms.
    pub dropout_hazard_rate: Option<f64>,
}

/// Results for a two-arm log-rank design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRankResult {
    pub required_events: u32,
    pub events_control: u32,
    pub events_treatment: u32,
    pub achieved_power: f64,
    pub hazard_ratio: f64,
    pub n_control: Option<u32>,
    pub n_treatment: Option<u32>,
    pub total_n: Option<u32>,
    pub probability_event_control: Option<f64>,
    pub probability_event_treatment: Option<f64>,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &LogRankInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_positive("allocation_ratio", input.allocation_ratio)?;
    validation::validate_positive("hazard_ratio", input.hazard_ratio)?;

    if (input.hazard_ratio - 1.0).abs() < f64::EPSILON {
        return Err(Error::InvalidInput {
            field: "hazardRatio".into(),
            message: "must differ from 1".into(),
        });
    }

    match input.solve_mode {
        SolveMode::SampleSize => {
            let power = input.power.ok_or_else(|| Error::InvalidInput {
                field: "power".into(),
                message: "is required when solving for required events".into(),
            })?;
            validation::validate_power(power)?;
            if input.total_events.is_some() {
                return Err(Error::InvalidInput {
                    field: "totalEvents".into(),
                    message: "must not be set when solving for required events".into(),
                });
            }
        }
        SolveMode::Power => {
            let total_events = input.total_events.ok_or_else(|| Error::InvalidInput {
                field: "totalEvents".into(),
                message: "is required when solving for power".into(),
            })?;
            if total_events < 1 {
                return Err(Error::InvalidInput {
                    field: "totalEvents".into(),
                    message: "must be at least 1".into(),
                });
            }
            if input.power.is_some() {
                return Err(Error::InvalidInput {
                    field: "power".into(),
                    message: "must not be set when solving for power".into(),
                });
            }
        }
        SolveMode::DetectableEffect => {
            return Err(Error::UnsupportedMethod(
                "detectable effect solve mode is not implemented for log-rank test".into(),
            ));
        }
    }

    validate_accrual_inputs(input)?;

    Ok(())
}

fn validate_accrual_inputs(input: &LogRankInput) -> Result<()> {
    let accrual_fields = [
        ("controlHazardRate", input.control_hazard_rate),
        ("accrualDuration", input.accrual_duration),
        ("minimumFollowUp", input.minimum_follow_up),
    ];

    let provided = accrual_fields
        .iter()
        .filter(|(_, value)| value.is_some())
        .count();

    if provided == 0 {
        if input.dropout_hazard_rate.is_some() {
            return Err(Error::InvalidInput {
                field: "dropoutHazardRate".into(),
                message: "requires control hazard rate, accrual duration, and minimum follow-up"
                    .into(),
            });
        }
        return Ok(());
    }

    if provided != 3 {
        return Err(Error::InvalidInput {
            field: "controlHazardRate".into(),
            message: "control hazard rate, accrual duration, and minimum follow-up must all be provided together".into(),
        });
    }

    let control_hazard = input.control_hazard_rate.expect("checked");
    let accrual_duration = input.accrual_duration.expect("checked");
    let minimum_follow_up = input.minimum_follow_up.expect("checked");

    validation::validate_positive("control_hazard_rate", control_hazard)?;
    validation::validate_positive("accrual_duration", accrual_duration)?;
    validation::validate_positive("minimum_follow_up", minimum_follow_up)?;

    if let Some(dropout) = input.dropout_hazard_rate {
        if dropout < 0.0 {
            return Err(Error::InvalidInput {
                field: "dropoutHazardRate".into(),
                message: "must be at least 0".into(),
            });
        }
    }

    Ok(())
}

pub fn calculate(input: LogRankInput) -> Result<LogRankResult> {
    validate(&input)?;

    let accrual_enabled = input.control_hazard_rate.is_some();
    let mut warnings = vec![
        CalculationWarning::new(
            "proportional_hazards",
            "Assumes proportional hazards and independent censoring.",
        ),
        CalculationWarning::new(
            "schoenfeld_approximation",
            "Uses the Schoenfeld (1981) normal approximation for log-rank event counts; required events are rounded up to the next integer.",
        ),
    ];

    if accrual_enabled {
        warnings.push(CalculationWarning::new(
            "uniform_accrual",
            "Enrollment sizing assumes uniform accrual over the accrual period and exponential event and dropout hazards (Lachin and Foulkes 1986).",
        ));
        warnings.push(CalculationWarning::new(
            "study_duration",
            "Study duration is taken as accrual duration plus minimum follow-up.",
        ));
    } else {
        warnings.push(CalculationWarning::new(
            "events_not_subjects",
            "Reports required events only. Provide control hazard rate, accrual duration, and minimum follow-up to translate events into enrolled subjects.",
        ));
    }

    let (required_events, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let events = solve_required_events(&input, target_power)?;
            let power = schoenfeld_power(
                events,
                input.hazard_ratio,
                input.alpha,
                input.allocation_ratio,
                input.alternative,
            );
            (events, power)
        }
        SolveMode::Power => {
            let events = input.total_events.expect("validated");
            let power = schoenfeld_power(
                events,
                input.hazard_ratio,
                input.alpha,
                input.allocation_ratio,
                input.alternative,
            );
            (events, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    let (events_control, events_treatment) = split_events(required_events, input.allocation_ratio);

    let (n_control, n_treatment, total_n, probability_event_control, probability_event_treatment) =
        if accrual_enabled {
            let control_hazard = input.control_hazard_rate.expect("validated");
            let accrual_duration = input.accrual_duration.expect("validated");
            let minimum_follow_up = input.minimum_follow_up.expect("validated");
            let dropout_hazard = input.dropout_hazard_rate.unwrap_or(0.0);
            let treatment_hazard = control_hazard * input.hazard_ratio;

            let p_control = event_probability_uniform_accrual(
                control_hazard,
                dropout_hazard,
                accrual_duration,
                minimum_follow_up,
            );
            let p_treatment = event_probability_uniform_accrual(
                treatment_hazard,
                dropout_hazard,
                accrual_duration,
                minimum_follow_up,
            );
            let total = solve_enrolled_subjects(
                required_events,
                p_control,
                p_treatment,
                input.allocation_ratio,
            );
            let (n_control, n_treatment) = split_enrollment(total, input.allocation_ratio);

            if dropout_hazard > 0.0 {
                warnings.push(CalculationWarning::new(
                    "dropout_hazard",
                    "Dropout is modeled as an independent exponential loss-to-follow-up hazard.",
                ));
            }

            (
                Some(n_control),
                Some(n_treatment),
                Some(total),
                Some(p_control),
                Some(p_treatment),
            )
        } else {
            (None, None, None, None, None)
        };

    Ok(LogRankResult {
        required_events,
        events_control,
        events_treatment,
        achieved_power,
        hazard_ratio: input.hazard_ratio,
        n_control,
        n_treatment,
        total_n,
        probability_event_control,
        probability_event_treatment,
        warnings,
    })
}

fn solve_required_events(input: &LogRankInput, target_power: f64) -> Result<u32> {
    let continuous = schoenfeld_events(
        input.hazard_ratio,
        input.alpha,
        target_power,
        input.allocation_ratio,
        input.alternative,
    );
    let min_events = continuous.ceil().max(1.0) as u32;

    find_minimum_integer(min_events, MAX_SAMPLE_SIZE_SEARCH, |events| {
        schoenfeld_power(
            events,
            input.hazard_ratio,
            input.alpha,
            input.allocation_ratio,
            input.alternative,
        ) >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find required events up to {MAX_SAMPLE_SIZE_SEARCH} meeting target power"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn base_input() -> LogRankInput {
        LogRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            total_events: None,
            hazard_ratio: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            control_hazard_rate: None,
            accrual_duration: None,
            minimum_follow_up: None,
            dropout_hazard_rate: None,
        }
    }

    #[test]
    fn sample_size_two_sided_matches_gsdesign() {
        let result = calculate(base_input()).expect("calculate");
        assert_eq!(result.required_events, 66);
        assert_eq!(result.events_control, 33);
        assert_eq!(result.events_treatment, 33);
        assert_relative_eq!(result.achieved_power, 0.803_894_1, epsilon = 1e-5);
    }

    #[test]
    fn sample_size_one_sided_matches_gsdesign() {
        let mut input = base_input();
        input.alternative = Alternative::Greater;
        let result = calculate(input).expect("calculate");
        assert_eq!(result.required_events, 52);
        assert_relative_eq!(result.achieved_power, 0.803_537, epsilon = 1e-5);
    }

    #[test]
    fn sample_size_unequal_allocation_matches_gsdesign() {
        let mut input = base_input();
        input.allocation_ratio = 2.0;
        let result = calculate(input).expect("calculate");
        assert_eq!(result.required_events, 74);
        assert_eq!(result.events_control, 25);
        assert_eq!(result.events_treatment, 49);
    }

    #[test]
    fn power_mode_matches_gsdesign() {
        let input = LogRankInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            total_events: Some(66),
            hazard_ratio: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            control_hazard_rate: None,
            accrual_duration: None,
            minimum_follow_up: None,
            dropout_hazard_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.required_events, 66);
        assert_relative_eq!(result.achieved_power, 0.803_894_1, epsilon = 1e-5);
    }

    #[test]
    fn rejects_hazard_ratio_of_one() {
        let mut input = base_input();
        input.hazard_ratio = 1.0;
        assert!(calculate(input).is_err());
    }

    #[test]
    fn smaller_hazard_ratio_needs_fewer_events_than_mild_effect() {
        let strong = calculate(base_input()).expect("calculate");
        let mut mild_input = base_input();
        mild_input.hazard_ratio = 0.75;
        let mild = calculate(mild_input).expect("calculate");
        assert!(strong.required_events < mild.required_events);
        assert_eq!(mild.required_events, 380);
    }

    #[test]
    fn accrual_inputs_translate_events_to_enrolled_subjects() {
        let mut input = base_input();
        input.control_hazard_rate = Some(2.0_f64.ln() / 6.0);
        input.accrual_duration = Some(12.0);
        input.minimum_follow_up = Some(18.0);

        let result = calculate(input).expect("calculate");
        assert_eq!(result.required_events, 66);
        assert_eq!(result.total_n, Some(79));
        assert_eq!(result.n_control, Some(40));
        assert_eq!(result.n_treatment, Some(39));
        assert_relative_eq!(
            result.probability_event_control.expect("p control"),
            0.932_373_7,
            epsilon = 1e-5
        );
        assert_relative_eq!(
            result.probability_event_treatment.expect("p treatment"),
            0.744_965_1,
            epsilon = 1e-5
        );
    }

    #[test]
    fn accrual_with_dropout_increases_enrollment() {
        let mut input = base_input();
        input.control_hazard_rate = Some(2.0_f64.ln() / 6.0);
        input.accrual_duration = Some(12.0);
        input.minimum_follow_up = Some(18.0);
        input.dropout_hazard_rate = Some(0.001);

        let result = calculate(input).expect("calculate");
        assert_eq!(result.total_n, Some(80));
    }

    #[test]
    fn rejects_partial_accrual_inputs() {
        let mut input = base_input();
        input.control_hazard_rate = Some(0.1);
        assert!(calculate(input).is_err());
    }
}
