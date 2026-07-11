//! Family-wise alpha adjustment for multiple comparisons.

use serde::{Deserialize, Serialize};

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
}

/// Results for a family-wise alpha adjustment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiplicityResult {
    pub adjusted_alpha: f64,
    pub family_wise_alpha: f64,
    pub number_of_comparisons: u32,
    pub adjustment_method: MultiplicityMethod,
    /// Ratio of adjusted alpha to the unadjusted per-comparison alpha if the
    /// family-wise rate were split naively without adjustment logic.
    pub alpha_reduction_factor: f64,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &MultiplicityInput) -> Result<()> {
    validation::validate_alpha(input.family_wise_alpha)?;
    validation::validate_comparison_count(input.number_of_comparisons)
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
    }

    let adjusted_alpha = match input.adjustment_method {
        MultiplicityMethod::Bonferroni => {
            bonferroni_alpha(input.family_wise_alpha, input.number_of_comparisons)
        }
        MultiplicityMethod::Sidak => {
            sidak_alpha(input.family_wise_alpha, input.number_of_comparisons)
        }
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
        })
        .expect("calculate");

        assert_relative_eq!(result.adjusted_alpha, 0.05, max_relative = 1e-12);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.code == "single_comparison"));
    }

    #[test]
    fn rejects_zero_comparisons() {
        let err = calculate(MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 0,
            adjustment_method: MultiplicityMethod::Bonferroni,
        })
        .expect_err("invalid");

        assert!(matches!(err, Error::InvalidInput { .. }));
    }
}
