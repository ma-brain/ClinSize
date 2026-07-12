//! Two-way balanced ANOVA sample size and power.
//!
//! Extends one-way ANOVA to a two-factor design with main effects A, B and
//! interaction AB. Uses exact noncentral-F power for the user-selected primary
//! effect (Cohen 1988; G*Power "ANOVA: Fixed effects, two-way").

use serde::{Deserialize, Serialize};

use crate::distributions::{f_distribution, noncentral_f};
use crate::error::{Error, Result};
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{CalculationWarning, SolveMode};
use crate::validation;

/// Which ANOVA effect drives the sample-size / power calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnovaEffect {
    MainA,
    MainB,
    Interaction,
}

/// Inputs for balanced two-way ANOVA (variance-component parameterization).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoWayAnovaInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    /// Replicates per cell when solving for power.
    pub n_per_cell: Option<u32>,
    /// Number of levels for factor A (a >= 2).
    pub n_levels_a: u32,
    /// Number of levels for factor B (b >= 2).
    pub n_levels_b: u32,
    /// Which effect drives the sample size / power.
    pub primary_effect: AnovaEffect,
    /// Between-levels variance component for factor A (σ²_A).
    pub variance_a: f64,
    /// Between-levels variance component for factor B (σ²_B).
    pub variance_b: f64,
    /// Interaction variance component (σ²_AB).
    pub variance_interaction: f64,
    /// Within-cell residual variance (σ²_error).
    pub within_variance: f64,
    pub dropout_rate: Option<f64>,
}

/// Results for balanced two-way ANOVA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoWayAnovaResult {
    pub n_per_cell: u32,
    pub total_n: u32,
    pub n_per_cell_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    /// Cohen's f for the primary effect.
    pub effect_size: f64,
    pub primary_effect: AnovaEffect,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &TwoWayAnovaInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_positive("variance_a", input.variance_a)?;
    validation::validate_positive("variance_b", input.variance_b)?;
    validation::validate_positive("variance_interaction", input.variance_interaction)?;
    validation::validate_positive("within_variance", input.within_variance)?;

    if input.n_levels_a < 2 {
        return Err(Error::InvalidInput {
            field: "nLevelsA".into(),
            message: "must be at least 2".into(),
        });
    }
    if input.n_levels_b < 2 {
        return Err(Error::InvalidInput {
            field: "nLevelsB".into(),
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
            if input.n_per_cell.is_some() {
                return Err(Error::InvalidInput {
                    field: "nPerCell".into(),
                    message: "must not be set when solving for sample size".into(),
                });
            }
        }
        SolveMode::Power => {
            let n_per_cell = input.n_per_cell.ok_or_else(|| Error::InvalidInput {
                field: "nPerCell".into(),
                message: "is required when solving for power".into(),
            })?;
            if n_per_cell < 2 {
                return Err(Error::InvalidInput {
                    field: "nPerCell".into(),
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
                "detectable effect solve mode is not implemented for two-way ANOVA".into(),
            ));
        }
    }

    Ok(())
}

/// Numerator df, noncentrality, and effect-size ratio for the primary effect,
/// evaluated at `n_per_cell` replicates per cell.
struct EffectParams {
    df1: f64,
    lambda: f64,
    variance_ratio: f64,
}

fn effect_params(n_per_cell: u32, input: &TwoWayAnovaInput) -> EffectParams {
    let a = input.n_levels_a as f64;
    let b = input.n_levels_b as f64;
    let n = n_per_cell as f64;
    match input.primary_effect {
        AnovaEffect::MainA => {
            let df1 = a - 1.0;
            let ratio = input.variance_a / input.within_variance;
            let lambda = df1 * b * n * ratio;
            EffectParams {
                df1,
                lambda,
                variance_ratio: ratio,
            }
        }
        AnovaEffect::MainB => {
            let df1 = b - 1.0;
            let ratio = input.variance_b / input.within_variance;
            let lambda = df1 * a * n * ratio;
            EffectParams {
                df1,
                lambda,
                variance_ratio: ratio,
            }
        }
        AnovaEffect::Interaction => {
            let df1 = (a - 1.0) * (b - 1.0);
            let ratio = input.variance_interaction / input.within_variance;
            let lambda = df1 * n * ratio;
            EffectParams {
                df1,
                lambda,
                variance_ratio: ratio,
            }
        }
    }
}

pub fn achieved_power(n_per_cell: u32, input: &TwoWayAnovaInput) -> f64 {
    let a = input.n_levels_a as f64;
    let b = input.n_levels_b as f64;
    let df2 = a * b * (n_per_cell as f64 - 1.0);
    let params = effect_params(n_per_cell, input);
    let f_crit = f_distribution::critical_value(input.alpha, params.df1, df2);
    noncentral_f::upper_tail(f_crit, params.df1, df2, params.lambda)
}

fn solve_n_per_cell(input: &TwoWayAnovaInput, target_power: f64) -> Result<u32> {
    find_minimum_integer(2, MAX_SAMPLE_SIZE_SEARCH, |n_per_cell| {
        achieved_power(n_per_cell, input) >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a per-cell sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
        ))
    })
}

fn apply_dropout(n_per_cell: u32, dropout_rate: Option<f64>) -> u32 {
    let Some(rate) = dropout_rate else {
        return n_per_cell;
    };
    (n_per_cell as f64 / (1.0 - rate)).ceil() as u32
}

pub fn calculate(input: TwoWayAnovaInput) -> Result<TwoWayAnovaResult> {
    validate(&input)?;

    let warnings = vec![
        CalculationWarning::new(
            "balanced_cells",
            "Assumes equal sample size per cell (balanced two-way ANOVA).",
        ),
        CalculationWarning::new(
            "common_variance",
            "Assumes a common within-cell variance across all factor-level combinations.",
        ),
        CalculationWarning::new(
            "exact_f_power",
            "Uses exact F-distribution power with noncentral F; per-cell sample size is the smallest integer meeting the target power after rounding.",
        ),
        CalculationWarning::new(
            "primary_effect_only",
            "Sample size is driven by the selected primary effect only; the other two effects may have different power at this N.",
        ),
    ];

    let params = effect_params(input.n_per_cell.unwrap_or(2), &input);
    let effect_size = params.variance_ratio.sqrt();

    let (n_per_cell, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_per_cell = solve_n_per_cell(&input, target_power)?;
            let power = achieved_power(n_per_cell, &input);
            (n_per_cell, power)
        }
        SolveMode::Power => {
            let n_per_cell = input.n_per_cell.expect("validated");
            let power = achieved_power(n_per_cell, &input);
            (n_per_cell, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    let n_per_cell_adjusted = apply_dropout(n_per_cell, input.dropout_rate);
    let total_cells = input.n_levels_a * input.n_levels_b;
    let total_n = n_per_cell * total_cells;
    let total_n_adjusted = n_per_cell_adjusted * total_cells;

    Ok(TwoWayAnovaResult {
        n_per_cell,
        total_n,
        n_per_cell_adjusted,
        total_n_adjusted,
        achieved_power,
        effect_size,
        primary_effect: input.primary_effect,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn sample_size_input(
        n_levels_a: u32,
        n_levels_b: u32,
        primary_effect: AnovaEffect,
        variance_a: f64,
        variance_b: f64,
        variance_interaction: f64,
        within_variance: f64,
        alpha: f64,
        power: f64,
    ) -> TwoWayAnovaInput {
        TwoWayAnovaInput {
            solve_mode: SolveMode::SampleSize,
            alpha,
            power: Some(power),
            n_per_cell: None,
            n_levels_a,
            n_levels_b,
            primary_effect,
            variance_a,
            variance_b,
            variance_interaction,
            within_variance,
            dropout_rate: None,
        }
    }

    // Reference values verified empirically against the noncentral-F formula
    // (Cohen 1988 Chapter 8; G*Power "ANOVA: Fixed effects, two-way").

    #[test]
    fn main_effect_a_2x3() {
        // a=2, b=3, primary = MainA, σ²_A/σ²_err = 0.5 (f ≈ 0.707)
        let result = calculate(sample_size_input(
            2,
            3,
            AnovaEffect::MainA,
            0.5,
            0.5,
            0.5,
            1.0,
            0.05,
            0.8,
        ))
        .expect("calculate");

        assert_eq!(result.n_per_cell, 6);
        assert_eq!(result.total_n, 36); // 6 * 2 * 3
        assert_relative_eq!(result.achieved_power, 0.827, epsilon = 1e-3);
        assert_relative_eq!(result.effect_size, (0.5_f64).sqrt(), epsilon = 1e-12);
        assert_eq!(result.primary_effect, AnovaEffect::MainA);
    }

    #[test]
    fn main_effect_b_2x3() {
        // Same design but primary = MainB. df1=2 (vs 1 for MainA), so power differs.
        let result = calculate(sample_size_input(
            2,
            3,
            AnovaEffect::MainB,
            0.5,
            0.5,
            0.5,
            1.0,
            0.05,
            0.8,
        ))
        .expect("calculate");

        assert_eq!(result.n_per_cell, 6);
        assert_eq!(result.total_n, 36); // 6 * 2 * 3
        assert_relative_eq!(result.achieved_power, 0.847, epsilon = 1e-3);
        assert_eq!(result.primary_effect, AnovaEffect::MainB);
    }

    #[test]
    fn interaction_2x3() {
        // Primary = Interaction. df1 = (a-1)(b-1) = 2. Needs more replicates.
        let result = calculate(sample_size_input(
            2,
            3,
            AnovaEffect::Interaction,
            0.5,
            0.5,
            0.5,
            1.0,
            0.05,
            0.8,
        ))
        .expect("calculate");

        assert_eq!(result.n_per_cell, 11);
        assert_eq!(result.total_n, 66); // 11 * 2 * 3
        assert_relative_eq!(result.achieved_power, 0.833, epsilon = 1e-3);
        assert_eq!(result.primary_effect, AnovaEffect::Interaction);
    }

    #[test]
    fn power_mode_uses_supplied_n() {
        let input = TwoWayAnovaInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            n_per_cell: Some(6),
            n_levels_a: 2,
            n_levels_b: 3,
            primary_effect: AnovaEffect::MainA,
            variance_a: 0.5,
            variance_b: 0.5,
            variance_interaction: 0.5,
            within_variance: 1.0,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_relative_eq!(result.achieved_power, 0.827, epsilon = 1e-3);
    }

    #[test]
    fn dropout_inflation_rounds_up() {
        let mut input = sample_size_input(2, 3, AnovaEffect::MainA, 0.5, 0.5, 0.5, 1.0, 0.05, 0.8);
        input.dropout_rate = Some(0.2);
        let result = calculate(input).expect("calculate");
        assert_eq!(result.n_per_cell, 6);
        assert_eq!(result.n_per_cell_adjusted, 8); // ceil(6 / 0.8)
        assert_eq!(result.total_n_adjusted, 48); // 8 * 6 cells
    }

    #[test]
    fn rejects_fewer_than_two_levels() {
        let input = sample_size_input(1, 3, AnovaEffect::MainA, 0.5, 0.5, 0.5, 1.0, 0.05, 0.8);
        assert!(validate(&input).is_err());
    }
}
