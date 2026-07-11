//! Mann-Whitney U (Wilcoxon rank-sum) sample size and power.

use serde::{Deserialize, Serialize};

use super::nonparametric::{
    mann_whitney_achieved_power, mann_whitney_total_n, probability_superiority_from_cohens_d,
    treatment_fraction,
};
use crate::error::{Error, Result};
use crate::methods::binary::shared::treatment_n_from_control;
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for the Mann-Whitney U test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MannWhitneyInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub control_n: Option<u32>,
    /// Treatment minus control mean difference on the continuous scale.
    pub mean_difference: f64,
    /// Common within-group standard deviation.
    pub standard_deviation: f64,
    /// Treatment-to-control allocation ratio.
    pub allocation_ratio: f64,
    pub alternative: Alternative,
    pub dropout_rate: Option<f64>,
}

/// Results for the Mann-Whitney U test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MannWhitneyResult {
    pub n_control: u32,
    pub n_treatment: u32,
    pub total_n: u32,
    pub n_control_adjusted: u32,
    pub n_treatment_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    pub probability_superiority: f64,
    pub effect_size: f64,
    pub warnings: Vec<CalculationWarning>,
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

fn split_total_n(total_n: u32, allocation_ratio: f64) -> (u32, u32) {
    let t = treatment_fraction(allocation_ratio);
    let n_treatment = (total_n as f64 * t).round().max(1.0) as u32;
    let n_control = total_n.saturating_sub(n_treatment).max(1);
    (n_control, n_treatment)
}

pub fn validate(input: &MannWhitneyInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_positive("standard_deviation", input.standard_deviation)?;
    validation::validate_positive("allocation_ratio", input.allocation_ratio)?;

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
            if control_n < 1 {
                return Err(Error::InvalidInput {
                    field: "controlN".into(),
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
                "detectable effect solve mode is not implemented for Mann-Whitney".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: MannWhitneyInput) -> Result<MannWhitneyResult> {
    validate(&input)?;

    let effect_size = input.mean_difference / input.standard_deviation;
    let probability_superiority = probability_superiority_from_cohens_d(effect_size);

    let warnings = vec![
        CalculationWarning::new(
            "noether_1987",
            "Uses Noether (1987) normal approximation for the Mann-Whitney U / Wilcoxon rank-sum test.",
        ),
        CalculationWarning::new(
            "continuous_no_ties",
            "Assumes a continuous endpoint without ties and maps the location shift to P(treatment > control) under equal-variance normality.",
        ),
    ];

    let (n_control, n_treatment, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let total_n_float = mann_whitney_total_n(
                probability_superiority,
                input.alpha,
                target_power,
                input.allocation_ratio,
                input.alternative,
            );
            if !total_n_float.is_finite() {
                return Err(Error::ConvergenceFailure(
                    "could not determine a finite total sample size for the stated alternative"
                        .into(),
                ));
            }
            let total_n = find_minimum_integer(
                total_n_float.ceil().max(2.0) as u32,
                MAX_SAMPLE_SIZE_SEARCH,
                |total| {
                    mann_whitney_achieved_power(
                        total,
                        probability_superiority,
                        input.alpha,
                        input.alternative,
                        input.allocation_ratio,
                    ) >= target_power
                },
            )
            .ok_or_else(|| {
                Error::ConvergenceFailure(format!(
                    "could not find a total sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
                ))
            })?;
            let (n_control, n_treatment) = split_total_n(total_n, input.allocation_ratio);
            let power = mann_whitney_achieved_power(
                total_n,
                probability_superiority,
                input.alpha,
                input.alternative,
                input.allocation_ratio,
            );
            (n_control, n_treatment, power)
        }
        SolveMode::Power => {
            let n_control = input.control_n.expect("validated");
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let total_n = n_control + n_treatment;
            let power = mann_whitney_achieved_power(
                total_n,
                probability_superiority,
                input.alpha,
                input.alternative,
                input.allocation_ratio,
            );
            (n_control, n_treatment, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    let (n_control_adjusted, n_treatment_adjusted) =
        apply_dropout(n_control, n_treatment, input.dropout_rate);

    Ok(MannWhitneyResult {
        n_control,
        n_treatment,
        total_n: n_control + n_treatment,
        n_control_adjusted,
        n_treatment_adjusted,
        total_n_adjusted: n_control_adjusted + n_treatment_adjusted,
        achieved_power,
        probability_superiority,
        effect_size,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn equal_allocation_reference_shift() {
        let input = MannWhitneyInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            mean_difference: 0.3583,
            standard_deviation: 1.0,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.total_n, 262);
        assert_relative_eq!(result.probability_superiority, 0.6, epsilon = 1e-3);
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }
}
