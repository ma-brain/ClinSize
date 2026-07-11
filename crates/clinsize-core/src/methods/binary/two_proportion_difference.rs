//! Two-sample difference in proportions (normal approximation).

use serde::{Deserialize, Serialize};

use super::shared::{power_prop_test_equal, power_two_proportion, treatment_n_from_control};
use crate::distributions::normal;
use crate::error::{Error, Result};
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, SolveMode, StudyObjective};
use crate::validation;

/// Inputs for two-sample binary comparison via difference in proportions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoProportionDifferenceInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub control_n: Option<u32>,
    pub control_rate: f64,
    pub treatment_rate: f64,
    pub allocation_ratio: f64,
    pub alternative: Alternative,
    pub study_objective: StudyObjective,
    /// Allowed deficit for non-inferiority (higher-is-better). Required when objective is NI.
    pub noninferiority_margin: Option<f64>,
    pub dropout_rate: Option<f64>,
}

/// Results for two-sample difference in proportions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwoProportionDifferenceResult {
    pub n_control: u32,
    pub n_treatment: u32,
    pub total_n: u32,
    pub n_control_adjusted: u32,
    pub n_treatment_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    pub rate_difference: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &TwoProportionDifferenceInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_probability("control_rate", input.control_rate)?;
    validation::validate_probability("treatment_rate", input.treatment_rate)?;
    validation::validate_positive("allocation_ratio", input.allocation_ratio)?;

    match input.study_objective {
        StudyObjective::Superiority => {
            if input.control_rate == input.treatment_rate {
                return Err(Error::InvalidInput {
                    field: "treatmentRate".into(),
                    message: "must differ from control rate for superiority".into(),
                });
            }
            if input.noninferiority_margin.is_some() {
                return Err(Error::InvalidInput {
                    field: "noninferiorityMargin".into(),
                    message: "must not be set for superiority".into(),
                });
            }
        }
        StudyObjective::NonInferiority => {
            let margin = input
                .noninferiority_margin
                .ok_or_else(|| Error::InvalidInput {
                    field: "noninferiorityMargin".into(),
                    message: "is required for non-inferiority".into(),
                })?;
            if margin <= 0.0 || margin >= 1.0 {
                return Err(Error::InvalidInput {
                    field: "noninferiorityMargin".into(),
                    message: "must be greater than 0 and less than 1".into(),
                });
            }
            if input.alternative == Alternative::TwoSided {
                return Err(Error::InvalidInput {
                    field: "alternative".into(),
                    message: "non-inferiority requires a one-sided alternative".into(),
                });
            }
        }
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
                "detectable effect solve mode is not implemented for two-proportion difference"
                    .into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: TwoProportionDifferenceInput) -> Result<TwoProportionDifferenceResult> {
    validate(&input)?;

    let mut warnings = vec![
        CalculationWarning::new(
            "normal_approximation",
            "Uses a normal approximation to the binomial distribution (R `power.prop.test` for superiority; Chow et al. 2003 for non-inferiority).",
        ),
        CalculationWarning::new(
            "higher_is_better",
            "Assumes a higher event rate is favorable. Treatment minus control defines the rate difference.",
        ),
    ];

    let rate_difference = input.treatment_rate - input.control_rate;

    let (n_control, n_treatment, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_control = solve_control_n(&input, target_power)?;
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let power = achieved_power(n_control, n_treatment, &input);
            (n_control, n_treatment, power)
        }
        SolveMode::Power => {
            let n_control = input.control_n.expect("validated");
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let power = achieved_power(n_control, n_treatment, &input);
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

    if matches!(input.study_objective, StudyObjective::NonInferiority) {
        warnings.push(CalculationWarning::new(
            "noninferiority_margin",
            "Non-inferiority margin is the maximum acceptable deficit (control minus treatment) for a higher-is-better endpoint.",
        ));
    }

    Ok(TwoProportionDifferenceResult {
        n_control,
        n_treatment,
        total_n: n_control + n_treatment,
        n_control_adjusted,
        n_treatment_adjusted,
        total_n_adjusted: n_control_adjusted + n_treatment_adjusted,
        achieved_power,
        rate_difference,
        warnings,
    })
}

fn achieved_power(n_control: u32, n_treatment: u32, input: &TwoProportionDifferenceInput) -> f64 {
    match input.study_objective {
        StudyObjective::Superiority => power_two_proportion(
            n_control,
            n_treatment,
            input.control_rate,
            input.treatment_rate,
            input.alpha,
            input.alternative,
        ),
        StudyObjective::NonInferiority => power_noninferiority(
            n_control,
            n_treatment,
            input.control_rate,
            input.treatment_rate,
            input.alpha,
            input.noninferiority_margin.expect("validated"),
        ),
    }
}

fn power_noninferiority(
    n_control: u32,
    n_treatment: u32,
    p_control: f64,
    p_treatment: f64,
    alpha: f64,
    margin: f64,
) -> f64 {
    let delta = p_treatment - p_control;
    let se = (p_control * (1.0 - p_control) / n_control as f64
        + p_treatment * (1.0 - p_treatment) / n_treatment as f64)
        .sqrt();
    let z_alpha = normal::upper_tail_critical(alpha);
    normal::cdf((delta + margin) / se - z_alpha)
}

fn solve_control_n(input: &TwoProportionDifferenceInput, target_power: f64) -> Result<u32> {
    match input.study_objective {
        StudyObjective::Superiority if (input.allocation_ratio - 1.0).abs() < f64::EPSILON => {
            solve_equal_superiority_n(input, target_power)
        }
        StudyObjective::Superiority => solve_unequal_superiority_n(input, target_power),
        StudyObjective::NonInferiority => solve_noninferiority_n(input, target_power),
    }
}

fn solve_equal_superiority_n(
    input: &TwoProportionDifferenceInput,
    target_power: f64,
) -> Result<u32> {
    find_minimum_integer(2, MAX_SAMPLE_SIZE_SEARCH, |n| {
        power_prop_test_equal(
            n as f64,
            input.control_rate,
            input.treatment_rate,
            input.alpha,
            input.alternative,
        ) >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a per-group sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
        ))
    })
}

fn solve_unequal_superiority_n(
    input: &TwoProportionDifferenceInput,
    target_power: f64,
) -> Result<u32> {
    find_minimum_integer(2, MAX_SAMPLE_SIZE_SEARCH, |n_control| {
        let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
        power_two_proportion(
            n_control,
            n_treatment,
            input.control_rate,
            input.treatment_rate,
            input.alpha,
            input.alternative,
        ) >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a control-group sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
        ))
    })
}

fn solve_noninferiority_n(input: &TwoProportionDifferenceInput, target_power: f64) -> Result<u32> {
    let margin = input.noninferiority_margin.expect("validated");
    let delta = input.treatment_rate - input.control_rate;
    let k = input.allocation_ratio;
    let z_alpha = normal::upper_tail_critical(input.alpha);
    let z_beta = normal::quantile(target_power);
    let p_bar = (input.treatment_rate + k * input.control_rate) / (1.0 + k);
    let numerator = z_alpha * ((1.0 + 1.0 / k) * p_bar * (1.0 - p_bar)).sqrt()
        + z_beta
            * (input.treatment_rate * (1.0 - input.treatment_rate) / k
                + input.control_rate * (1.0 - input.control_rate))
                .sqrt();
    let denominator = delta - (-margin);
    if denominator <= 0.0 {
        return Err(Error::InvalidInput {
            field: "treatmentRate".into(),
            message: "expected treatment-control difference must exceed the non-inferiority margin"
                .into(),
        });
    }
    let n_control = (numerator.powi(2) / denominator.powi(2)).ceil().max(2.0) as u32;
    Ok(n_control)
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

    fn superiority_input(
        control_rate: f64,
        treatment_rate: f64,
        alpha: f64,
        power: f64,
        alternative: Alternative,
        allocation_ratio: f64,
    ) -> TwoProportionDifferenceInput {
        TwoProportionDifferenceInput {
            solve_mode: SolveMode::SampleSize,
            alpha,
            power: Some(power),
            control_n: None,
            control_rate,
            treatment_rate,
            allocation_ratio,
            alternative,
            study_objective: StudyObjective::Superiority,
            noninferiority_margin: None,
            dropout_rate: None,
        }
    }

    #[test]
    fn matches_r_power_prop_test_two_sided() {
        let result = calculate(superiority_input(
            0.3,
            0.45,
            0.05,
            0.8,
            Alternative::TwoSided,
            1.0,
        ))
        .expect("calculate");

        assert_eq!(result.n_control, 163);
        assert_eq!(result.n_treatment, 163);
        assert_relative_eq!(result.achieved_power, 0.8016161, epsilon = 1e-4);
        assert_relative_eq!(result.rate_difference, 0.15, epsilon = 1e-12);
    }

    #[test]
    fn matches_r_power_prop_test_one_sided() {
        let result = calculate(superiority_input(
            0.3,
            0.45,
            0.05,
            0.8,
            Alternative::Greater,
            1.0,
        ))
        .expect("calculate");

        assert_eq!(result.n_control, 128);
        assert_eq!(result.n_treatment, 128);
        assert_relative_eq!(result.achieved_power, 0.801, epsilon = 1e-3);
    }

    #[test]
    fn matches_r_achieved_power_at_fixed_n() {
        let input = TwoProportionDifferenceInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            control_n: Some(163),
            control_rate: 0.3,
            treatment_rate: 0.45,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            study_objective: StudyObjective::Superiority,
            noninferiority_margin: None,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_relative_eq!(result.achieved_power, 0.8016161, epsilon = 1e-4);
    }

    #[test]
    fn noninferiority_matches_trial_size() {
        let input = TwoProportionDifferenceInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            control_rate: 0.3,
            treatment_rate: 0.35,
            allocation_ratio: 1.0,
            alternative: Alternative::Greater,
            study_objective: StudyObjective::NonInferiority,
            noninferiority_margin: Some(0.1),
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_eq!(result.n_control, 121);
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }
}
