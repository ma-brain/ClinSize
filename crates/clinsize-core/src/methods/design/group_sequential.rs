//! Group sequential design planning for equally spaced interim looks.

use serde::{Deserialize, Serialize};

use super::sequential_bounds::{power_under_drift, sample_size_inflation, solve_upper_bounds};
use super::spending::{incremental_spends, SpendingFunction};
use crate::distributions::normal;
use crate::error::{Error, Result};
use crate::types::CalculationWarning;
use crate::validation;

/// Inputs for a group sequential design summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSequentialInput {
    /// Two-sided family-wise Type I error rate.
    pub alpha: f64,
    /// Target power for the group sequential design.
    pub target_power: f64,
    /// Total number of looks, including the final analysis.
    pub number_of_looks: u32,
    pub spending_function: SpendingFunction,
}

/// Per-look summary for a group sequential design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSequentialLookResult {
    pub look: u32,
    pub information_fraction: f64,
    pub incremental_alpha_spent: f64,
    pub cumulative_alpha_spent: f64,
    pub upper_z_boundary: f64,
}

/// Results for a group sequential design summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupSequentialResult {
    pub looks: Vec<GroupSequentialLookResult>,
    pub sample_size_inflation_factor: f64,
    pub required_drift: f64,
    pub fixed_design_drift: f64,
    pub achieved_power: f64,
    pub spending_function: SpendingFunction,
    pub warnings: Vec<CalculationWarning>,
}

pub fn validate(input: &GroupSequentialInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_power(input.target_power)?;
    validation::validate_comparison_count(input.number_of_looks)?;
    if input.number_of_looks < 2 {
        return Err(Error::InvalidInput {
            field: "numberOfLooks".into(),
            message: "must be at least 2 for a group sequential design".into(),
        });
    }
    if input.number_of_looks > 10 {
        return Err(Error::InvalidInput {
            field: "numberOfLooks".into(),
            message: "values above 10 are not supported in this release".into(),
        });
    }
    Ok(())
}

/// Plan a group sequential design with equally spaced information fractions.
pub fn calculate(input: GroupSequentialInput) -> Result<GroupSequentialResult> {
    validate(&input)?;

    let timing: Vec<f64> = (1..=input.number_of_looks)
        .map(|look| f64::from(look) / f64::from(input.number_of_looks))
        .collect();
    let incremental =
        incremental_spends(input.alpha, input.number_of_looks, input.spending_function);
    let bounds = solve_upper_bounds(&incremental, &timing)?;
    // gsDesign symmetric two-sided convention (test.type = 4): compare against the
    // one-sided fixed-design drift at the same nominal alpha.
    let fixed_design_drift =
        normal::quantile(1.0 - input.alpha) + normal::quantile(input.target_power);
    let (required_drift, inflation) =
        sample_size_inflation(&bounds, &timing, fixed_design_drift, input.target_power)?;
    let achieved_power = power_under_drift(&bounds, &timing, fixed_design_drift, inflation);

    let mut cumulative = 0.0;
    let looks = incremental
        .iter()
        .zip(bounds.iter())
        .enumerate()
        .map(|(idx, (increment, bound))| {
            cumulative += increment;
            GroupSequentialLookResult {
                look: (idx as u32) + 1,
                information_fraction: timing[idx],
                incremental_alpha_spent: *increment,
                cumulative_alpha_spent: cumulative,
                upper_z_boundary: *bound,
            }
        })
        .collect();

    let warnings = build_warnings(&input);

    Ok(GroupSequentialResult {
        looks,
        sample_size_inflation_factor: inflation,
        required_drift,
        fixed_design_drift,
        achieved_power,
        spending_function: input.spending_function,
        warnings,
    })
}

fn build_warnings(input: &GroupSequentialInput) -> Vec<CalculationWarning> {
    let mut warnings = vec![CalculationWarning::new(
        "equally_spaced_looks",
        "Assumes equally spaced information fractions; custom timing is not yet supported.",
    )];

    match input.spending_function {
        SpendingFunction::ObrienFleming => warnings.push(CalculationWarning::new(
            "obrien_fleming_spending",
            "Uses the Lan-DeMets O'Brien-Fleming spending approximation for two-sided efficacy bounds.",
        )),
        SpendingFunction::Pocock => warnings.push(CalculationWarning::new(
            "pocock_spending",
            "Uses the Lan-DeMets Pocock spending approximation; interim boundaries are nearly constant.",
        )),
    }

    if input.sample_size_inflation_factor_would_be_high(input.number_of_looks) {
        warnings.push(CalculationWarning::new(
            "many_looks",
            "Many interim looks increase maximum sample size; review whether fewer looks are feasible.",
        ));
    }

    warnings
}

trait InflationHeuristic {
    fn sample_size_inflation_factor_would_be_high(&self, number_of_looks: u32) -> bool;
}

impl InflationHeuristic for GroupSequentialInput {
    fn sample_size_inflation_factor_would_be_high(&self, number_of_looks: u32) -> bool {
        number_of_looks >= 6
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn obrien_fleming_k3_matches_gsdesign_summary() {
        let result = calculate(GroupSequentialInput {
            alpha: 0.05,
            target_power: 0.8,
            number_of_looks: 3,
            spending_function: SpendingFunction::ObrienFleming,
        })
        .expect("calculate");

        assert_relative_eq!(
            result.sample_size_inflation_factor,
            1.020305,
            epsilon = 0.02
        );
        assert_relative_eq!(result.looks[0].upper_z_boundary, 3.200102, epsilon = 0.02);
        assert_relative_eq!(result.achieved_power, 0.8, epsilon = 0.02);
    }

    #[test]
    fn pocock_k5_matches_gsdesign_inflation() {
        let result = calculate(GroupSequentialInput {
            alpha: 0.05,
            target_power: 0.8,
            number_of_looks: 5,
            spending_function: SpendingFunction::Pocock,
        })
        .expect("calculate");

        assert_relative_eq!(
            result.sample_size_inflation_factor,
            1.221578,
            epsilon = 0.02
        );
    }

    #[test]
    fn rejects_single_look_design() {
        let err = calculate(GroupSequentialInput {
            alpha: 0.05,
            target_power: 0.8,
            number_of_looks: 1,
            spending_function: SpendingFunction::ObrienFleming,
        })
        .expect_err("invalid");

        assert!(matches!(err, Error::InvalidInput { .. }));
    }
}
