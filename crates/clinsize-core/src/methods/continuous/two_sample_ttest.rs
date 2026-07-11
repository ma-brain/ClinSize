//! Two-sample t-test (equal variance) sample size and power.

use serde::{Deserialize, Serialize};

use crate::distributions::{noncentral_t, student_t};
use crate::error::{Error, Result};
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for the equal-variance two-sample t-test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoSampleTTestInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    /// Target power when solving for sample size.
    pub power: Option<f64>,
    /// Control-group size when solving for power.
    pub control_n: Option<u32>,
    /// Treatment minus control mean difference.
    pub mean_difference: f64,
    /// Common within-group standard deviation.
    pub standard_deviation: f64,
    /// Treatment-to-control allocation ratio (n_treatment / n_control).
    pub allocation_ratio: f64,
    pub alternative: Alternative,
    /// Optional dropout rate in [0, 1) applied after sample-size rounding.
    pub dropout_rate: Option<f64>,
}

/// Results for the equal-variance two-sample t-test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoSampleTTestResult {
    pub n_control: u32,
    pub n_treatment: u32,
    pub total_n: u32,
    pub n_control_adjusted: u32,
    pub n_treatment_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    pub effect_size: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &TwoSampleTTestInput) -> Result<()> {
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
                "detectable effect solve mode is not implemented for two-sample t-test".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: TwoSampleTTestInput) -> Result<TwoSampleTTestResult> {
    validate(&input)?;

    let mut warnings = vec![
        CalculationWarning::new(
            "equal_variance",
            "Assumes a common within-group standard deviation (equal-variance t-test).",
        ),
        CalculationWarning::new(
            "exact_t_power",
            "Uses exact t-distribution power with noncentral t; sample size is the smallest integer allocation meeting the target power after rounding.",
        ),
    ];

    let effect_size = input.mean_difference / input.standard_deviation;

    let (n_control, n_treatment, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_control = solve_control_n(&input, target_power)?;
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let power = achieved_power(
                n_control,
                n_treatment,
                input.mean_difference,
                input.standard_deviation,
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
                input.mean_difference,
                input.standard_deviation,
                input.alpha,
                input.alternative,
            );
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

    Ok(TwoSampleTTestResult {
        n_control,
        n_treatment,
        total_n: n_control + n_treatment,
        n_control_adjusted,
        n_treatment_adjusted,
        total_n_adjusted: n_control_adjusted + n_treatment_adjusted,
        achieved_power,
        effect_size,
        warnings,
    })
}

fn treatment_n_from_control(n_control: u32, allocation_ratio: f64) -> u32 {
    (n_control as f64 * allocation_ratio).ceil().max(2.0) as u32
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

fn noncentrality_parameter(
    n_control: u32,
    n_treatment: u32,
    mean_difference: f64,
    standard_deviation: f64,
) -> f64 {
    mean_difference
        / (standard_deviation * (1.0 / n_control as f64 + 1.0 / n_treatment as f64).sqrt())
}

pub fn achieved_power(
    n_control: u32,
    n_treatment: u32,
    mean_difference: f64,
    standard_deviation: f64,
    alpha: f64,
    alternative: Alternative,
) -> f64 {
    let df = (n_control + n_treatment - 2) as f64;
    let ncp = noncentrality_parameter(n_control, n_treatment, mean_difference, standard_deviation);

    match alternative {
        Alternative::TwoSided => {
            let t_crit = student_t::quantile(1.0 - alpha / 2.0, df);
            1.0 - noncentral_t::cdf(t_crit, df, ncp) + noncentral_t::cdf(-t_crit, df, ncp)
        }
        Alternative::Greater => {
            let t_crit = student_t::quantile(1.0 - alpha, df);
            1.0 - noncentral_t::cdf(t_crit, df, ncp)
        }
        Alternative::Less => {
            let t_crit = student_t::quantile(1.0 - alpha, df);
            noncentral_t::cdf(-t_crit, df, ncp)
        }
    }
}

fn solve_control_n(input: &TwoSampleTTestInput, target_power: f64) -> Result<u32> {
    find_minimum_integer(2, MAX_SAMPLE_SIZE_SEARCH, |n_control| {
        let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
        let power = achieved_power(
            n_control,
            n_treatment,
            input.mean_difference,
            input.standard_deviation,
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

    fn sample_size_input(
        mean_difference: f64,
        standard_deviation: f64,
        alpha: f64,
        power: f64,
        alternative: Alternative,
        allocation_ratio: f64,
        dropout_rate: Option<f64>,
    ) -> TwoSampleTTestInput {
        TwoSampleTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha,
            power: Some(power),
            control_n: None,
            mean_difference,
            standard_deviation,
            allocation_ratio,
            alternative,
            dropout_rate,
        }
    }

    #[test]
    fn rejects_zero_mean_difference() {
        let input = sample_size_input(0.0, 1.0, 0.05, 0.8, Alternative::TwoSided, 1.0, None);
        assert!(validate(&input).is_err());
    }

    #[test]
    fn rejects_missing_power_for_sample_size_mode() {
        let input = TwoSampleTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: None,
            control_n: None,
            mean_difference: 1.0,
            standard_deviation: 1.0,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        assert!(validate(&input).is_err());
    }

    #[test]
    fn matches_r_power_t_test_equal_allocation_two_sided() {
        // R: power.t.test(delta=1, sd=1, sig.level=0.05, power=0.8, type="two.sample")
        let result = calculate(sample_size_input(
            1.0,
            1.0,
            0.05,
            0.8,
            Alternative::TwoSided,
            1.0,
            None,
        ))
        .expect("calculate");

        assert_eq!(result.n_control, 17);
        assert_eq!(result.n_treatment, 17);
        assert_relative_eq!(result.achieved_power, 0.80704, epsilon = 1e-4);
        assert_relative_eq!(result.effect_size, 1.0, epsilon = 1e-12);
    }

    #[test]
    fn matches_r_power_t_test_equal_allocation_smaller_effect() {
        // R: power.t.test(delta=0.5, sd=1, sig.level=0.05, power=0.8, type="two.sample")
        let result = calculate(sample_size_input(
            0.5,
            1.0,
            0.05,
            0.8,
            Alternative::TwoSided,
            1.0,
            None,
        ))
        .expect("calculate");

        assert_eq!(result.n_control, 64);
        assert_eq!(result.n_treatment, 64);
        assert_relative_eq!(result.achieved_power, 0.80146, epsilon = 1e-4);
    }

    #[test]
    fn matches_r_power_t_test_one_sided() {
        // R: power.t.test(..., alternative="one.sided") maps to greater for positive delta
        let result = calculate(sample_size_input(
            1.0,
            1.0,
            0.05,
            0.8,
            Alternative::Greater,
            1.0,
            None,
        ))
        .expect("calculate");

        assert_eq!(result.n_control, 14);
        assert_eq!(result.n_treatment, 14);
        assert_relative_eq!(result.achieved_power, 0.82409, epsilon = 1e-4);
    }

    #[test]
    fn matches_r_achieved_power_at_fixed_n() {
        // R: power.t.test(n=17, delta=1, sd=1, sig.level=0.05, type="two.sample")
        let power = achieved_power(17, 17, 1.0, 1.0, 0.05, Alternative::TwoSided);
        assert_relative_eq!(power, 0.80704, epsilon = 1e-4);
    }

    #[test]
    fn unequal_allocation_ratio_two_needs_fewer_controls() {
        let equal = calculate(sample_size_input(
            1.0,
            1.0,
            0.05,
            0.8,
            Alternative::TwoSided,
            1.0,
            None,
        ))
        .expect("equal");
        let unequal = calculate(sample_size_input(
            1.0,
            1.0,
            0.05,
            0.8,
            Alternative::TwoSided,
            2.0,
            None,
        ))
        .expect("unequal");

        assert!(unequal.n_control < equal.n_control);
        assert_eq!(
            unequal.n_treatment,
            treatment_n_from_control(unequal.n_control, 2.0)
        );
        assert!(unequal.achieved_power >= 0.8);
    }

    #[test]
    fn dropout_inflation_rounds_up() {
        let result = calculate(sample_size_input(
            1.0,
            1.0,
            0.05,
            0.8,
            Alternative::TwoSided,
            1.0,
            Some(0.2),
        ))
        .expect("calculate");

        assert_eq!(result.n_control, 17);
        assert_eq!(result.n_control_adjusted, 22);
        assert_eq!(result.n_treatment_adjusted, 22);
    }

    #[test]
    fn power_mode_uses_supplied_control_n() {
        let input = TwoSampleTTestInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            control_n: Some(17),
            mean_difference: 1.0,
            standard_deviation: 1.0,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.n_control, 17);
        assert_relative_eq!(result.achieved_power, 0.80704, epsilon = 1e-4);
    }
}
