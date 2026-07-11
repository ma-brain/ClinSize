//! Standard normal distribution helpers using R's math library (`r_mathlib`).

use r_mathlib::{normal_cdf, normal_quantile};

/// CDF of the standard normal at `x`.
pub fn cdf(x: f64) -> f64 {
    normal_cdf(x, 0.0, 1.0, true, false)
}

/// Quantile of the standard normal for probability `p` in (0, 1), lower tail.
pub fn quantile(p: f64) -> f64 {
    normal_quantile(p, 0.0, 1.0, true, false)
}

/// Upper-tail critical value `z` such that `P(Z > z) = alpha`.
pub fn upper_tail_critical(alpha: f64) -> f64 {
    normal_quantile(alpha, 0.0, 1.0, false, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn cdf_matches_r_pnorm_at_zero() {
        assert_relative_eq!(cdf(0.0), 0.5, epsilon = 1e-12);
    }

    #[test]
    fn upper_tail_critical_matches_r_qnorm() {
        assert_relative_eq!(upper_tail_critical(0.025), 1.959964, epsilon = 1e-5);
        assert_relative_eq!(upper_tail_critical(0.05), 1.644854, epsilon = 1e-5);
    }
}
