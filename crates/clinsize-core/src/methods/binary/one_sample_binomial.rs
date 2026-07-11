//! One-sample binomial (single-arm response rate) sample size and power.

use serde::{Deserialize, Serialize};

use crate::distributions::normal;
use crate::error::{Error, Result};
use crate::methods::continuous::single_sample_t;
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, SolveMode};
use crate::validation;

/// Inputs for a one-sample binomial proportion test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneSampleBinomialInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    pub n: Option<u32>,
    /// Reference or null response rate.
    pub reference_rate: f64,
    /// Hypothesized response rate under the alternative.
    pub response_rate: f64,
    pub alternative: Alternative,
    pub dropout_rate: Option<f64>,
}

/// Results for a one-sample binomial proportion test.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OneSampleBinomialResult {
    pub n: u32,
    pub n_adjusted: u32,
    pub achieved_power: f64,
    pub rate_difference: f64,
    pub warnings: Vec<CalculationWarning>,
}

fn tail_multiplier(alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => 2.0,
        Alternative::Greater | Alternative::Less => 1.0,
    }
}

fn rate_shift(reference_rate: f64, response_rate: f64, alternative: Alternative) -> f64 {
    let diff = response_rate - reference_rate;
    match alternative {
        Alternative::TwoSided => diff.abs(),
        Alternative::Greater => diff.max(0.0),
        Alternative::Less => (-diff).max(0.0),
    }
}

pub fn validate(input: &OneSampleBinomialInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_probability("reference_rate", input.reference_rate)?;
    validation::validate_probability("response_rate", input.response_rate)?;

    if input.reference_rate == input.response_rate {
        return Err(Error::InvalidInput {
            field: "responseRate".into(),
            message: "must differ from reference rate".into(),
        });
    }

    match input.alternative {
        Alternative::Greater if input.response_rate <= input.reference_rate => {
            return Err(Error::InvalidInput {
                field: "responseRate".into(),
                message: "must be greater than reference rate for a greater alternative".into(),
            });
        }
        Alternative::Less if input.response_rate >= input.reference_rate => {
            return Err(Error::InvalidInput {
                field: "responseRate".into(),
                message: "must be less than reference rate for a less alternative".into(),
            });
        }
        _ => {}
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
            if input.n.is_some() {
                return Err(Error::InvalidInput {
                    field: "n".into(),
                    message: "must not be set when solving for sample size".into(),
                });
            }
        }
        SolveMode::Power => {
            let n = input.n.ok_or_else(|| Error::InvalidInput {
                field: "n".into(),
                message: "is required when solving for power".into(),
            })?;
            if n < 1 {
                return Err(Error::InvalidInput {
                    field: "n".into(),
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
                "detectable effect solve mode is not implemented for one-sample binomial".into(),
            ));
        }
    }

    Ok(())
}

pub fn achieved_power(
    n: u32,
    reference_rate: f64,
    response_rate: f64,
    alpha: f64,
    alternative: Alternative,
) -> f64 {
    let shift = rate_shift(reference_rate, response_rate, alternative);
    if shift <= 0.0 {
        return alpha;
    }
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let numerator =
        shift * (n as f64).sqrt() - z_alpha * (reference_rate * (1.0 - reference_rate)).sqrt();
    let z = numerator / (response_rate * (1.0 - response_rate)).sqrt();
    match alternative {
        Alternative::TwoSided => (2.0 * normal::cdf(z) - 1.0).clamp(0.0, 1.0),
        Alternative::Greater | Alternative::Less => normal::cdf(z).clamp(0.0, 1.0),
    }
}

fn solve_sample_size(
    reference_rate: f64,
    response_rate: f64,
    alpha: f64,
    target_power: f64,
    alternative: Alternative,
) -> Result<u32> {
    let shift = rate_shift(reference_rate, response_rate, alternative);
    if shift <= 0.0 {
        return Err(Error::InvalidInput {
            field: "responseRate".into(),
            message: "does not support the requested alternative".into(),
        });
    }

    find_minimum_integer(1, MAX_SAMPLE_SIZE_SEARCH, |n| {
        achieved_power(n, reference_rate, response_rate, alpha, alternative) >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
        ))
    })
}

pub fn calculate(input: OneSampleBinomialInput) -> Result<OneSampleBinomialResult> {
    validate(&input)?;

    let warnings = vec![
        CalculationWarning::new(
            "one_sample_binomial",
            "Compares a single-arm response rate to a reference proportion.",
        ),
        CalculationWarning::new(
            "normal_approximation",
            "Uses a normal approximation to the binomial distribution; exact methods are not implemented.",
        ),
    ];

    let (n, achieved_power) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n = solve_sample_size(
                input.reference_rate,
                input.response_rate,
                input.alpha,
                target_power,
                input.alternative,
            )?;
            let power = achieved_power(
                n,
                input.reference_rate,
                input.response_rate,
                input.alpha,
                input.alternative,
            );
            (n, power)
        }
        SolveMode::Power => {
            let n = input.n.expect("validated");
            let power = achieved_power(
                n,
                input.reference_rate,
                input.response_rate,
                input.alpha,
                input.alternative,
            );
            (n, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    Ok(OneSampleBinomialResult {
        n,
        n_adjusted: single_sample_t::apply_dropout(n, input.dropout_rate),
        achieved_power,
        rate_difference: input.response_rate - input.reference_rate,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn sample_size_input(
        reference_rate: f64,
        response_rate: f64,
        alpha: f64,
        power: f64,
        alternative: Alternative,
    ) -> OneSampleBinomialInput {
        OneSampleBinomialInput {
            solve_mode: SolveMode::SampleSize,
            alpha,
            power: Some(power),
            n: None,
            reference_rate,
            response_rate,
            alternative,
            dropout_rate: None,
        }
    }

    #[test]
    fn matches_normal_approx_reference_case() {
        let result = calculate(sample_size_input(
            0.2,
            0.4,
            0.05,
            0.8,
            Alternative::TwoSided,
        ))
        .expect("calculate");

        assert_eq!(result.n, 50);
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }

    #[test]
    fn second_reference_case() {
        let result = calculate(sample_size_input(
            0.3,
            0.5,
            0.05,
            0.8,
            Alternative::TwoSided,
        ))
        .expect("calculate");

        assert_eq!(result.n, 60);
    }
}
