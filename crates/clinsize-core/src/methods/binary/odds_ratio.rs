//! Two-sample odds-ratio superiority sample size and power.

use serde::{Deserialize, Serialize};

use super::shared::{chow_control_n_log_odds_ratio, odds_ratio, treatment_n_from_control};
use crate::distributions::normal;
use crate::error::{Error, Result};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for two-sample odds-ratio superiority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OddsRatioInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub control_n: Option<u32>,
    pub control_rate: f64,
    pub treatment_rate: f64,
    pub allocation_ratio: f64,
    pub alternative: Alternative,
    pub dropout_rate: Option<f64>,
}

/// Results for two-sample odds-ratio superiority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OddsRatioResult {
    pub n_control: u32,
    pub n_treatment: u32,
    pub total_n: u32,
    pub n_control_adjusted: u32,
    pub n_treatment_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    pub odds_ratio: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &OddsRatioInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_probability("control_rate", input.control_rate)?;
    validation::validate_probability("treatment_rate", input.treatment_rate)?;
    validation::validate_positive("allocation_ratio", input.allocation_ratio)?;

    if odds_ratio(input.control_rate, input.treatment_rate) == 1.0 {
        return Err(Error::InvalidInput {
            field: "treatmentRate".into(),
            message: "must yield an odds ratio different from 1".into(),
        });
    }

    if let Some(dropout) = input.dropout_rate {
        validation::validate_dropout_rate(dropout)?;
    }

    match input.solve_mode {
        SolveMode::SampleSize => {
            let power = input.power.ok_or_else(|| Error::InvalidInput {
                field: "power".into(),
                message: "is required when solving for sample size".into(),
            })?;
            validation::validate_power(power)?;
            if input.control_n.is_some() {
                return Err(Error::InvalidInput {
                    field: "controlN".into(),
                    message: "must not be set when solving for sample size".into(),
                });
            }
        }
        SolveMode::Power => {
            let control_n = input.control_n.ok_or_else(|| Error::InvalidInput {
                field: "controlN".into(),
                message: "is required when solving for power".into(),
            })?;
            if control_n < 2 {
                return Err(Error::InvalidInput {
                    field: "controlN".into(),
                    message: "must be at least 2".into(),
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
                "detectable effect solve mode is not implemented for odds ratio".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: OddsRatioInput) -> Result<OddsRatioResult> {
    validate(&input)?;

    let mut warnings = vec![
        CalculationWarning::new(
            "log_odds_ratio_normal",
            "Uses a log odds-ratio normal approximation (Chow et al. 2003; TrialSize `RelativeRisk.Equality`).",
        ),
        CalculationWarning::new(
            "higher_is_better",
            "Assumes a higher event rate is favorable.",
        ),
    ];

    let effect = odds_ratio(input.control_rate, input.treatment_rate);

    let (n_control, n_treatment, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_control = chow_control_n_log_odds_ratio(
                input.control_rate,
                input.treatment_rate,
                input.allocation_ratio,
                input.alpha,
                target_power,
                input.alternative,
            )
            .ceil()
            .max(2.0) as u32;
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let power = achieved_power(n_control, n_treatment, &input);
            (n_control, n_treatment, power)
        }
        SolveMode::Power => {
            let n_control = input.control_n.expect("validated");
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let power = achieved_power(n_control, n_treatment, &input);
            (n_control, n_treatment, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    let (n_control_adjusted, n_treatment_adjusted) =
        apply_dropout(n_control, n_treatment, input.dropout_rate);

    if input.dropout_rate.is_some() {
        warnings.push(CalculationWarning::new(
            "dropout_inflation",
            "Dropout-adjusted sample sizes inflate rounded per-group sizes by 1/(1-dropout).",
        ));
    }

    Ok(OddsRatioResult {
        n_control,
        n_treatment,
        total_n: n_control + n_treatment,
        n_control_adjusted,
        n_treatment_adjusted,
        total_n_adjusted: n_control_adjusted + n_treatment_adjusted,
        achieved_power,
        odds_ratio: effect,
        warnings,
    })
}

fn achieved_power(n_control: u32, n_treatment: u32, input: &OddsRatioInput) -> f64 {
    let log_or = odds_ratio(input.control_rate, input.treatment_rate)
        .ln()
        .abs();
    let se = (1.0 / (n_control as f64 * input.control_rate * (1.0 - input.control_rate))
        + 1.0 / (n_treatment as f64 * input.treatment_rate * (1.0 - input.treatment_rate)))
        .sqrt();
    let tside = match input.alternative {
        Alternative::TwoSided => 2.0,
        Alternative::Greater | Alternative::Less => 1.0,
    };
    let z_alpha = normal::upper_tail_critical(input.alpha / tside);
    normal::cdf(log_or / se - z_alpha)
}

fn apply_dropout(n_control: u32, n_treatment: u32, dropout_rate: Option<f64>) -> (u32, u32) {
    let Some(rate) = dropout_rate else {
        return (n_control, n_treatment);
    };
    let factor = 1.0 / (1.0 - rate);
    (
        (n_control as f64 * factor).ceil() as u32,
        (n_treatment as f64 * factor).ceil() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn matches_trial_size_example() {
        let input = OddsRatioInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            control_rate: 0.25,
            treatment_rate: 0.4,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.n_control, 156);
        assert_relative_eq!(result.odds_ratio, 2.0, epsilon = 1e-12);
    }

    fn base_sample_size_input() -> OddsRatioInput {
        OddsRatioInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            control_rate: 0.25,
            treatment_rate: 0.4,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        }
    }

    fn base_power_input() -> OddsRatioInput {
        OddsRatioInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            control_n: Some(156),
            control_rate: 0.25,
            treatment_rate: 0.4,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        }
    }

    #[test]
    fn rejects_control_n_in_sample_size_mode() {
        let mut input = base_sample_size_input();
        input.control_n = Some(100);
        let err = calculate(input).unwrap_err();
        assert!(err
            .to_string()
            .contains("must not be set when solving for sample size"));
    }

    #[test]
    fn rejects_power_in_power_mode() {
        let mut input = base_power_input();
        input.power = Some(0.8);
        let err = calculate(input).unwrap_err();
        assert!(err
            .to_string()
            .contains("must not be set when solving for power"));
    }
}
