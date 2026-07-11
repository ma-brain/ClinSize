//! Central Student's t distribution helpers using R's math library (`r_mathlib`).

use r_mathlib::{students_t_cdf, students_t_quantile};

/// Cumulative distribution function of the central t distribution.
pub fn cdf(x: f64, df: f64) -> f64 {
    students_t_cdf(x, df, true, false)
}

/// Quantile function of the central t distribution for probability `p` in (0, 1).
pub fn quantile(p: f64, df: f64) -> f64 {
    students_t_quantile(p, df, true, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn cdf_matches_r_pt_at_zero() {
        assert_relative_eq!(cdf(0.0, 30.0), 0.5, epsilon = 1e-12);
    }

    #[test]
    fn quantile_matches_r_qt() {
        assert_relative_eq!(quantile(0.975, 30.0), 2.042272, epsilon = 1e-5);
    }
}
