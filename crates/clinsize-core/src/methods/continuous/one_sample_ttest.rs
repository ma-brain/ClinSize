//! One-sample t-test sample size and power.

use serde::{Deserialize, Serialize};

use super::single_sample_t;
use crate::error::{Error, Result};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for the one-sample t-test against a reference mean.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneSampleTTestInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub n: Option<u32>,
    /// Expected mean minus reference mean.
    pub mean_difference: f64,
    pub standard_deviation: f64,
    pub alternative: Alternative,
    pub dropout_rate: Option<f64>,
}

/// Results for the one-sample t-test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneSampleTTestResult {
    pub n: u32,
    pub n_adjusted: u32,
    pub achieved_power: f64,
    pub effect_size: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &OneSampleTTestInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_positive("standard_deviation", input.standard_deviation)?;

    if input.mean_difference == 0.0 {
        return Err(Error::InvalidInput {
            field: "meanDifference".into(),
            message: "must be non-zero".into(),
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
            if input.n.is_some() {
                return Err(Error::InvalidInput {
                    field: "n".into(),
                    message: "must not be set when solving for sample size".into(),
                });
            }
        }
        SolveMode::Power => {
            let n = input.n.ok_or_else(|| Error::InvalidInput {
                field: "n".into(),
                message: "is required when solving for power".into(),
            })?;
            if n < 2 {
                return Err(Error::InvalidInput {
                    field: "n".into(),
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
                "detectable effect solve mode is not implemented for one-sample t-test".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: OneSampleTTestInput) -> Result<OneSampleTTestResult> {
    validate(&input)?;

    let warnings = vec![
        CalculationWarning::new(
            "one_sample",
            "Compares one group mean to a known reference value.",
        ),
        CalculationWarning::new(
            "exact_t_power",
            "Uses exact t-distribution power with noncentral t; sample size is the smallest integer meeting the target power after rounding.",
        ),
    ];

    let effect_size = input.mean_difference / input.standard_deviation;

    let (n, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n = single_sample_t::solve_sample_size(
                target_power,
                input.mean_difference,
                input.standard_deviation,
                input.alpha,
                input.alternative,
            )?;
            let power = single_sample_t::achieved_power(
                n,
                input.mean_difference,
                input.standard_deviation,
                input.alpha,
                input.alternative,
            );
            (n, power)
        }
        SolveMode::Power => {
            let n = input.n.expect("validated");
            let power = single_sample_t::achieved_power(
                n,
                input.mean_difference,
                input.standard_deviation,
                input.alpha,
                input.alternative,
            );
            (n, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    let n_adjusted = single_sample_t::apply_dropout(n, input.dropout_rate);

    Ok(OneSampleTTestResult {
        n,
        n_adjusted,
        achieved_power,
        effect_size,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn sample_size_input(
        mean_difference: f64,
        standard_deviation: f64,
        alpha: f64,
        power: f64,
        alternative: Alternative,
        dropout_rate: Option<f64>,
    ) -> OneSampleTTestInput {
        OneSampleTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha,
            power: Some(power),
            n: None,
            mean_difference,
            standard_deviation,
            alternative,
            dropout_rate,
        }
    }

    #[test]
    fn matches_r_power_t_test_two_sided() {
        let result = calculate(sample_size_input(
            1.0,
            1.0,
            0.05,
            0.8,
            Alternative::TwoSided,
            None,
        ))
        .expect("calculate");

        assert_eq!(result.n, 10);
        assert_relative_eq!(result.achieved_power, 0.8030962, epsilon = 1e-4);
    }

    #[test]
    fn matches_r_power_t_test_one_sided() {
        let result = calculate(sample_size_input(
            1.0,
            1.0,
            0.05,
            0.8,
            Alternative::Greater,
            None,
        ))
        .expect("calculate");

        assert_eq!(result.n, 8);
        assert_relative_eq!(result.achieved_power, 0.8150194, epsilon = 1e-4);
    }

    #[test]
    fn matches_r_smaller_effect() {
        let result = calculate(sample_size_input(
            0.5,
            1.0,
            0.05,
            0.8,
            Alternative::TwoSided,
            None,
        ))
        .expect("calculate");

        assert_eq!(result.n, 34);
    }
}
