//! Blinded sample size re-estimation for a continuous two-sample t-test.
//!
//! ## Formula / algorithm
//!
//! 1. Plan per-arm sample size `n₀` with the equal-variance two-sample t-test
//!    using the assumed common SD `σ₀` (Friede & Kieser, 2006).
//! 2. At a blinded interim look after fraction `τ` of the planned enrollment,
//!    pool all observed subjects and estimate the common SD `s_b` without
//!    unblinding treatment assignment.
//! 3. Re-estimate per-arm sample size:
//!
//!    `n_re = ceil(n₀ × (s_b / σ₀)²)`
//!
//! 4. Apply a pre-specified cap:
//!
//!    `n_cap = min(n_re, ceil(n₀ × max_multiplier))`
//!
//! Treatment allocation follows the same ratio as the planned design.
//!
//! ## Assumptions
//!
//! - Independent observations, common within-group variance (equal-variance
//!   two-sample t-test).
//! - Blinded pooled interim SD is used only to update variance; the planned
//!   treatment effect `Δ` is held fixed (Friede-Kieser blinded SSR rule).
//! - One interim look at fraction `τ` of planned per-arm enrollment.
//! - Superiority framing with a fixed two-sided (or one-sided) alpha.
//!
//! ## Intended validation source
//!
//! - Friede, M., & Kieser, M. (2006). Sample size recalculation for the
//!   t-test. *Biometrical Journal*, 48(4), 590–599.
//! - Planned sample size validated against R `stats::power.t.test`.
//! - Re-estimation arithmetic validated against manual R reference scripts.
//!
//! ## Known limitations
//!
//! - Continuous endpoints only; binary and survival SSR are not implemented.
//! - Single blinded interim look; no repeated re-estimation or unblinded SSR.
//! - Does not simulate Type I error inflation; reports the deterministic
//!   re-estimation rule only.
//! - CHW/CROS/Mehta-Gould exact SSR procedures are out of scope for v1.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::methods::continuous::two_sample_ttest::{self, TwoSampleTTestInput};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for blinded sample size re-estimation planning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlindedSsreInput {
    /// Two-sided or one-sided Type I error rate for the final analysis.
    pub alpha: f64,
    /// Target power for the initial (planned) sample size calculation.
    pub target_power: f64,
    /// Treatment minus control mean difference used for planning.
    pub mean_difference: f64,
    /// Assumed common SD at the design stage (`σ₀`).
    pub planned_standard_deviation: f64,
    /// Blinded pooled interim SD (`s_b`) for what-if re-estimation.
    /// Defaults to `plannedStandardDeviation` when omitted (no variance change).
    pub blinded_interim_standard_deviation: Option<f64>,
    /// Fraction of planned per-arm enrollment observed at the interim look.
    pub interim_fraction: f64,
    /// Treatment-to-control allocation ratio (n_treatment / n_control).
    pub allocation_ratio: f64,
    /// Maximum allowed inflation relative to planned per-arm sample size.
    pub max_sample_size_multiplier: f64,
    pub alternative: Alternative,
}

/// Results for blinded sample size re-estimation planning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlindedSsreResult {
    pub planned_n_control: u32,
    pub planned_n_treatment: u32,
    pub planned_total_n: u32,
    pub interim_n_control: u32,
    pub interim_n_treatment: u32,
    pub interim_total_n: u32,
    pub re_estimated_n_control: u32,
    pub re_estimated_n_treatment: u32,
    pub re_estimated_total_n: u32,
    pub capped_n_control: u32,
    pub capped_n_treatment: u32,
    pub capped_total_n: u32,
    /// Ratio of re-estimated total N to planned total N (before cap).
    pub sample_size_inflation_factor: f64,
    /// Ratio of capped total N to planned total N.
    pub capped_inflation_factor: f64,
    /// `(s_b / σ₀)²`.
    pub variance_ratio: f64,
    /// Achieved power at capped allocation using planned `Δ` and `σ₀`.
    pub achieved_power_at_capped: f64,
    /// Whether the cap reduced the re-estimated sample size.
    pub was_capped: bool,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &BlindedSsreInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_power(input.target_power)?;
    validation::validate_positive("plannedStandardDeviation", input.planned_standard_deviation)?;
    validation::validate_positive("allocationRatio", input.allocation_ratio)?;

    if let Some(sd) = input.blinded_interim_standard_deviation {
        validation::validate_positive("blindedInterimStandardDeviation", sd)?;
    }

    if input.mean_difference == 0.0 {
        return Err(Error::InvalidInput {
            field: "meanDifference".into(),
            message: "must be non-zero".into(),
        });
    }

    if input.interim_fraction <= 0.0 || input.interim_fraction >= 1.0 {
        return Err(Error::InvalidInput {
            field: "interimFraction".into(),
            message: "must be greater than 0 and less than 1".into(),
        });
    }

    if input.max_sample_size_multiplier < 1.0 {
        return Err(Error::InvalidInput {
            field: "maxSampleSizeMultiplier".into(),
            message: "must be at least 1".into(),
        });
    }

    Ok(())
}

/// Plan blinded sample size re-estimation for a two-sample t-test.
pub fn calculate(input: BlindedSsreInput) -> Result<BlindedSsreResult> {
    validate(&input)?;

    let blinded_sd = input
        .blinded_interim_standard_deviation
        .unwrap_or(input.planned_standard_deviation);

    let planned = two_sample_ttest::calculate(TwoSampleTTestInput {
        solve_mode: SolveMode::SampleSize,
        alpha: input.alpha,
        power: Some(input.target_power),
        control_n: None,
        mean_difference: input.mean_difference,
        standard_deviation: input.planned_standard_deviation,
        allocation_ratio: input.allocation_ratio,
        alternative: input.alternative,
        dropout_rate: None,
    })?;

    let variance_ratio = (blinded_sd / input.planned_standard_deviation).powi(2);

    let re_estimated_n_control = (planned.n_control as f64 * variance_ratio).ceil().max(2.0) as u32;
    let re_estimated_n_treatment =
        treatment_n_from_control(re_estimated_n_control, input.allocation_ratio);

    let cap_n_control = (planned.n_control as f64 * input.max_sample_size_multiplier)
        .ceil()
        .max(2.0) as u32;
    let _cap_n_treatment = treatment_n_from_control(cap_n_control, input.allocation_ratio);

    let capped_n_control = re_estimated_n_control.min(cap_n_control);
    let capped_n_treatment = treatment_n_from_control(capped_n_control, input.allocation_ratio);

    let interim_n_control = (planned.n_control as f64 * input.interim_fraction)
        .ceil()
        .max(1.0) as u32;
    let interim_n_treatment = treatment_n_from_control(interim_n_control, input.allocation_ratio);

    let planned_total = planned.n_control + planned.n_treatment;
    let re_estimated_total = re_estimated_n_control + re_estimated_n_treatment;
    let capped_total = capped_n_control + capped_n_treatment;

    let sample_size_inflation_factor = re_estimated_total as f64 / planned_total as f64;
    let capped_inflation_factor = capped_total as f64 / planned_total as f64;
    let was_capped = capped_n_control < re_estimated_n_control;

    let achieved_power_at_capped = two_sample_ttest::achieved_power(
        capped_n_control,
        capped_n_treatment,
        input.mean_difference,
        input.planned_standard_deviation,
        input.alpha,
        input.alternative,
    );

    let warnings = build_warnings(&input, blinded_sd, was_capped);

    Ok(BlindedSsreResult {
        planned_n_control: planned.n_control,
        planned_n_treatment: planned.n_treatment,
        planned_total_n: planned_total,
        interim_n_control,
        interim_n_treatment,
        interim_total_n: interim_n_control + interim_n_treatment,
        re_estimated_n_control,
        re_estimated_n_treatment,
        re_estimated_total_n: re_estimated_total,
        capped_n_control,
        capped_n_treatment,
        capped_total_n: capped_total,
        sample_size_inflation_factor,
        capped_inflation_factor,
        variance_ratio,
        achieved_power_at_capped,
        was_capped,
        warnings,
    })
}

fn treatment_n_from_control(n_control: u32, allocation_ratio: f64) -> u32 {
    (n_control as f64 * allocation_ratio).ceil().max(2.0) as u32
}

fn build_warnings(
    input: &BlindedSsreInput,
    blinded_sd: f64,
    was_capped: bool,
) -> Vec<CalculationWarning> {
    let mut warnings = vec![
        CalculationWarning::new(
            "blinded_variance_only",
            "Uses blinded pooled interim SD to update variance only; the planned treatment effect is held fixed (Friede-Kieser rule).",
        ),
        CalculationWarning::new(
            "continuous_two_sample",
            "Applies to continuous two-sample t-test endpoints with a common within-group SD.",
        ),
        CalculationWarning::new(
            "single_interim_look",
            "Assumes one blinded interim look at the specified enrollment fraction.",
        ),
    ];

    if input.blinded_interim_standard_deviation.is_none() {
        warnings.push(CalculationWarning::new(
            "default_interim_sd",
            "Blinded interim SD defaults to the planned SD; provide a hypothetical interim SD for what-if re-estimation.",
        ));
    }

    if blinded_sd > input.planned_standard_deviation * 1.2 {
        warnings.push(CalculationWarning::new(
            "variance_increase",
            "Blinded interim SD exceeds the planned SD by more than 20%; sample size may increase substantially.",
        ));
    }

    if was_capped {
        warnings.push(CalculationWarning::new(
            "cap_applied",
            "Re-estimated sample size exceeds the maximum multiplier and was reduced to the pre-specified cap.",
        ));
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn base_input(
        blinded_sd: Option<f64>,
        interim_fraction: f64,
        max_multiplier: f64,
    ) -> BlindedSsreInput {
        BlindedSsreInput {
            alpha: 0.05,
            target_power: 0.8,
            mean_difference: 1.0,
            planned_standard_deviation: 1.0,
            blinded_interim_standard_deviation: blinded_sd,
            interim_fraction,
            allocation_ratio: 1.0,
            max_sample_size_multiplier: max_multiplier,
            alternative: Alternative::TwoSided,
        }
    }

    #[test]
    fn planned_sample_size_matches_two_sample_ttest() {
        let result = calculate(base_input(None, 0.5, 1.5)).expect("calculate");

        assert_eq!(result.planned_n_control, 17);
        assert_eq!(result.planned_n_treatment, 17);
        assert_relative_eq!(result.variance_ratio, 1.0, epsilon = 1e-12);
        assert_eq!(result.re_estimated_n_control, 17);
        assert!(!result.was_capped);
    }

    #[test]
    fn re_estimation_matches_manual_reference_when_sd_increases() {
        // R reference: n_planned=17, sb=1.2 -> ratio 1.44 -> n_re=25
        let result = calculate(base_input(Some(1.2), 0.5, 1.5)).expect("calculate");

        assert_eq!(result.planned_n_control, 17);
        assert_relative_eq!(result.variance_ratio, 1.44, epsilon = 1e-12);
        assert_eq!(result.re_estimated_n_control, 25);
        assert_eq!(result.capped_n_control, 25);
        assert!(!result.was_capped);
        assert_relative_eq!(result.achieved_power_at_capped, 0.93371, epsilon = 1e-4);
    }

    #[test]
    fn cap_reduces_re_estimated_sample_size() {
        // Integer planned n=17, sb=1.5 -> ratio 2.25 -> n_re=ceil(38.25)=39, cap at ceil(25.5)=26
        let result = calculate(base_input(Some(1.5), 0.5, 1.5)).expect("calculate");

        assert_eq!(result.re_estimated_n_control, 39);
        assert_eq!(result.capped_n_control, 26);
        assert!(result.was_capped);
        assert_relative_eq!(result.capped_inflation_factor, 26.0 / 17.0, epsilon = 1e-12);
    }

    #[test]
    fn interim_enrollment_scales_with_fraction() {
        let result = calculate(base_input(None, 0.5, 1.5)).expect("calculate");

        assert_eq!(result.interim_n_control, 9);
        assert_eq!(result.interim_n_treatment, 9);
    }

    #[test]
    fn rejects_invalid_interim_fraction() {
        let err = calculate(base_input(None, 1.0, 1.5)).expect_err("invalid");
        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[test]
    fn rejects_multiplier_below_one() {
        let err = calculate(base_input(None, 0.5, 0.9)).expect_err("invalid");
        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[test]
    fn unequal_allocation_preserves_ratio() {
        let input = BlindedSsreInput {
            allocation_ratio: 2.0,
            ..base_input(Some(1.2), 0.5, 1.5)
        };
        let result = calculate(input).expect("calculate");

        assert_eq!(
            result.re_estimated_n_treatment,
            treatment_n_from_control(result.re_estimated_n_control, 2.0)
        );
    }
}
