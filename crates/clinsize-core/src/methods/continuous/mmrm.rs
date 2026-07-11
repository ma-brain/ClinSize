//! MMRM (mixed model for repeated measures) sample size and power.
//!
//! Uses the Lu–Skellam (1988) analytical GLS variance efficiency under a simplified
//! single-ρ within-subject correlation parameterization.

use serde::{Deserialize, Serialize};

use crate::distributions::normal;
use crate::error::{Error, Result};
use crate::methods::binary::shared::treatment_n_from_control;
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, CorrelationStructure, SolveMode};
use crate::validation;

/// Inputs for two-group parallel MMRM at the final post-baseline visit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MmrmInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub control_n: Option<u32>,
    /// Treatment minus control effect at the primary/final visit (δ).
    pub treatment_effect: f64,
    /// Residual standard deviation (σ).
    pub residual_standard_deviation: f64,
    pub correlation_structure: CorrelationStructure,
    /// Within-subject correlation parameter (ρ).
    pub correlation: f64,
    /// Number of post-baseline visits (k).
    pub n_post_baseline_visits: u32,
    /// Optional per-visit dropout rate (d_visit) in [0, 1).
    pub per_visit_dropout_rate: Option<f64>,
    pub allocation_ratio: f64,
    pub alternative: Alternative,
}

/// Results for two-group parallel MMRM sizing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MmrmResult {
    pub n_control: u32,
    pub n_treatment: u32,
    pub total_n: u32,
    pub n_control_adjusted: u32,
    pub n_treatment_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    pub rho_final: f64,
    pub gls_factor: f64,
    pub gls_variance_efficiency_factor: f64,
    pub v_eff: f64,
    pub cumulative_dropout: f64,
    pub warnings: Vec<CalculationWarning>,
}

/// ρ_final used in the Lu–Skellam GLS efficiency formulas.
pub fn rho_final(
    structure: CorrelationStructure,
    correlation: f64,
    n_post_baseline_visits: u32,
) -> f64 {
    match structure {
        CorrelationStructure::Ar1 | CorrelationStructure::Toeplitz => {
            correlation.powi(n_post_baseline_visits as i32)
        }
        CorrelationStructure::Unstructured
        | CorrelationStructure::CompoundSymmetry
        | CorrelationStructure::Csh => correlation,
    }
}

/// GLS variance efficiency factor = 1 + (k − 1) × ρ_final.
pub fn gls_variance_efficiency_factor(
    structure: CorrelationStructure,
    correlation: f64,
    n_post_baseline_visits: u32,
) -> f64 {
    let rho = rho_final(structure, correlation, n_post_baseline_visits);
    1.0 + (n_post_baseline_visits as f64 - 1.0) * rho
}

/// GLS factor = 1 / (1 + (k − 1) × ρ_final).
pub fn gls_factor(
    structure: CorrelationStructure,
    correlation: f64,
    n_post_baseline_visits: u32,
) -> f64 {
    1.0 / gls_variance_efficiency_factor(structure, correlation, n_post_baseline_visits)
}

/// Effective variance V_eff = 2 × σ² × (1 − ρ_final) × GLS_factor.
pub fn v_eff(
    residual_standard_deviation: f64,
    structure: CorrelationStructure,
    correlation: f64,
    n_post_baseline_visits: u32,
) -> f64 {
    let rho = rho_final(structure, correlation, n_post_baseline_visits);
    let gls = gls_factor(structure, correlation, n_post_baseline_visits);
    2.0 * residual_standard_deviation.powi(2) * (1.0 - rho) * gls
}

/// Cumulative dropout d_cum = 1 − (1 − d_visit)^k.
pub fn cumulative_dropout(per_visit_dropout_rate: f64, n_post_baseline_visits: u32) -> f64 {
    1.0 - (1.0 - per_visit_dropout_rate).powi(n_post_baseline_visits as i32)
}

fn tail_multiplier(alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => 2.0,
        Alternative::Greater | Alternative::Less => 1.0,
    }
}

/// Continuous per-arm sample size from the normal approximation.
pub fn n_arm_continuous(
    treatment_effect: f64,
    v_eff: f64,
    alpha: f64,
    power: f64,
    alternative: Alternative,
) -> f64 {
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let z_beta = normal::quantile(power);
    (z_alpha + z_beta).powi(2) * v_eff / treatment_effect.powi(2)
}

pub fn achieved_power(
    n_control: u32,
    n_treatment: u32,
    treatment_effect: f64,
    v_eff: f64,
    alpha: f64,
    alternative: Alternative,
) -> f64 {
    let variance_diff =
        v_eff / 2.0 * (1.0 / n_control as f64 + 1.0 / n_treatment as f64);
    let z_stat = treatment_effect / variance_diff.sqrt();
    match alternative {
        Alternative::TwoSided => {
            let z_crit = normal::upper_tail_critical(alpha / 2.0);
            normal::cdf(z_stat - z_crit) + normal::cdf(-z_stat - z_crit)
        }
        Alternative::Greater => {
            let z_crit = normal::upper_tail_critical(alpha);
            normal::cdf(z_stat - z_crit)
        }
        Alternative::Less => {
            let z_crit = normal::upper_tail_critical(alpha);
            normal::cdf(-z_stat - z_crit)
        }
    }
}

fn apply_visit_dropout(n: u32, cumulative: f64) -> u32 {
    if cumulative <= 0.0 {
        return n;
    }
    (n as f64 / (1.0 - cumulative)).ceil() as u32
}

pub fn validate(input: &MmrmInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_positive(
        "residualStandardDeviation",
        input.residual_standard_deviation,
    )?;
    validation::validate_positive("allocationRatio", input.allocation_ratio)?;
    validation::validate_correlation(input.correlation)?;

    if input.treatment_effect == 0.0 {
        return Err(Error::InvalidInput {
            field: "treatmentEffect".into(),
            message: "must be non-zero".into(),
        });
    }

    if input.n_post_baseline_visits < 1 {
        return Err(Error::InvalidInput {
            field: "nPostBaselineVisits".into(),
            message: "must be at least 1".into(),
        });
    }

    if let Some(rate) = input.per_visit_dropout_rate {
        validation::validate_dropout_rate(rate)?;
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
                "detectable effect solve mode is not implemented for MMRM".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: MmrmInput) -> Result<MmrmResult> {
    validate(&input)?;

    let rho = rho_final(
        input.correlation_structure,
        input.correlation,
        input.n_post_baseline_visits,
    );
    let gls = gls_factor(
        input.correlation_structure,
        input.correlation,
        input.n_post_baseline_visits,
    );
    let efficiency = gls_variance_efficiency_factor(
        input.correlation_structure,
        input.correlation,
        input.n_post_baseline_visits,
    );
    let variance_eff = v_eff(
        input.residual_standard_deviation,
        input.correlation_structure,
        input.correlation,
        input.n_post_baseline_visits,
    );
    let dropout_cum = input
        .per_visit_dropout_rate
        .map(|rate| cumulative_dropout(rate, input.n_post_baseline_visits))
        .unwrap_or(0.0);

    let mut warnings = vec![
        CalculationWarning::new(
            "mmrm_gls_approximation",
            "Uses the Lu–Skellam analytical GLS variance efficiency under a simplified single-ρ within-subject correlation; not a full unstructured correlation matrix.",
        ),
        CalculationWarning::new(
            "mmrm_normal_approximation",
            "Sample size uses a normal approximation (z-scores); achieved power is recomputed at the rounded integer allocation.",
        ),
        CalculationWarning::new(
            "mmrm_final_visit_effect",
            "Assumes the treatment effect δ applies at the final post-baseline visit under equal residual variance across arms.",
        ),
    ];

    if input.per_visit_dropout_rate.is_some() {
        warnings.push(CalculationWarning::new(
            "visit_dropout_inflation",
            "Enrollable sample sizes inflate evaluable per-arm counts by 1/(1 − d_cum), where d_cum = 1 − (1 − d_visit)^k.",
        ));
    }

    let (n_control, n_treatment, achieved) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_control = solve_control_n(&input, variance_eff, target_power)?;
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let power = achieved_power(
                n_control,
                n_treatment,
                input.treatment_effect,
                variance_eff,
                input.alpha,
                input.alternative,
            );
            (n_control, n_treatment, power)
        }
        SolveMode::Power => {
            let n_control = input.control_n.expect("validated");
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let power = achieved_power(
                n_control,
                n_treatment,
                input.treatment_effect,
                variance_eff,
                input.alpha,
                input.alternative,
            );
            (n_control, n_treatment, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    let n_control_adjusted = apply_visit_dropout(n_control, dropout_cum);
    let n_treatment_adjusted = apply_visit_dropout(n_treatment, dropout_cum);

    Ok(MmrmResult {
        n_control,
        n_treatment,
        total_n: n_control + n_treatment,
        n_control_adjusted,
        n_treatment_adjusted,
        total_n_adjusted: n_control_adjusted + n_treatment_adjusted,
        achieved_power: achieved,
        rho_final: rho,
        gls_factor: gls,
        gls_variance_efficiency_factor: efficiency,
        v_eff: variance_eff,
        cumulative_dropout: dropout_cum,
        warnings,
    })
}

fn solve_control_n(input: &MmrmInput, variance_eff: f64, target_power: f64) -> Result<u32> {
    find_minimum_integer(1, MAX_SAMPLE_SIZE_SEARCH, |n_control| {
        let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
        let power = achieved_power(
            n_control,
            n_treatment,
            input.treatment_effect,
            variance_eff,
            input.alpha,
            input.alternative,
        );
        power >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a control-group sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn reference_input() -> MmrmInput {
        MmrmInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            treatment_effect: 2.0,
            residual_standard_deviation: 2.0,
            correlation_structure: CorrelationStructure::Unstructured,
            correlation: 0.5,
            n_post_baseline_visits: 3,
            per_visit_dropout_rate: Some(0.05),
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
        }
    }

    #[test]
    fn rho_final_unstructured_equals_rho() {
        assert_relative_eq!(
            rho_final(CorrelationStructure::Unstructured, 0.5, 3),
            0.5,
            epsilon = 1e-12
        );
    }

    #[test]
    fn rho_final_ar1_uses_power_k() {
        assert_relative_eq!(
            rho_final(CorrelationStructure::Ar1, 0.5, 3),
            0.125,
            epsilon = 1e-12
        );
    }

    #[test]
    fn reference_example_lu_skellam() {
        let result = calculate(reference_input()).expect("calculate");

        assert_relative_eq!(result.gls_variance_efficiency_factor, 2.0, epsilon = 1e-3);
        assert_relative_eq!(result.cumulative_dropout, 0.142625, epsilon = 1e-3);
        assert_eq!(result.n_control, 4);
        assert_eq!(result.n_treatment, 4);
        assert_eq!(result.total_n, 8);
        assert_eq!(result.n_control_adjusted, 5);
        assert_eq!(result.n_treatment_adjusted, 5);
        assert_eq!(result.total_n_adjusted, 10);
        assert_relative_eq!(result.rho_final, 0.5, epsilon = 1e-12);
        assert_relative_eq!(result.gls_factor, 0.5, epsilon = 1e-12);
        assert_relative_eq!(result.v_eff, 2.0, epsilon = 1e-3);
        assert!(result.achieved_power >= 0.8);
    }

    #[test]
    fn rejects_zero_treatment_effect() {
        let mut input = reference_input();
        input.treatment_effect = 0.0;
        assert!(validate(&input).is_err());
    }

    #[test]
    fn rejects_zero_post_baseline_visits() {
        let mut input = reference_input();
        input.n_post_baseline_visits = 0;
        assert!(validate(&input).is_err());
    }

    #[test]
    fn power_mode_uses_supplied_control_n() {
        let input = MmrmInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            control_n: Some(4),
            treatment_effect: 2.0,
            residual_standard_deviation: 2.0,
            correlation_structure: CorrelationStructure::Unstructured,
            correlation: 0.5,
            n_post_baseline_visits: 3,
            per_visit_dropout_rate: None,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
        };
        let result = calculate(input).expect("calculate");
        assert!(result.achieved_power >= 0.8);
        assert_eq!(result.n_control_adjusted, 4);
    }

    #[test]
    fn more_visits_increase_efficiency_factor_for_positive_rho() {
        let low_k = gls_variance_efficiency_factor(CorrelationStructure::Unstructured, 0.5, 2);
        let high_k = gls_variance_efficiency_factor(CorrelationStructure::Unstructured, 0.5, 4);
        assert!(high_k > low_k);
    }
}
