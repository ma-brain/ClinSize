//! Equicorrelated multivariate normal probabilities for Dunnett adjustment.
//!
//! Uses the one-dimensional integral representation for equal correlation ρ
//! between treatment-vs-control contrasts (equal per-group sample size).

use r_mathlib::normal_pdf;

use super::normal;

/// Dunnett correlation between treatment-vs-control contrasts with equal group sizes.
pub const DUNNETT_EQUAL_N_CORRELATION: f64 = 0.5;

/// Probability that all coordinates fall in the symmetric interval `[-c, c]`
/// for a `dimensions`-variate standard normal with equal pairwise correlation.
pub fn symmetric_rectangle_probability(c: f64, dimensions: u32, correlation: f64) -> f64 {
    if c <= 0.0 {
        return 0.0;
    }
    if dimensions == 0 {
        return 1.0;
    }
    if dimensions == 1 {
        let upper = normal::cdf(c);
        let lower = normal::cdf(-c);
        return upper - lower;
    }

    let rho = correlation.clamp(0.0, 0.999_999);
    let scale = (1.0 - rho).sqrt();
    let sqrt_rho = rho.sqrt();
    let power = dimensions as i32;

    let mut integral = 0.0;
    let step = 0.02;
    let mut z = -8.0;
    let mut prev_y = integrand(z, c, scale, sqrt_rho, power);

    while z < 8.0 {
        z += step;
        let y = integrand(z, c, scale, sqrt_rho, power);
        integral += 0.5 * step * (prev_y + y);
        prev_y = y;
    }

    integral.clamp(0.0, 1.0)
}

fn integrand(z: f64, c: f64, scale: f64, sqrt_rho: f64, power: i32) -> f64 {
    let upper = normal::cdf((c + z * sqrt_rho) / scale);
    let lower = normal::cdf((-c + z * sqrt_rho) / scale);
    let margin = (upper - lower).clamp(0.0, 1.0);
    normal_pdf(z, 0.0, 1.0, false) * margin.powi(power)
}

/// Two-sided Dunnett critical value controlling FWER at `1 - family_wise_alpha`.
pub fn dunnett_two_sided_critical_value(
    treatment_arms: u32,
    family_wise_alpha: f64,
) -> Option<f64> {
    if treatment_arms == 0 {
        return None;
    }
    if treatment_arms == 1 {
        return Some(normal::quantile(1.0 - family_wise_alpha / 2.0));
    }

    let target = 1.0 - family_wise_alpha;
    let mut lo = 0.0;
    let mut hi = 1.0;

    while symmetric_rectangle_probability(hi, treatment_arms, DUNNETT_EQUAL_N_CORRELATION) < target
    {
        hi *= 2.0;
        if hi > 20.0 {
            return None;
        }
    }

    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if symmetric_rectangle_probability(mid, treatment_arms, DUNNETT_EQUAL_N_CORRELATION)
            > target
        {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    Some(0.5 * (lo + hi))
}

/// Equivalent per-comparison two-sided alpha for Dunnett (many arms vs control).
pub fn dunnett_two_sided_adjusted_alpha(
    treatment_arms: u32,
    family_wise_alpha: f64,
) -> Option<f64> {
    let critical = dunnett_two_sided_critical_value(treatment_arms, family_wise_alpha)?;
    Some(2.0 * normal::cdf(-critical))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn single_dimension_matches_normal_interval() {
        let prob = symmetric_rectangle_probability(1.959964, 1, DUNNETT_EQUAL_N_CORRELATION);
        assert_relative_eq!(prob, 0.95, epsilon = 1e-4);
    }

    #[test]
    fn dunnett_two_arms_matches_r_mvtnorm_reference() {
        let alpha = dunnett_two_sided_adjusted_alpha(2, 0.05).expect("alpha");
        assert_relative_eq!(alpha, 0.02695777, epsilon = 1e-4);
    }

    #[test]
    fn dunnett_three_arms_matches_r_mvtnorm_reference() {
        let alpha = dunnett_two_sided_adjusted_alpha(3, 0.05).expect("alpha");
        assert_relative_eq!(alpha, 0.01882430, epsilon = 1e-4);
    }

    #[test]
    fn dunnett_five_arms_matches_r_mvtnorm_reference() {
        let alpha = dunnett_two_sided_adjusted_alpha(5, 0.05).expect("alpha");
        assert_relative_eq!(alpha, 0.01202302, epsilon = 1e-4);
    }

    #[test]
    fn single_arm_returns_family_wise_alpha() {
        let alpha = dunnett_two_sided_adjusted_alpha(1, 0.05).expect("alpha");
        assert_relative_eq!(alpha, 0.05, epsilon = 1e-6);
    }
}
