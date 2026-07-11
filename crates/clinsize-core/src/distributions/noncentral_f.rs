//! Noncentral F distribution via R's `pnf` (`r_mathlib`).

use r_mathlib::non_central_f_cdf;

/// Upper-tail probability `P(F > x | df1, df2, ncp)`.
pub fn upper_tail(x: f64, df1: f64, df2: f64, ncp: f64) -> f64 {
    non_central_f_cdf(x, df1, df2, ncp, false, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn matches_r_pnf() {
        // groups=3, n=6, between.var=1, within.var=1
        assert_relative_eq!(
            upper_tail(3.68232, 2.0, 15.0, 12.0),
            0.8053172,
            epsilon = 1e-4
        );
    }
}
