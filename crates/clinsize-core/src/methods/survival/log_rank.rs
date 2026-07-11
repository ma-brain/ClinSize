//! Two-arm log-rank test event count (Schoenfeld approximation).

use serde::{Deserialize, Serialize};

use super::shared::{schoenfeld_events, schoenfeld_power, split_events};
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

    Ok(())
}

pub fn calculate(input: LogRankInput) -> Result<LogRankResult> {
    validate(&input)?;

    let warnings = vec![
        CalculationWarning::new(
            "proportional_hazards",
            "Assumes proportional hazards and independent censoring.",
        ),
        CalculationWarning::new(
            "schoenfeld_approximation",
            "Uses the Schoenfeld (1981) normal approximation for log-rank event counts; required events are rounded up to the next integer.",
        ),
        CalculationWarning::new(
            "events_not_subjects",
            "Reports required events, not enrolled subjects. Accrual and follow-up assumptions are needed to translate events into sample size.",
        ),
    ];

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

    Ok(LogRankResult {
        required_events,
        events_control,
        events_treatment,
        achieved_power,
        hazard_ratio: input.hazard_ratio,
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

    #[test]
    fn sample_size_two_sided_matches_gsdesign() {
        let input = LogRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            total_events: None,
            hazard_ratio: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.required_events, 66);
        assert_eq!(result.events_control, 33);
        assert_eq!(result.events_treatment, 33);
        assert_relative_eq!(result.achieved_power, 0.803_894_1, epsilon = 1e-5);
    }

    #[test]
    fn sample_size_one_sided_matches_gsdesign() {
        let input = LogRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            total_events: None,
            hazard_ratio: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::Greater,
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.required_events, 52);
        assert_relative_eq!(result.achieved_power, 0.803_537, epsilon = 1e-5);
    }

    #[test]
    fn sample_size_unequal_allocation_matches_gsdesign() {
        let input = LogRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            total_events: None,
            hazard_ratio: 0.5,
            allocation_ratio: 2.0,
            alternative: Alternative::TwoSided,
        };
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
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.required_events, 66);
        assert_relative_eq!(result.achieved_power, 0.803_894_1, epsilon = 1e-5);
    }

    #[test]
    fn rejects_hazard_ratio_of_one() {
        let input = LogRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            total_events: None,
            hazard_ratio: 1.0,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
        };
        assert!(calculate(input).is_err());
    }

    #[test]
    fn smaller_hazard_ratio_needs_fewer_events_than_mild_effect() {
        let strong = calculate(LogRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            total_events: None,
            hazard_ratio: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
        })
        .expect("calculate");
        let mild = calculate(LogRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            total_events: None,
            hazard_ratio: 0.75,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
        })
        .expect("calculate");
        assert!(strong.required_events < mild.required_events);
        assert_eq!(mild.required_events, 380);
    }
}
