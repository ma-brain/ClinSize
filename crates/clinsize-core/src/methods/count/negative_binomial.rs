//! Two-sample negative binomial sample size and power for recurrent event counts.
//!
//! Zhu & Lakkis (2014) Wald test for the log rate ratio (gsDesignNB Method 3).

use serde::{Deserialize, Serialize};

use crate::distributions::normal;
use crate::error::{Error, Result};
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for two-sample negative binomial comparison of event rates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NegativeBinomialInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub control_n: Option<u32>,
    /// Control-group event rate λ₁ (events per unit exposure).
    pub control_rate: f64,
    /// Treatment-group event rate λ₂.
    pub treatment_rate: f64,
    /// NB2 dispersion parameter k (common across arms).
    pub dispersion: f64,
    /// Exposure time per subject (default 1).
    #[serde(default = "default_exposure_time")]
    pub exposure_time: f64,
    /// Treatment-to-control allocation ratio (n_treatment / n_control).
    pub allocation_ratio: f64,
    pub alternative: Alternative,
    pub dropout_rate: Option<f64>,
}

fn default_exposure_time() -> f64 {
    1.0
}

/// Results for two-sample negative binomial comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NegativeBinomialResult {
    pub n_control: u32,
    pub n_treatment: u32,
    pub total_n: u32,
    pub n_control_adjusted: u32,
    pub n_treatment_adjusted: u32,
    pub total_n_adjusted: u32,
    pub achieved_power: f64,
    pub rate_ratio: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn rate_ratio(control_rate: f64, treatment_rate: f64) -> f64 {
    treatment_rate / control_rate
}

pub fn log_rate_ratio(control_rate: f64, treatment_rate: f64) -> f64 {
    (treatment_rate / control_rate).ln()
}

pub fn variance_factor(
    control_rate: f64,
    treatment_rate: f64,
    dispersion: f64,
    exposure_time: f64,
    allocation_ratio: f64,
) -> f64 {
    let mu1 = control_rate * exposure_time;
    let mu2 = treatment_rate * exposure_time;
    let r = allocation_ratio;
    (1.0 / mu1 + dispersion) + (1.0 / r) * (1.0 / mu2 + dispersion)
}

pub fn validate(input: &NegativeBinomialInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_positive("control_rate", input.control_rate)?;
    validation::validate_positive("treatment_rate", input.treatment_rate)?;
    validation::validate_positive("dispersion", input.dispersion)?;
    validation::validate_positive("exposure_time", input.exposure_time)?;
    validation::validate_positive("allocation_ratio", input.allocation_ratio)?;

    if input.control_rate == input.treatment_rate {
        return Err(Error::InvalidInput {
            field: "treatmentRate".into(),
            message: "must differ from control rate".into(),
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
                "detectable effect solve mode is not implemented for negative binomial".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: NegativeBinomialInput) -> Result<NegativeBinomialResult> {
    validate(&input)?;

    let mut warnings = vec![
        CalculationWarning::new(
            "nb2_wald",
            "Uses Zhu & Lakkis (2014) Wald test for the log rate ratio with NB2 variance Var(Y) = μ + kμ² and common dispersion k.",
        ),
        CalculationWarning::new(
            "fixed_exposure",
            "Assumes fixed exposure time per subject and independent negative binomial counts.",
        ),
    ];

    let rr = rate_ratio(input.control_rate, input.treatment_rate);

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

    Ok(NegativeBinomialResult {
        n_control,
        n_treatment,
        total_n: n_control + n_treatment,
        n_control_adjusted,
        n_treatment_adjusted,
        total_n_adjusted: n_control_adjusted + n_treatment_adjusted,
        achieved_power,
        rate_ratio: rr,
        warnings,
    })
}

fn treatment_n_from_control(n_control: u32, allocation_ratio: f64) -> u32 {
    (n_control as f64 * allocation_ratio).ceil().max(2.0) as u32
}

fn z_critical(alpha: f64, alternative: Alternative) -> f64 {
    let tails = match alternative {
        Alternative::TwoSided => 2.0,
        Alternative::Greater | Alternative::Less => 1.0,
    };
    normal::upper_tail_critical(alpha / tails)
}

pub fn achieved_power(n_control: u32, _n_treatment: u32, input: &NegativeBinomialInput) -> f64 {
    let theta = log_rate_ratio(input.control_rate, input.treatment_rate).abs();
    let v_tilde = variance_factor(
        input.control_rate,
        input.treatment_rate,
        input.dispersion,
        input.exposure_time,
        input.allocation_ratio,
    );
    let se = (v_tilde / n_control as f64).sqrt();
    let z_alpha = z_critical(input.alpha, input.alternative);
    normal::cdf(theta / se - z_alpha)
}

fn closed_form_control_n(input: &NegativeBinomialInput, target_power: f64) -> u32 {
    let theta = log_rate_ratio(input.control_rate, input.treatment_rate);
    let v_tilde = variance_factor(
        input.control_rate,
        input.treatment_rate,
        input.dispersion,
        input.exposure_time,
        input.allocation_ratio,
    );
    let z_alpha = z_critical(input.alpha, input.alternative);
    let z_beta = normal::quantile(target_power);
    (theta.abs().powi(-2) * (z_alpha + z_beta).powi(2) * v_tilde)
        .ceil()
        .max(2.0) as u32
}

fn solve_control_n(input: &NegativeBinomialInput, target_power: f64) -> Result<u32> {
    let closed = closed_form_control_n(input, target_power);
    find_minimum_integer(closed.saturating_sub(1).max(2), MAX_SAMPLE_SIZE_SEARCH, |n_control| {
        let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
        achieved_power(n_control, n_treatment, input) >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a control-group sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
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

    fn sample_size_input(
        control_rate: f64,
        treatment_rate: f64,
        dispersion: f64,
        alpha: f64,
        power: f64,
        allocation_ratio: f64,
    ) -> NegativeBinomialInput {
        NegativeBinomialInput {
            solve_mode: SolveMode::SampleSize,
            alpha,
            power: Some(power),
            control_n: None,
            control_rate,
            treatment_rate,
            dispersion,
            exposure_time: 1.0,
            allocation_ratio,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        }
    }

    #[test]
    fn rejects_equal_rates() {
        let input = sample_size_input(2.0, 2.0, 1.0, 0.05, 0.8, 1.0);
        assert!(validate(&input).is_err());
    }

    #[test]
    fn matches_zhu_lakkis_halving_rate() {
        // λ₁=2, λ₂=1, k=1, exposure=1, ratio=1, α=0.05, power=0.8 → n_control=58
        let result = calculate(sample_size_input(2.0, 1.0, 1.0, 0.05, 0.8, 1.0)).expect("calculate");

        assert_eq!(result.n_control, 58);
        assert_eq!(result.n_treatment, 58);
        assert_eq!(result.total_n, 116);
        assert_relative_eq!(result.rate_ratio, 0.5, epsilon = 1e-12);
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }

    #[test]
    fn matches_zhu_lakkis_doubling_rate() {
        // λ₁=5, λ₂=10, k=0.5, exposure=1 → n_control=22, total=44
        let result = calculate(sample_size_input(5.0, 10.0, 0.5, 0.05, 0.8, 1.0)).expect("calculate");

        assert_eq!(result.n_control, 22);
        assert_eq!(result.n_treatment, 22);
        assert_eq!(result.total_n, 44);
        assert_relative_eq!(result.rate_ratio, 2.0, epsilon = 1e-12);
    }

    #[test]
    fn power_mode_uses_supplied_control_n() {
        let input = NegativeBinomialInput {
            solve_mode: SolveMode::Power,
            alpha: 0.05,
            power: None,
            control_n: Some(58),
            control_rate: 2.0,
            treatment_rate: 1.0,
            dispersion: 1.0,
            exposure_time: 1.0,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate(input).expect("calculate");
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }
}
