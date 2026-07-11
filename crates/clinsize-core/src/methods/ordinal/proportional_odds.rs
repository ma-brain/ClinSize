//! Ordinal proportional odds sample size and power (Whitehead 1993 / Hmisc posamsize).

use serde::{Deserialize, Serialize};

use crate::distributions::normal;
use crate::error::{Error, Result};
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for two-group ordinal comparison under proportional odds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProportionalOddsInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub control_n: Option<u32>,
    /// Control-group category probabilities (ordered best → worst); must sum to 1.
    pub category_probabilities: Vec<f64>,
    /// Target odds ratio (> 1).
    pub odds_ratio: f64,
    /// Fraction of subjects assigned to treatment (0, 1).
    pub treatment_fraction: f64,
    pub dropout_rate: Option<f64>,
}

/// Results for two-group ordinal comparison under proportional odds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProportionalOddsResult {
    pub n_control: u32,
    pub n_treatment: u32,
    pub total_n: u32,
    pub n_control_adjusted: u32,
    pub n_treatment_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    /// Whitehead efficiency factor ps = 1 − Σpᵢ³.
    pub efficiency: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn efficiency_factor(category_probabilities: &[f64]) -> f64 {
    1.0 - category_probabilities.iter().map(|p| p.powi(3)).sum::<f64>()
}

pub fn validate(input: &ProportionalOddsInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;

    if input.category_probabilities.is_empty() {
        return Err(Error::InvalidInput {
            field: "categoryProbabilities".into(),
            message: "must contain at least one category".into(),
        });
    }

    for (index, &p) in input.category_probabilities.iter().enumerate() {
        if p <= 0.0 || p >= 1.0 || !p.is_finite() {
            return Err(Error::InvalidInput {
                field: "categoryProbabilities".into(),
                message: format!("entry {} must be greater than 0 and less than 1", index + 1),
            });
        }
    }

    let sum: f64 = input.category_probabilities.iter().sum();
    if (sum - 1.0).abs() > 1e-6 {
        return Err(Error::InvalidInput {
            field: "categoryProbabilities".into(),
            message: format!("must sum to 1 (got {sum:.6})"),
        });
    }

    if input.odds_ratio <= 1.0 {
        return Err(Error::InvalidInput {
            field: "oddsRatio".into(),
            message: "must be greater than 1".into(),
        });
    }

    if input.treatment_fraction <= 0.0 || input.treatment_fraction >= 1.0 {
        return Err(Error::InvalidInput {
            field: "treatmentFraction".into(),
            message: "must be greater than 0 and less than 1".into(),
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
                "detectable effect solve mode is not implemented for proportional odds".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: ProportionalOddsInput) -> Result<ProportionalOddsResult> {
    validate(&input)?;

    let mut warnings = vec![
        CalculationWarning::new(
            "proportional_odds",
            "Uses Whitehead (1993) / Hmisc posamsize formula assuming proportional odds and control-group category probabilities.",
        ),
        CalculationWarning::new(
            "category_ordering",
            "Category probabilities are ordered from best to worst; the odds ratio applies uniformly across cumulative logits.",
        ),
    ];

    let ps = efficiency_factor(&input.category_probabilities);
    let log_or = input.odds_ratio.ln();

    let (n_control, n_treatment, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let total_n = solve_total_n(&input, target_power, ps, log_or)?;
            let (n_control, n_treatment) = split_total_n(total_n, input.treatment_fraction);
            let power = achieved_power(n_control, n_treatment, log_or, ps, input.alpha);
            (n_control, n_treatment, power)
        }
        SolveMode::Power => {
            let n_control = input.control_n.expect("validated");
            let total_n = infer_total_n_from_control(n_control, input.treatment_fraction);
            let n_treatment = total_n - n_control;
            let power = achieved_power(n_control, n_treatment, log_or, ps, input.alpha);
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

    Ok(ProportionalOddsResult {
        n_control,
        n_treatment,
        total_n: n_control + n_treatment,
        n_control_adjusted,
        n_treatment_adjusted,
        total_n_adjusted: n_control_adjusted + n_treatment_adjusted,
        achieved_power,
        efficiency: ps,
        warnings,
    })
}

fn allocation_factor(treatment_fraction: f64) -> f64 {
    let f = treatment_fraction;
    (1.0 - f) / f
}

pub fn continuous_total_n(
    odds_ratio: f64,
    treatment_fraction: f64,
    alpha: f64,
    power: f64,
    ps: f64,
) -> f64 {
    let a = allocation_factor(treatment_fraction);
    let z_alpha = normal::upper_tail_critical(alpha / 2.0);
    let z_beta = normal::quantile(power);
    let log_or = odds_ratio.ln();
    3.0 * (a + 1.0).powi(2) * (z_alpha + z_beta).powi(2) / a / log_or.powi(2) / ps
}

pub fn power_variance(n_control: u32, n_treatment: u32, ps: f64) -> f64 {
    let n1 = n_treatment as f64;
    let n2 = n_control as f64;
    let n = n1 + n2;
    n1 * n2 * n / (3.0 * (n + 1.0).powi(2)) * ps
}

pub fn achieved_power(
    n_control: u32,
    n_treatment: u32,
    log_odds_ratio: f64,
    ps: f64,
    alpha: f64,
) -> f64 {
    let v = power_variance(n_control, n_treatment, ps);
    let z_alpha = normal::upper_tail_critical(alpha / 2.0);
    normal::cdf(log_odds_ratio.abs() * v.sqrt() - z_alpha)
}

fn split_total_n(total_n: u32, treatment_fraction: f64) -> (u32, u32) {
    let n_treatment = (total_n as f64 * treatment_fraction).ceil().max(1.0) as u32;
    let n_control = total_n.saturating_sub(n_treatment).max(1);
    (n_control, n_treatment)
}

fn infer_total_n_from_control(n_control: u32, treatment_fraction: f64) -> u32 {
    let f = treatment_fraction;
    ((n_control as f64) / (1.0 - f)).ceil() as u32
}

fn solve_total_n(
    input: &ProportionalOddsInput,
    target_power: f64,
    ps: f64,
    log_or: f64,
) -> Result<u32> {
    let continuous =
        continuous_total_n(input.odds_ratio, input.treatment_fraction, input.alpha, target_power, ps);
    let start = continuous.floor().max(2.0) as u32;

    find_minimum_integer(start.saturating_sub(1).max(2), MAX_SAMPLE_SIZE_SEARCH, |total_n| {
        let (n_control, n_treatment) = split_total_n(total_n, input.treatment_fraction);
        achieved_power(n_control, n_treatment, log_or, ps, input.alpha) >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a total sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
        ))
    })
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
    fn rejects_probabilities_not_summing_to_one() {
        let input = ProportionalOddsInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            category_probabilities: vec![0.5, 0.5, 0.1],
            odds_ratio: 2.0,
            treatment_fraction: 0.5,
            dropout_rate: None,
        };
        assert!(validate(&input).is_err());
    }

    #[test]
    fn matches_hmisc_posamsize_reference() {
        // p=[0.2,0.5,0.2,0.1], OR=2, f=0.5, α=0.05, power=0.8 → total ≈ 228.5 (ceil 229)
        let input = ProportionalOddsInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            category_probabilities: vec![0.2, 0.5, 0.2, 0.1],
            odds_ratio: 2.0,
            treatment_fraction: 0.5,
            dropout_rate: None,
        };
        let ps = efficiency_factor(&input.category_probabilities);
        assert_relative_eq!(ps, 0.858, epsilon = 1e-3);

        let continuous = continuous_total_n(2.0, 0.5, 0.05, 0.8, ps);
        assert_relative_eq!(continuous, 228.5, epsilon = 0.5);

        let result = calculate(input).expect("calculate");
        assert_eq!(result.total_n, 231);
        assert_relative_eq!(result.efficiency, 0.858, epsilon = 1e-3);
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }

    #[test]
    fn power_mode_uses_supplied_control_n() {
        let input = ProportionalOddsInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            control_n: Some(115),
            category_probabilities: vec![0.2, 0.5, 0.2, 0.1],
            odds_ratio: 2.0,
            treatment_fraction: 0.5,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }
}
