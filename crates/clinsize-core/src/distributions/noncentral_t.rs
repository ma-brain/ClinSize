//! Noncentral Student's t distribution via R's `pnt` (`r_mathlib`).

use r_mathlib::non_central_t_cdf;

use super::student_t;

/// Cumulative distribution function of the noncentral t distribution.
pub fn cdf(t: f64, df: f64, ncp: f64) -> f64 {
    if ncp == 0.0 {
        return student_t::cdf(t, df);
    }
    non_central_t_cdf(t, df, ncp, true, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn matches_r_pt_when_ncp_is_zero() {
        assert_relative_eq!(cdf(1.96, 30.0, 0.0), 0.9703288436, epsilon = 1e-6);
    }

    #[test]
    fn matches_r_pt_with_nonzero_ncp() {
        assert_relative_eq!(cdf(0.0, 30.0, 1.0), 0.1586552539, epsilon = 1e-6);
        assert_relative_eq!(cdf(1.96, 30.0, 1.0), 0.8199262218, epsilon = 1e-6);
        assert_relative_eq!(cdf(3.0, 30.0, 2.0), 0.8184879500, epsilon = 1e-6);
    }
}
