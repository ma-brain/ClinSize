//! MMRM (mixed model for repeated measures) sample size and power.
//!
//! ## Formula / algorithm
//!
//! Implements Lu, Luo & Chen (2008) for a two-group parallel design testing
//! the treatment difference `δ` at the final post-baseline visit under an
//! MMRM with visit as a categorical factor.
//!
//! With `K` post-baseline visits, within-subject correlation matrix `R`
//! (built from a single ρ as compound symmetry or AR(1)), and monotone
//! retention probabilities `r_j` (probability a subject is still observed at
//! visit `j`), the information matrix for one group's visit means is
//!
//! `I = Σ_{j=1..K} (r_j − r_{j+1}) × pad(inv(R[1..j, 1..j]))`  with `r_{K+1} = 0`,
//!
//! and the variance factor for the final-visit mean is `φ = inv(I)[K, K]`.
//! With complete data `φ = 1`, recovering the final-visit two-sample z-test.
//! The variance of the treatment contrast is then
//!
//! `Var(δ̂) = σ² × φ × (1/n_control + 1/n_treatment)`
//!
//! where `n` are *randomized* counts: subjects who drop out contribute their
//! observed visits through the retention weights, so no separate enrollment
//! inflation is applied.
//!
//! Per-visit dropout `d` maps to geometric retention `r_j = (1 − d)^j`.
//!
//! ## Intended validation source
//!
//! - Lu, K., Luo, X., & Chen, P.-Y. (2008). Sample size estimation for
//!   repeated measures analysis in randomized clinical trials with missing
//!   data. *The International Journal of Biostatistics*, 4(1).
//! - R `longpower::power.mmrm` (see `validation/continuous/mmrm/`).
//!
//! ## Known limitations
//!
//! - Single-ρ compound symmetry or AR(1) correlation only; unstructured,
//!   Toeplitz, and CSH matrices need more parameters than the UI collects.
//! - Common correlation, retention, and σ across arms.
//! - Normal approximation (z-based); no small-sample df adjustment.

use serde::{Deserialize, Serialize};

use crate::distributions::normal;
use crate::error::{Error, Result};
use crate::methods::binary::shared::treatment_n_from_control;
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::{Alternative, CalculationWarning, CorrelationStructure, SolveMode};
use crate::validation;

const MAX_VISITS: u32 = 20;

/// Inputs for two-group parallel MMRM at the final post-baseline visit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MmrmInput {
    pub solve_mode: SolveMode,
    pub alpha: f64,
    pub power: Option<f64>,
    /// Randomized control-group size when solving for power.
    pub control_n: Option<u32>,
    /// Treatment minus control effect at the primary/final visit (δ).
    pub treatment_effect: f64,
    /// Standard deviation at the final visit (σ).
    pub residual_standard_deviation: f64,
    /// Within-subject correlation model built from the single ρ. Only
    /// `CompoundSymmetry` and `Ar1` are supported.
    pub correlation_structure: CorrelationStructure,
    /// Within-subject correlation parameter (ρ).
    pub correlation: f64,
    /// Number of post-baseline visits (k).
    pub n_post_baseline_visits: u32,
    /// Optional per-visit dropout rate (d_visit) in [0, 1); retention at
    /// visit j is (1 − d_visit)^j (monotone missingness).
    pub per_visit_dropout_rate: Option<f64>,
    pub allocation_ratio: f64,
    pub alternative: Alternative,
}

/// Results for two-group parallel MMRM sizing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MmrmResult {
    /// Randomized control-group size.
    pub n_control: u32,
    /// Randomized treatment-group size.
    pub n_treatment: u32,
    pub total_n: u32,
    pub achieved_power: f64,
    /// Lu-Luo-Chen variance factor φ for the final-visit contrast
    /// (1 with complete data; grows with dropout).
    pub variance_factor: f64,
    /// Retention probability at the final visit.
    pub final_retention: f64,
    /// Cumulative dropout by the final visit (1 − final retention).
    pub cumulative_dropout: f64,
    pub warnings: Vec<CalculationWarning>,
}

fn correlation_entry(structure: CorrelationStructure, rho: f64, i: usize, j: usize) -> f64 {
    if i == j {
        return 1.0;
    }
    match structure {
        CorrelationStructure::Ar1 => rho.powi((i as i32 - j as i32).abs()),
        _ => rho,
    }
}

/// Invert a small symmetric positive-definite matrix by Gauss-Jordan
/// elimination with partial pivoting. Returns `None` when singular.
fn invert_matrix(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = matrix.len();
    let mut work: Vec<Vec<f64>> = matrix
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let mut extended = row.clone();
            extended.extend((0..n).map(|j| if i == j { 1.0 } else { 0.0 }));
            extended
        })
        .collect();

    for col in 0..n {
        let pivot_row = (col..n).max_by(|&a, &b| {
            work[a][col]
                .abs()
                .partial_cmp(&work[b][col].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
        if work[pivot_row][col].abs() < 1e-12 {
            return None;
        }
        work.swap(col, pivot_row);

        let pivot = work[col][col];
        for value in work[col].iter_mut() {
            *value /= pivot;
        }
        let pivot_values = work[col].clone();
        for (row, row_values) in work.iter_mut().enumerate() {
            if row == col {
                continue;
            }
            let factor = row_values[col];
            if factor == 0.0 {
                continue;
            }
            for (value, pivot_value) in row_values.iter_mut().zip(&pivot_values) {
                *value -= factor * pivot_value;
            }
        }
    }

    Some(work.into_iter().map(|mut row| row.split_off(n)).collect())
}

/// Lu-Luo-Chen variance factor φ for the final-visit mean of one group.
pub fn llc_variance_factor(
    structure: CorrelationStructure,
    rho: f64,
    n_visits: u32,
    per_visit_dropout_rate: f64,
) -> Result<f64> {
    let k = n_visits as usize;
    let retention: Vec<f64> = (1..=k)
        .map(|j| (1.0 - per_visit_dropout_rate).powi(j as i32))
        .collect();

    // Information matrix I = Σ_j (r_j − r_{j+1}) × pad(inv(R[1..j,1..j])).
    let mut information = vec![vec![0.0; k]; k];
    for j in 1..=k {
        let weight = retention[j - 1] - if j < k { retention[j] } else { 0.0 };
        if weight <= 0.0 {
            continue;
        }
        let leading: Vec<Vec<f64>> = (0..j)
            .map(|row| {
                (0..j)
                    .map(|col| correlation_entry(structure, rho, row, col))
                    .collect()
            })
            .collect();
        let inverse = invert_matrix(&leading).ok_or_else(|| {
            Error::ConvergenceFailure(
                "within-subject correlation matrix is singular; check ρ".into(),
            )
        })?;
        for (row, inverse_row) in inverse.iter().enumerate() {
            for (col, value) in inverse_row.iter().enumerate() {
                information[row][col] += weight * value;
            }
        }
    }

    let covariance = invert_matrix(&information).ok_or_else(|| {
        Error::ConvergenceFailure(
            "MMRM information matrix is singular; check ρ and dropout inputs".into(),
        )
    })?;
    Ok(covariance[k - 1][k - 1])
}

/// Achieved power for randomized group sizes under the LLC variance.
pub fn achieved_power(
    n_control: u32,
    n_treatment: u32,
    treatment_effect: f64,
    residual_standard_deviation: f64,
    variance_factor: f64,
    alpha: f64,
    alternative: Alternative,
) -> f64 {
    let variance_diff = residual_standard_deviation.powi(2)
        * variance_factor
        * (1.0 / n_control as f64 + 1.0 / n_treatment as f64);
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

pub fn validate(input: &MmrmInput) -> Result<()> {
    validation::validate_alpha(input.alpha)?;
    validation::validate_positive(
        "residualStandardDeviation",
        input.residual_standard_deviation,
    )?;
    validation::validate_positive("allocationRatio", input.allocation_ratio)?;
    validation::validate_correlation(input.correlation)?;

    match input.correlation_structure {
        CorrelationStructure::CompoundSymmetry | CorrelationStructure::Ar1 => {}
        other => {
            return Err(Error::InvalidInput {
                field: "correlationStructure".into(),
                message: format!(
                    "{other:?} requires more correlation parameters than a single ρ; \
                     choose compound symmetry or AR(1)"
                ),
            });
        }
    }

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
    if input.n_post_baseline_visits > MAX_VISITS {
        return Err(Error::InvalidInput {
            field: "nPostBaselineVisits".into(),
            message: format!("must be at most {MAX_VISITS}"),
        });
    }

    // Compound symmetry is positive definite only for ρ > −1/(k−1).
    if matches!(
        input.correlation_structure,
        CorrelationStructure::CompoundSymmetry
    ) && input.n_post_baseline_visits > 1
    {
        let lower_bound = -1.0 / (input.n_post_baseline_visits as f64 - 1.0);
        if input.correlation <= lower_bound {
            return Err(Error::InvalidInput {
                field: "correlation".into(),
                message: format!(
                    "compound symmetry with {} visits requires ρ > {lower_bound:.4}",
                    input.n_post_baseline_visits
                ),
            });
        }
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
                "detectable effect solve mode is not implemented for MMRM".into(),
            ));
        }
    }

    Ok(())
}

pub fn calculate(input: MmrmInput) -> Result<MmrmResult> {
    validate(&input)?;

    let dropout = input.per_visit_dropout_rate.unwrap_or(0.0);
    let variance_factor = llc_variance_factor(
        input.correlation_structure,
        input.correlation,
        input.n_post_baseline_visits,
        dropout,
    )?;
    let final_retention = (1.0 - dropout).powi(input.n_post_baseline_visits as i32);

    let mut warnings = vec![
        CalculationWarning::new(
            "mmrm_lu_luo_chen",
            "Uses the Lu, Luo & Chen (2008) MMRM variance for the final-visit treatment contrast under monotone missingness; matches R longpower::power.mmrm.",
        ),
        CalculationWarning::new(
            "mmrm_normal_approximation",
            "Sample size uses a normal approximation (z-scores); achieved power is recomputed at the rounded integer allocation.",
        ),
        CalculationWarning::new(
            "mmrm_final_visit_effect",
            "Assumes the treatment effect δ applies at the final post-baseline visit with equal σ, correlation, and retention across arms.",
        ),
        CalculationWarning::new(
            "single_rho_structure",
            match input.correlation_structure {
                CorrelationStructure::Ar1 => "Within-subject correlation is AR(1): corr(visit i, visit j) = ρ^|i−j|.",
                _ => "Within-subject correlation is compound symmetry: equal correlation ρ between all visit pairs.",
            },
        ),
    ];

    if input.per_visit_dropout_rate.is_some() {
        warnings.push(CalculationWarning::new(
            "monotone_dropout",
            "Reported sample sizes are randomized counts; dropouts contribute their observed visits through geometric retention (1 − d)^j, so no separate enrollment inflation applies.",
        ));
    }

    let (n_control, n_treatment, achieved) = match input.solve_mode {
        SolveMode::SampleSize => {
            let target_power = input.power.expect("validated");
            let n_control = solve_control_n(&input, variance_factor, target_power)?;
            let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
            let power = achieved_power(
                n_control,
                n_treatment,
                input.treatment_effect,
                input.residual_standard_deviation,
                variance_factor,
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
                input.residual_standard_deviation,
                variance_factor,
                input.alpha,
                input.alternative,
            );
            (n_control, n_treatment, power)
        }
        SolveMode::DetectableEffect => unreachable!("validated"),
    };

    Ok(MmrmResult {
        n_control,
        n_treatment,
        total_n: n_control + n_treatment,
        achieved_power: achieved,
        variance_factor,
        final_retention,
        cumulative_dropout: 1.0 - final_retention,
        warnings,
    })
}

fn solve_control_n(input: &MmrmInput, variance_factor: f64, target_power: f64) -> Result<u32> {
    find_minimum_integer(2, MAX_SAMPLE_SIZE_SEARCH, |n_control| {
        let n_treatment = treatment_n_from_control(n_control, input.allocation_ratio);
        let power = achieved_power(
            n_control,
            n_treatment,
            input.treatment_effect,
            input.residual_standard_deviation,
            variance_factor,
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

    // Reference values from R longpower::power.mmrm (Lu, Luo & Chen 2008):
    //   cs  <- function(k, rho) { m <- matrix(rho, k, k); diag(m) <- 1; m }
    //   ar1 <- function(k, rho) outer(1:k, 1:k, function(i, j) rho^abs(i - j))
    //   ret <- function(k, d) (1 - d)^(1:k)
    //   power.mmrm(Ra, ra, sigmaa, lambda, delta, sig.level, power, alternative)

    fn reference_input() -> MmrmInput {
        MmrmInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            treatment_effect: 2.0,
            residual_standard_deviation: 2.0,
            correlation_structure: CorrelationStructure::CompoundSymmetry,
            correlation: 0.5,
            n_post_baseline_visits: 3,
            per_visit_dropout_rate: Some(0.05),
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
        }
    }

    #[test]
    fn complete_data_variance_factor_is_one() {
        let phi =
            llc_variance_factor(CorrelationStructure::CompoundSymmetry, 0.5, 3, 0.0).expect("phi");
        assert_relative_eq!(phi, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn variance_factor_matches_longpower_cs_dropout() {
        // phi for CS rho=0.5, k=3, 5%/visit dropout = 1.13306118
        let phi =
            llc_variance_factor(CorrelationStructure::CompoundSymmetry, 0.5, 3, 0.05).expect("phi");
        assert_relative_eq!(phi, 1.13306118, epsilon = 1e-7);
    }

    #[test]
    fn variance_factor_matches_longpower_ar1_dropout() {
        // phi for AR1 rho=0.6, k=4, 10%/visit dropout = 1.44575044
        let phi = llc_variance_factor(CorrelationStructure::Ar1, 0.6, 4, 0.1).expect("phi");
        assert_relative_eq!(phi, 1.44575044, epsilon = 1e-7);
    }

    #[test]
    fn sample_size_matches_longpower_cs_dropout() {
        // longpower: n per arm = 17.786522 -> smallest integer allocation is 18;
        // power.mmrm(N = 36, ...) = 0.8046597
        let result = calculate(reference_input()).expect("calculate");

        assert_eq!(result.n_control, 18);
        assert_eq!(result.n_treatment, 18);
        assert_eq!(result.total_n, 36);
        assert_relative_eq!(result.achieved_power, 0.8046597, epsilon = 1e-5);
        assert_relative_eq!(result.variance_factor, 1.13306118, epsilon = 1e-7);
        assert_relative_eq!(result.final_retention, 0.857375, epsilon = 1e-9);
        assert_relative_eq!(result.cumulative_dropout, 0.142625, epsilon = 1e-9);
    }

    #[test]
    fn sample_size_matches_longpower_complete_data() {
        // longpower: n per arm = 15.697759 -> 16; power.mmrm(N = 32, ...) = 0.8074296
        let mut input = reference_input();
        input.per_visit_dropout_rate = None;
        let result = calculate(input).expect("calculate");

        assert_eq!(result.n_control, 16);
        assert_eq!(result.n_treatment, 16);
        assert_relative_eq!(result.achieved_power, 0.8074296, epsilon = 1e-5);
        assert_relative_eq!(result.variance_factor, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn sample_size_matches_longpower_ar1_one_sided() {
        // longpower: AR1 rho=0.6, k=4, 10%/visit, sigma=3, delta=1.5, one-sided
        // alpha=0.025, power=0.9 -> n per arm = 121.528892 -> 122;
        // power.mmrm(N = 244, ...) = 0.9010971
        let input = MmrmInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.025,
            power: Some(0.9),
            control_n: None,
            treatment_effect: 1.5,
            residual_standard_deviation: 3.0,
            correlation_structure: CorrelationStructure::Ar1,
            correlation: 0.6,
            n_post_baseline_visits: 4,
            per_visit_dropout_rate: Some(0.1),
            allocation_ratio: 1.0,
            alternative: Alternative::Greater,
        };
        let result = calculate(input).expect("calculate");

        assert_eq!(result.n_control, 122);
        assert_eq!(result.n_treatment, 122);
        assert_relative_eq!(result.achieved_power, 0.9010971, epsilon = 1e-5);
    }

    #[test]
    fn unequal_allocation_matches_longpower_lambda_two() {
        // longpower: CS rho=0.3, k=5, 8%/visit, sigma=1.2, delta=0.5, two-sided
        // alpha=0.05, power=0.9, lambda=2 -> n_treatment=263.76, n_control=131.88;
        // power.mmrm(N = 396, lambda = 2, ...) = 0.9002562
        let input = MmrmInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.9),
            control_n: None,
            treatment_effect: 0.5,
            residual_standard_deviation: 1.2,
            correlation_structure: CorrelationStructure::CompoundSymmetry,
            correlation: 0.3,
            n_post_baseline_visits: 5,
            per_visit_dropout_rate: Some(0.08),
            allocation_ratio: 2.0,
            alternative: Alternative::TwoSided,
        };
        let result = calculate(input).expect("calculate");

        assert_eq!(result.n_control, 132);
        assert_eq!(result.n_treatment, 264);
        assert_relative_eq!(result.achieved_power, 0.9002562, epsilon = 1e-4);
        assert_relative_eq!(result.variance_factor, 1.45268834, epsilon = 1e-7);
    }

    #[test]
    fn power_mode_matches_longpower_at_fixed_n() {
        // power.mmrm(N = 74, AR1 rho=0.6, k=4, 10%/visit, sigma=3, delta=1.5,
        // one-sided alpha=0.025) = 0.4319623
        let input = MmrmInput {
            solve_mode: SolveMode::Power,
            alpha: 0.025,
            power: None,
            control_n: Some(37),
            treatment_effect: 1.5,
            residual_standard_deviation: 3.0,
            correlation_structure: CorrelationStructure::Ar1,
            correlation: 0.6,
            n_post_baseline_visits: 4,
            per_visit_dropout_rate: Some(0.1),
            allocation_ratio: 1.0,
            alternative: Alternative::Greater,
        };
        let result = calculate(input).expect("calculate");
        assert_relative_eq!(result.achieved_power, 0.4319623, epsilon = 1e-5);
    }

    #[test]
    fn mmrm_never_beats_final_visit_ttest_with_complete_data() {
        // Regression guard for the pre-2026-07 formula, which produced n = 4
        // per arm here while the final-visit t-test needs 17.
        let mut input = reference_input();
        input.per_visit_dropout_rate = None;
        let result = calculate(input).expect("calculate");
        // Normal-approximation two-sample n per arm at d = 1 is 15.7 -> 16.
        assert!(result.n_control >= 16);
    }

    #[test]
    fn rejects_unstructured_correlation() {
        let mut input = reference_input();
        input.correlation_structure = CorrelationStructure::Unstructured;
        assert!(validate(&input).is_err());
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
    fn rejects_compound_symmetry_below_positive_definite_bound() {
        let mut input = reference_input();
        input.correlation = -0.6;
        assert!(validate(&input).is_err());
    }
}
