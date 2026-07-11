//! Two-sample ANCOVA sample size and power with approximate variance reduction.
//!
//! Uses σ_adj = σ_y × √(1 − ρ²) and delegates to the equal-variance two-sample
//! t-test. This is an approximate single-covariate adjustment, not a full
//! model-based ANCOVA calculation.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::methods::continuous::two_sample_ttest::{self, TwoSampleTTestInput};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for two-group parallel ANCOVA with one baseline covariate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AncovaTwoSampleInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub control_n: Option<u32>,
    pub mean_difference: f64,
    /// Unadjusted outcome standard deviation.
    pub standard_deviation: f64,
    /// Baseline-outcome correlation ρ in (−1, 1).
    pub baseline_outcome_correlation: f64,
    pub allocation_ratio: f64,
    pub alternative: Alternative,
    pub dropout_rate: Option<f64>,
}

/// Results for two-group parallel ANCOVA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AncovaTwoSampleResult {
    pub n_control: u32,
    pub n_treatment: u32,
    pub total_n: u32,
    pub n_control_adjusted: u32,
    pub n_treatment_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    /// Cohen's d using the unadjusted outcome standard deviation.
    pub effect_size: f64,
    pub unadjusted_standard_deviation: f64,
    pub adjusted_standard_deviation: f64,
    pub baseline_outcome_correlation: f64,
    pub variance_reduction_factor: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn variance_reduction_factor(correlation: f64) -> f64 {
    1.0 - correlation * correlation
}

pub fn adjusted_standard_deviation(unadjusted_sd: f64, correlation: f64) -> f64 {
    unadjusted_sd * variance_reduction_factor(correlation).sqrt()
}

pub fn validate(input: &AncovaTwoSampleInput) -> Result<()> {
    validation::validate_correlation(input.baseline_outcome_correlation)?;

    let ttest_input = to_two_sample_input(input)?;
    two_sample_ttest::validate(&ttest_input)
}

pub fn calculate(input: AncovaTwoSampleInput) -> Result<AncovaTwoSampleResult> {
    validate(&input)?;

    let unadjusted_sd = input.standard_deviation;
    let correlation = input.baseline_outcome_correlation;
    let reduction = variance_reduction_factor(correlation);
    let adjusted_sd = adjusted_standard_deviation(unadjusted_sd, correlation);

    let ttest_input = to_two_sample_input(&input)?;
    let mut ttest_result = two_sample_ttest::calculate(ttest_input)?;

    let effect_size = input.mean_difference / unadjusted_sd;

    ttest_result.warnings.insert(
        0,
        CalculationWarning::new(
            "ancova_variance_reduction",
            "Uses approximate variance reduction σ_adj = σ_y × √(1 − ρ²) for a single baseline covariate; assumes parallel slopes and does not model covariate imbalance.",
        ),
    );

    Ok(AncovaTwoSampleResult {
        n_control: ttest_result.n_control,
        n_treatment: ttest_result.n_treatment,
        total_n: ttest_result.total_n,
        n_control_adjusted: ttest_result.n_control_adjusted,
        n_treatment_adjusted: ttest_result.n_treatment_adjusted,
        total_n_adjusted: ttest_result.total_n_adjusted,
        achieved_power: ttest_result.achieved_power,
        effect_size,
        unadjusted_standard_deviation: unadjusted_sd,
        adjusted_standard_deviation: adjusted_sd,
        baseline_outcome_correlation: correlation,
        variance_reduction_factor: reduction,
        warnings: ttest_result.warnings,
    })
}

fn to_two_sample_input(input: &AncovaTwoSampleInput) -> Result<TwoSampleTTestInput> {
    Ok(TwoSampleTTestInput {
        solve_mode: input.solve_mode,
        alpha: input.alpha,
        power: input.power,
        control_n: input.control_n,
        mean_difference: input.mean_difference,
        standard_deviation: adjusted_standard_deviation(
            input.standard_deviation,
            input.baseline_outcome_correlation,
        ),
        allocation_ratio: input.allocation_ratio,
        alternative: input.alternative,
        dropout_rate: input.dropout_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn sample_size_input(
        mean_difference: f64,
        standard_deviation: f64,
        correlation: f64,
        alpha: f64,
        power: f64,
        allocation_ratio: f64,
        dropout_rate: Option<f64>,
    ) -> AncovaTwoSampleInput {
        AncovaTwoSampleInput {
            solve_mode: SolveMode::SampleSize,
            alpha,
            power: Some(power),
            control_n: None,
            mean_difference,
            standard_deviation,
            baseline_outcome_correlation: correlation,
            allocation_ratio,
            alternative: Alternative::TwoSided,
            dropout_rate,
        }
    }

    #[test]
    fn rejects_perfect_correlation() {
        let input = sample_size_input(3.0, 10.0, 1.0, 0.05, 0.8, 1.0, None);
        assert!(validate(&input).is_err());
    }

    #[test]
    fn matches_r_power_t_test_with_adjusted_sd() {
        // R: sd_adj <- 10 * sqrt(1 - 0.5^2); power.t.test(delta=3, sd=sd_adj, ...)
        let result =
            calculate(sample_size_input(3.0, 10.0, 0.5, 0.05, 0.8, 1.0, None)).expect("calculate");

        assert_eq!(result.n_control, 132);
        assert_eq!(result.n_treatment, 132);
        assert_relative_eq!(result.adjusted_standard_deviation, 8.660254, epsilon = 1e-5);
        assert_relative_eq!(result.variance_reduction_factor, 0.75, epsilon = 1e-12);
        assert_relative_eq!(result.achieved_power, 0.80065, epsilon = 1e-4);
        assert_relative_eq!(result.effect_size, 0.3, epsilon = 1e-12);
    }

    #[test]
    fn matches_r_smaller_effect_with_moderate_correlation() {
        // R: sd_adj <- sqrt(1 - 0.3^2); power.t.test(delta=0.5, sd=sd_adj, ...)
        let result =
            calculate(sample_size_input(0.5, 1.0, 0.3, 0.05, 0.8, 1.0, None)).expect("calculate");

        assert_eq!(result.n_control, 59);
        assert_eq!(result.n_treatment, 59);
        assert_relative_eq!(result.achieved_power, 0.8059902, epsilon = 1e-4);
    }

    #[test]
    fn power_mode_uses_supplied_control_n() {
        let input = AncovaTwoSampleInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            control_n: Some(132),
            mean_difference: 3.0,
            standard_deviation: 10.0,
            baseline_outcome_correlation: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_relative_eq!(result.achieved_power, 0.80065, epsilon = 1e-4);
    }
}
