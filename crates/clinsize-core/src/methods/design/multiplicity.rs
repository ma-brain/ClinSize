//! Family-wise alpha adjustment for multiple comparisons.

use serde::{Deserialize, Serialize};

use crate::distributions::equicorrelated_normal;
use crate::error::{Error, Result};
use crate::types::CalculationWarning;
use crate::validation;

/// Procedure for converting a family-wise Type I error rate into a
/// per-comparison alpha for sample size planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiplicityMethod {
    /// `alpha_adj = alpha_family / m`. Valid under any dependence structure.
    Bonferroni,
    /// `alpha_adj = 1 - (1 - alpha_family)^(1/m)`. Assumes independent tests.
    Sidak,
    /// Dunnett adjustment for `m` treatment arms vs a common control (equal group sizes).
    Dunnett,
    /// Holm step-down gatekeeping for a fixed-order hypothesis at position `k`.
    Holm,
}

/// Inputs for a family-wise alpha adjustment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiplicityInput {
    /// Family-wise Type I error rate to control across all comparisons.
    pub family_wise_alpha: f64,
    /// Number of comparisons in the family.
    pub number_of_comparisons: u32,
    pub adjustment_method: MultiplicityMethod,
    /// Position in the gatekeeping sequence (1 = first), required for Holm.
    pub gate_position: Option<u32>,
}

/// Results for a family-wise alpha adjustment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiplicityResult {
    pub adjusted_alpha: f64,
    pub family_wise_alpha: f64,
    pub number_of_comparisons: u32,
    pub adjustment_method: MultiplicityMethod,
    /// Gate position when Holm gatekeeping is used.
    pub gate_position: Option<u32>,
    /// Ratio of adjusted alpha to the unadjusted per-comparison alpha if the
    /// family-wise rate were split naively without adjustment logic.
    pub alpha_reduction_factor: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &MultiplicityInput) -> Result<()> {
    validation::validate_alpha(input.family_wise_alpha)?;
    validation::validate_comparison_count(input.number_of_comparisons)?;

    if input.adjustment_method == MultiplicityMethod::Holm {
        let gate_position = input.gate_position.ok_or_else(|| Error::InvalidInput {
            field: "gatePosition".into(),
            message: "is required when using Holm gatekeeping".into(),
        })?;
        validation::validate_gate_position(gate_position, input.number_of_comparisons)?;
    } else if input.gate_position.is_some() {
        return Err(Error::InvalidInput {
            field: "gatePosition".into(),
            message: "must not be set unless using Holm gatekeeping".into(),
        });
    }

    Ok(())
}

/// Compute a per-comparison alpha from a family-wise error rate.
pub fn calculate(input: MultiplicityInput) -> Result<MultiplicityResult> {
    validate(&input)?;

    let adjusted_alpha = match input.adjustment_method {
        MultiplicityMethod::Bonferroni => {
            bonferroni_alpha(input.family_wise_alpha, input.number_of_comparisons)
        }
        MultiplicityMethod::Sidak => {
            sidak_alpha(input.family_wise_alpha, input.number_of_comparisons)
        }
        MultiplicityMethod::Dunnett => {
            dunnett_alpha(input.family_wise_alpha, input.number_of_comparisons)?
        }
        MultiplicityMethod::Holm => holm_alpha(
            input.family_wise_alpha,
            input.number_of_comparisons,
            input.gate_position.expect("validated above"),
        ),
    };

    if adjusted_alpha <= 0.0 {
        return Err(Error::InvalidInput {
            field: "numberOfComparisons".into(),
            message: "produces a non-positive adjusted alpha; reduce the number of comparisons or increase the family-wise alpha".into(),
        });
    }

    let alpha_reduction_factor = adjusted_alpha / input.family_wise_alpha;
    let warnings = build_warnings(&input);

    Ok(MultiplicityResult {
        adjusted_alpha,
        family_wise_alpha: input.family_wise_alpha,
        number_of_comparisons: input.number_of_comparisons,
        adjustment_method: input.adjustment_method,
        gate_position: input.gate_position,
        alpha_reduction_factor,
        warnings,
    })
}

fn bonferroni_alpha(family_wise_alpha: f64, number_of_comparisons: u32) -> f64 {
    family_wise_alpha / f64::from(number_of_comparisons)
}

fn sidak_alpha(family_wise_alpha: f64, number_of_comparisons: u32) -> f64 {
    1.0 - (1.0 - family_wise_alpha).powf(1.0 / f64::from(number_of_comparisons))
}

fn dunnett_alpha(family_wise_alpha: f64, treatment_arms: u32) -> Result<f64> {
    equicorrelated_normal::dunnett_two_sided_adjusted_alpha(treatment_arms, family_wise_alpha)
        .ok_or_else(|| Error::ConvergenceFailure(
            "failed to solve Dunnett critical value for the requested alpha and number of treatment arms".into(),
        ))
}

/// Holm step-down local alpha for gate `k` in a family of `m` ordered hypotheses.
fn holm_alpha(family_wise_alpha: f64, number_of_comparisons: u32, gate_position: u32) -> f64 {
    let remaining = number_of_comparisons - gate_position + 1;
    family_wise_alpha / f64::from(remaining)
}

fn build_warnings(input: &MultiplicityInput) -> Vec<CalculationWarning> {
    let mut warnings = Vec::new();

    if input.number_of_comparisons == 1 {
        warnings.push(CalculationWarning::new(
            "single_comparison",
            "No adjustment is needed when there is only one comparison.",
        ));
    }

    match input.adjustment_method {
        MultiplicityMethod::Bonferroni => warnings.push(CalculationWarning::new(
            "bonferroni_conservative",
            "Bonferroni is conservative when comparisons are positively correlated; it remains valid under any dependence structure.",
        )),
        MultiplicityMethod::Sidak => warnings.push(CalculationWarning::new(
            "sidak_independence",
            "Sidak assumes independent comparisons; positively correlated endpoints may require Bonferroni or another method.",
        )),
        MultiplicityMethod::Dunnett => warnings.push(CalculationWarning::new(
            "dunnett_equal_n",
            "Dunnett assumes each treatment arm is compared with a common control using equal per-group sample sizes (contrast correlation 0.5).",
        )),
        MultiplicityMethod::Holm => warnings.push(CalculationWarning::new(
            "holm_fixed_order",
            "Holm gatekeeping assumes a pre-specified testing order; this alpha applies at the chosen gate after prior gates are passed.",
        )),
    }

    let adjusted_alpha = match input.adjustment_method {
        MultiplicityMethod::Bonferroni => {
            bonferroni_alpha(input.family_wise_alpha, input.number_of_comparisons)
        }
        MultiplicityMethod::Sidak => {
            sidak_alpha(input.family_wise_alpha, input.number_of_comparisons)
        }
        MultiplicityMethod::Dunnett => {
            dunnett_alpha(input.family_wise_alpha, input.number_of_comparisons).unwrap_or(0.0)
        }
        MultiplicityMethod::Holm => holm_alpha(
            input.family_wise_alpha,
            input.number_of_comparisons,
            input.gate_position.unwrap_or(1),
        ),
    };

    if adjusted_alpha < 0.001 {
        warnings.push(CalculationWarning::new(
            "very_small_alpha",
            "Adjusted alpha is very small; sample size requirements may increase substantially.",
        ));
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn bonferroni_matches_manual_reference() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 2,
            adjustment_method: MultiplicityMethod::Bonferroni,
            gate_position: None,
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.025, max_relative = 1e-12);
        assert_relative_eq!(result.alpha_reduction_factor, 0.5, max_relative = 1e-12);
    }

    #[test]
    fn bonferroni_five_comparisons_matches_manual_reference() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 5,
            adjustment_method: MultiplicityMethod::Bonferroni,
            gate_position: None,
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.01, max_relative = 1e-12);
    }

    #[test]
    fn sidak_two_comparisons_matches_manual_reference() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 2,
            adjustment_method: MultiplicityMethod::Sidak,
            gate_position: None,
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.025320566, max_relative = 1e-6);
    }

    #[test]
    fn sidak_five_comparisons_matches_manual_reference() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 5,
            adjustment_method: MultiplicityMethod::Sidak,
            gate_position: None,
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.010206212, max_relative = 1e-6);
    }

    #[test]
    fn single_comparison_returns_family_wise_alpha() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 1,
            adjustment_method: MultiplicityMethod::Bonferroni,
            gate_position: None,
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.05, max_relative = 1e-12);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.code == "single_comparison"));
    }

    #[test]
    fn dunnett_two_arms_matches_reference() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 2,
            adjustment_method: MultiplicityMethod::Dunnett,
            gate_position: None,
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.02695777, epsilon = 1e-4);
        assert!(result.adjusted_alpha > 0.025);
    }

    #[test]
    fn dunnett_three_arms_matches_reference() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 3,
            adjustment_method: MultiplicityMethod::Dunnett,
            gate_position: None,
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.01882430, epsilon = 1e-4);
    }

    #[test]
    fn holm_first_gate_matches_bonferroni() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 5,
            adjustment_method: MultiplicityMethod::Holm,
            gate_position: Some(1),
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.01, max_relative = 1e-12);
        assert_eq!(result.gate_position, Some(1));
    }

    #[test]
    fn holm_third_gate_matches_manual_reference() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 5,
            adjustment_method: MultiplicityMethod::Holm,
            gate_position: Some(3),
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.05 / 3.0, max_relative = 1e-12);
    }

    #[test]
    fn holm_final_gate_returns_family_wise_alpha() {
        let result = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 5,
            adjustment_method: MultiplicityMethod::Holm,
            gate_position: Some(5),
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.05, max_relative = 1e-12);
    }

    #[test]
    fn holm_requires_gate_position() {
        let err = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 3,
            adjustment_method: MultiplicityMethod::Holm,
            gate_position: None,
        })
        .expect_err("invalid");

        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[test]
    fn rejects_gate_position_for_bonferroni() {
        let err = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 3,
            adjustment_method: MultiplicityMethod::Bonferroni,
            gate_position: Some(1),
        })
        .expect_err("invalid");

        assert!(matches!(err, Error::InvalidInput { .. }));
    }

    #[test]
    fn rejects_zero_comparisons() {
        let err = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 0,
            adjustment_method: MultiplicityMethod::Bonferroni,
            gate_position: None,
        })
        .expect_err("invalid");

        assert!(matches!(err, Error::InvalidInput { .. }));
    }
}
