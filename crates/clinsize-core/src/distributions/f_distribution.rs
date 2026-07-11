//! Central F distribution helpers using R's math library (`r_mathlib`).

use r_mathlib::{f_cdf, f_quantile};

/// Upper-tail critical value `F` such that `P(F > x | df1, df2) = alpha`.
pub fn critical_value(alpha: f64, df1: f64, df2: f64) -> f64 {
    f_quantile(alpha, df1, df2, false, false)
}

/// Cumulative distribution function of the central F distribution.
pub fn cdf(x: f64, df1: f64, df2: f64) -> f64 {
    f_cdf(x, df1, df2, true, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn critical_value_matches_r_qf() {
        // qf(0.05, df1=2, df2=15, lower.tail=FALSE) for groups=3, n=6
        assert_relative_eq!(critical_value(0.05, 2.0, 15.0), 3.68232, epsilon = 1e-4);
    }
}
