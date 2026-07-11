//! Paired t-test sample size and power.

use serde::{Deserialize, Serialize};

use super::single_sample_t;
use crate::error::{Error, Result};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for the paired t-test on within-subject differences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedTTestInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    /// Number of pairs when solving for power.
    pub n_pairs: Option<u32>,
    /// Expected mean of paired differences.
    pub mean_difference: f64,
    /// Standard deviation of paired differences.
    pub standard_deviation: f64,
    pub alternative: Alternative,
    pub dropout_rate: Option<f64>,
}

/// Results for the paired t-test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairedTTestResult {
    pub n_pairs: u32,
    pub n_pairs_adjusted: u32,
    pub achieved_power: f64,
    pub effect_size: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &PairedTTestInput) -> Result<()> {
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
            if input.n_pairs.is_some() {
                return Err(Error::InvalidInput {
                    field: "nPairs".into(),
                    message: "must not be set when solving for sample size".into(),
                });
            }
        }
        SolveMode::Power => {
            let n_pairs = input.n_pairs.ok_or_else(|| Error::InvalidInput {
                field: "nPairs".into(),
                message: "is required when solving for power".into(),
            })?;
            if n_pairs < 2 {
                return Err(Error::InvalidInput {
                    field: "nPairs".into(),
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
                "detectable effect solve mode is not implemented for paired t-test".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: PairedTTestInput) -> Result<PairedTTestResult> {
    validate(&input)?;

    let warnings = vec![
        CalculationWarning::new(
            "paired_differences",
            "Analyzes paired differences; assumes approximately normal difference scores.",
        ),
        CalculationWarning::new(
            "exact_t_power",
            "Uses exact t-distribution power with noncentral t; pair count is the smallest integer meeting the target power after rounding.",
        ),
    ];

    let effect_size = input.mean_difference / input.standard_deviation;

    let (n_pairs, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_pairs = single_sample_t::solve_sample_size(
                target_power,
                input.mean_difference,
                input.standard_deviation,
                input.alpha,
                input.alternative,
            )?;
            let power = single_sample_t::achieved_power(
                n_pairs,
                input.mean_difference,
                input.standard_deviation,
                input.alpha,
                input.alternative,
            );
            (n_pairs, power)
        }
        SolveMode::Power => {
            let n_pairs = input.n_pairs.expect("validated");
            let power = single_sample_t::achieved_power(
                n_pairs,
                input.mean_difference,
                input.standard_deviation,
                input.alpha,
                input.alternative,
            );
            (n_pairs, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    let n_pairs_adjusted = single_sample_t::apply_dropout(n_pairs, input.dropout_rate);

    Ok(PairedTTestResult {
        n_pairs,
        n_pairs_adjusted,
        achieved_power,
        effect_size,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn matches_r_power_t_test_paired() {
        let input = PairedTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            n_pairs: None,
            mean_difference: 1.0,
            standard_deviation: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");

        assert_eq!(result.n_pairs, 10);
        assert_relative_eq!(result.achieved_power, 0.8030962, epsilon = 1e-4);
    }
}
