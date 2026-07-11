//! One-way balanced ANOVA sample size and power.

use serde::{Deserialize, Serialize};

use crate::distributions::{f_distribution, noncentral_f};
use crate::error::{Error, Result};
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for balanced one-way ANOVA (`stats::power.anova.test` parameterization).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneWayAnovaInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    /// Per-group sample size when solving for power.
    pub n_per_group: Option<u32>,
    /// Number of groups (k >= 2).
    pub n_groups: u32,
    /// Between-group variance component.
    pub between_variance: f64,
    /// Within-group variance component.
    pub within_variance: f64,
    pub dropout_rate: Option<f64>,
}

/// Results for balanced one-way ANOVA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneWayAnovaResult {
    pub n_per_group: u32,
    pub total_n: u32,
    pub n_per_group_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    pub effect_size: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &OneWayAnovaInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_positive("between_variance", input.between_variance)?;
    validation::validate_positive("within_variance", input.within_variance)?;

    if input.n_groups < 2 {
        return Err(Error::InvalidInput {
            field: "nGroups".into(),
            message: "must be at least 2".into(),
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
            if input.n_per_group.is_some() {
                return Err(Error::InvalidInput {
                    field: "nPerGroup".into(),
                    message: "must not be set when solving for sample size".into(),
                });
            }
        }
        SolveMode::Power => {
            let n_per_group = input.n_per_group.ok_or_else(|| Error::InvalidInput {
                field: "nPerGroup".into(),
                message: "is required when solving for power".into(),
            })?;
            if n_per_group < 2 {
                return Err(Error::InvalidInput {
                    field: "nPerGroup".into(),
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
                "detectable effect solve mode is not implemented for one-way ANOVA".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: OneWayAnovaInput) -> Result<OneWayAnovaResult> {
    validate(&input)?;

    let warnings = vec![
        CalculationWarning::new(
            "balanced_groups",
            "Assumes equal sample size per group (balanced one-way ANOVA).",
        ),
        CalculationWarning::new(
            "common_variance",
            "Assumes a common within-group variance across all groups.",
        ),
        CalculationWarning::new(
            "exact_f_power",
            "Uses exact F-distribution power with noncentral F; per-group sample size is the smallest integer meeting the target power after rounding.",
        ),
    ];

    let effect_size = (input.between_variance / input.within_variance).sqrt();

    let (n_per_group, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_per_group = solve_n_per_group(&input, target_power)?;
            let power = achieved_power(n_per_group, &input);
            (n_per_group, power)
        }
        SolveMode::Power => {
            let n_per_group = input.n_per_group.expect("validated");
            let power = achieved_power(n_per_group, &input);
            (n_per_group, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    let n_per_group_adjusted = apply_dropout(n_per_group, input.dropout_rate);
    let total_n = n_per_group * input.n_groups;
    let total_n_adjusted = n_per_group_adjusted * input.n_groups;

    Ok(OneWayAnovaResult {
        n_per_group,
        total_n,
        n_per_group_adjusted,
        total_n_adjusted,
        achieved_power,
        effect_size,
        warnings,
    })
}

fn noncentrality_parameter(n_per_group: u32, input: &OneWayAnovaInput) -> f64 {
    (input.n_groups as f64 - 1.0)
        * n_per_group as f64
        * (input.between_variance / input.within_variance)
}

pub fn achieved_power(n_per_group: u32, input: &OneWayAnovaInput) -> f64 {
    let df1 = input.n_groups as f64 - 1.0;
    let df2 = input.n_groups as f64 * (n_per_group as f64 - 1.0);
    let lambda = noncentrality_parameter(n_per_group, input);
    let f_crit = f_distribution::critical_value(input.alpha, df1, df2);
    noncentral_f::upper_tail(f_crit, df1, df2, lambda)
}

fn solve_n_per_group(input: &OneWayAnovaInput, target_power: f64) -> Result<u32> {
    find_minimum_integer(2, MAX_SAMPLE_SIZE_SEARCH, |n_per_group| {
        achieved_power(n_per_group, input) >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a per-group sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
        ))
    })
}

fn apply_dropout(n_per_group: u32, dropout_rate: Option<f64>) -> u32 {
    let Some(rate) = dropout_rate else {
        return n_per_group;
    };
    (n_per_group as f64 / (1.0 - rate)).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn sample_size_input(
        n_groups: u32,
        between_variance: f64,
        within_variance: f64,
        alpha: f64,
        power: f64,
        dropout_rate: Option<f64>,
    ) -> OneWayAnovaInput {
        OneWayAnovaInput {
            solve_mode: SolveMode::SampleSize,
            alpha,
            power: Some(power),
            n_per_group: None,
            n_groups,
            between_variance,
            within_variance,
            dropout_rate,
        }
    }

    #[test]
    fn matches_r_power_anova_test_three_groups() {
        let result = calculate(sample_size_input(3, 1.0, 1.0, 0.05, 0.8, None)).expect("calculate");

        assert_eq!(result.n_per_group, 6);
        assert_eq!(result.total_n, 18);
        assert_relative_eq!(result.achieved_power, 0.8053172, epsilon = 1e-4);
        assert_relative_eq!(result.effect_size, 1.0, epsilon = 1e-12);
    }

    #[test]
    fn matches_r_power_anova_test_four_groups() {
        let result = calculate(sample_size_input(4, 1.0, 1.0, 0.05, 0.8, None)).expect("calculate");

        assert_eq!(result.n_per_group, 5);
        assert_relative_eq!(result.achieved_power, 0.8303491, epsilon = 1e-4);
    }

    #[test]
    fn dropout_inflation_rounds_up() {
        let result =
            calculate(sample_size_input(3, 1.0, 1.0, 0.05, 0.8, Some(0.2))).expect("calculate");

        assert_eq!(result.n_per_group, 6);
        assert_eq!(result.n_per_group_adjusted, 8);
        assert_eq!(result.total_n_adjusted, 24);
    }
}
