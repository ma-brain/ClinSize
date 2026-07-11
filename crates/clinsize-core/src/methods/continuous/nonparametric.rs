//! Shared Noether (1987) nonparametric sample size helpers.

use crate::distributions::normal;
use crate::types::Alternative;

/// Proportion of superiority P(Y > X) under equal-variance normal location-shift.
pub fn probability_superiority_from_cohens_d(cohens_d: f64) -> f64 {
    normal::cdf(cohens_d / std::f64::consts::SQRT_2)
}

/// P(difference > 0) for normally distributed paired differences.
pub fn probability_positive_difference(mean_difference: f64, standard_deviation: f64) -> f64 {
    normal::cdf(mean_difference / standard_deviation)
}

fn tail_multiplier(alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => 2.0,
        Alternative::Greater | Alternative::Less => 1.0,
    }
}

fn superiority_shift(probability: f64, alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => (probability - 0.5).abs(),
        Alternative::Greater => (probability - 0.5).max(0.0),
        Alternative::Less => (0.5 - probability).max(0.0),
    }
}

/// Treatment fraction t = n_treatment / (n_control + n_treatment).
pub fn treatment_fraction(allocation_ratio: f64) -> f64 {
    allocation_ratio / (1.0 + allocation_ratio)
}

/// Noether denominator factor G = 12 * t * (1 - t) * shift^2.
pub fn mann_whitney_denominator_factor(
    probability_superiority: f64,
    allocation_ratio: f64,
    alternative: Alternative,
) -> f64 {
    let shift = superiority_shift(probability_superiority, alternative);
    if shift <= 0.0 {
        return 0.0;
    }
    let t = treatment_fraction(allocation_ratio);
    12.0 * t * (1.0 - t) * shift * shift
}

/// Total sample size from Noether (1987) Mann-Whitney formula.
pub fn mann_whitney_total_n(
    probability_superiority: f64,
    alpha: f64,
    power: f64,
    allocation_ratio: f64,
    alternative: Alternative,
) -> f64 {
    let g = mann_whitney_denominator_factor(probability_superiority, allocation_ratio, alternative);
    if g <= 0.0 {
        return f64::INFINITY;
    }
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let z_beta = normal::quantile(power);
    (z_alpha + z_beta).powi(2) / g
}

/// Achieved power for Mann-Whitney using Noether normal approximation.
pub fn mann_whitney_achieved_power(
    total_n: u32,
    probability_superiority: f64,
    alpha: f64,
    alternative: Alternative,
    allocation_ratio: f64,
) -> f64 {
    let shift = superiority_shift(probability_superiority, alternative);
    if shift <= 0.0 {
        return alpha;
    }
    let t = treatment_fraction(allocation_ratio);
    let z_stat = (total_n as f64 * 12.0 * t * (1.0 - t)).sqrt() * shift;
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

/// Wilcoxon signed-rank sample size from Noether (1987).
pub fn wilcoxon_signed_rank_n(
    probability_positive: f64,
    alpha: f64,
    power: f64,
    alternative: Alternative,
) -> f64 {
    let shift = superiority_shift(probability_positive, alternative);
    if shift <= 0.0 {
        return f64::INFINITY;
    }
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let z_beta = normal::quantile(power);
    (z_alpha + z_beta).powi(2) / (6.0 * shift * shift)
}

/// Achieved power for Wilcoxon signed-rank using Noether normal approximation.
pub fn wilcoxon_signed_rank_achieved_power(
    n_pairs: u32,
    probability_positive: f64,
    alpha: f64,
    alternative: Alternative,
) -> f64 {
    let shift = superiority_shift(probability_positive, alternative);
    if shift <= 0.0 {
        return alpha;
    }
    let z_stat = (6.0 * n_pairs as f64).sqrt() * shift;
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

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn cohens_d_maps_to_probability_superiority() {
        assert_relative_eq!(
            probability_superiority_from_cohens_d(0.0),
            0.5,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            probability_superiority_from_cohens_d(1.0),
            normal::cdf(1.0 / std::f64::consts::SQRT_2),
            epsilon = 1e-12
        );
    }
}
