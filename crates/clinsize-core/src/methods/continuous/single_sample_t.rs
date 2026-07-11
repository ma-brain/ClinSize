//! Shared single-sample (one-sample and paired) t-test power calculations.

use crate::distributions::{noncentral_t, student_t};
use crate::error::{Error, Result};
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::Alternative;

pub fn noncentrality_parameter(n: u32, mean_difference: f64, standard_deviation: f64) -> f64 {
    mean_difference * (n as f64).sqrt() / standard_deviation
}

pub fn achieved_power(
    n: u32,
    mean_difference: f64,
    standard_deviation: f64,
    alpha: f64,
    alternative: Alternative,
) -> f64 {
    let df = (n - 1) as f64;
    let ncp = noncentrality_parameter(n, mean_difference, standard_deviation);

    match alternative {
        Alternative::TwoSided => {
            let t_crit = student_t::quantile(1.0 - alpha / 2.0, df);
            1.0 - noncentral_t::cdf(t_crit, df, ncp) + noncentral_t::cdf(-t_crit, df, ncp)
        }
        Alternative::Greater => {
            let t_crit = student_t::quantile(1.0 - alpha, df);
            1.0 - noncentral_t::cdf(t_crit, df, ncp)
        }
        Alternative::Less => {
            let t_crit = student_t::quantile(1.0 - alpha, df);
            noncentral_t::cdf(-t_crit, df, ncp)
        }
    }
}

pub fn solve_sample_size(
    target_power: f64,
    mean_difference: f64,
    standard_deviation: f64,
    alpha: f64,
    alternative: Alternative,
) -> Result<u32> {
    find_minimum_integer(2, MAX_SAMPLE_SIZE_SEARCH, |n| {
        achieved_power(n, mean_difference, standard_deviation, alpha, alternative)
            >= target_power
    })
    .ok_or_else(|| {
        Error::ConvergenceFailure(format!(
            "could not find a sample size up to {MAX_SAMPLE_SIZE_SEARCH} achieving power {target_power}"
        ))
    })
}

pub fn apply_dropout(n: u32, dropout_rate: Option<f64>) -> u32 {
    let Some(rate) = dropout_rate else {
        return n;
    };
    (n as f64 / (1.0 - rate)).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn matches_r_one_sample_power_at_n10() {
        let power = achieved_power(10, 1.0, 1.0, 0.05, Alternative::TwoSided);
        assert_relative_eq!(power, 0.8030962, epsilon = 1e-4);
    }
}
