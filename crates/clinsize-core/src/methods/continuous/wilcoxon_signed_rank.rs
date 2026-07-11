//! Wilcoxon signed-rank test sample size and power.

use serde::{Deserialize, Serialize};

use super::nonparametric::{
    probability_positive_difference, wilcoxon_signed_rank_achieved_power, wilcoxon_signed_rank_n,
};
use crate::error::{Error, Result};
use crate::methods::continuous::single_sample_t;
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for the Wilcoxon signed-rank test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WilcoxonSignedRankInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub n_pairs: Option<u32>,
    /// Expected mean of within-subject differences.
    pub mean_difference: f64,
    /// Standard deviation of within-subject differences.
    pub standard_deviation: f64,
    pub alternative: Alternative,
    pub dropout_rate: Option<f64>,
}

/// Results for the Wilcoxon signed-rank test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WilcoxonSignedRankResult {
    pub n_pairs: u32,
    pub n_pairs_adjusted: u32,
    pub achieved_power: f64,
    pub probability_positive_difference: f64,
    pub effect_size: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &WilcoxonSignedRankInput) -> Result<()> {
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
            if n_pairs < 1 {
                return Err(Error::InvalidInput {
                    field: "nPairs".into(),
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
                "detectable effect solve mode is not implemented for Wilcoxon signed-rank".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: WilcoxonSignedRankInput) -> Result<WilcoxonSignedRankResult> {
    validate(&input)?;

    let effect_size = input.mean_difference / input.standard_deviation;
    let probability_positive =
        probability_positive_difference(input.mean_difference, input.standard_deviation);

    let warnings = vec![
        CalculationWarning::new(
            "noether_1987",
            "Uses Noether (1987) normal approximation for the Wilcoxon signed-rank test.",
        ),
        CalculationWarning::new(
            "continuous_no_ties",
            "Assumes continuous paired differences without ties and maps the shift to P(difference > 0) under normality.",
        ),
    ];

    let (n_pairs, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_float = wilcoxon_signed_rank_n(
                probability_positive,
                input.alpha,
                target_power,
                input.alternative,
            );
            if !n_float.is_finite() {
                return Err(Error::ConvergenceFailure(
                    "could not determine a finite number of pairs for the stated alternative"
                        .into(),
                ));
            }
            let n_pairs = find_minimum_integer(n_float.ceil().max(1.0) as u32, MAX_SAMPLE_SIZE_SEARCH, |n| {
                wilcoxon_signed_rank_achieved_power(
                    n,
                    probability_positive,
                    input.alpha,
                    input.alternative,
                ) >= target_power
            })
            .ok_or_else(|| {
                Error::ConvergenceFailure(format!(
                    "could not find a pair count up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
                ))
            })?;
            let power = wilcoxon_signed_rank_achieved_power(
                n_pairs,
                probability_positive,
                input.alpha,
                input.alternative,
            );
            (n_pairs, power)
        }
        SolveMode::Power => {
            let n_pairs = input.n_pairs.expect("validated");
            let power = wilcoxon_signed_rank_achieved_power(
                n_pairs,
                probability_positive,
                input.alpha,
                input.alternative,
            );
            (n_pairs, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    Ok(WilcoxonSignedRankResult {
        n_pairs,
        n_pairs_adjusted: single_sample_t::apply_dropout(n_pairs, input.dropout_rate),
        achieved_power,
        probability_positive_difference: probability_positive,
        effect_size,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn reference_shift_matches_noether() {
        let input = WilcoxonSignedRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            n_pairs: None,
            mean_difference: 0.2533,
            standard_deviation: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.n_pairs, 131);
        assert_relative_eq!(result.probability_positive_difference, 0.6, epsilon = 1e-3);
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }
}
