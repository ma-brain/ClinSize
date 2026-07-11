//! Shared binary-endpoint sample size and power helpers.

use crate::distributions::normal;
use crate::types::Alternative;

/// Two-sided tail multiplier used by R `power.prop.test`.
pub fn tail_multiplier(alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => 2.0,
        Alternative::Greater | Alternative::Less => 1.0,
    }
}

/// Achieved power for equal group sizes using R `stats::power.prop.test`
/// (normal approximation, non-strict).
pub fn power_prop_test_equal(
    n_per_group: f64,
    p_control: f64,
    p_treatment: f64,
    alpha: f64,
    alternative: Alternative,
) -> f64 {
    let diff = (p_treatment - p_control).abs();
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let null_variance = (p_control + p_treatment) * (1.0 - (p_control + p_treatment) / 2.0);
    let numerator = n_per_group.sqrt() * diff - z_alpha * null_variance.sqrt();
    let denominator = (p_control * (1.0 - p_control) + p_treatment * (1.0 - p_treatment)).sqrt();
    normal::cdf(numerator / denominator)
}

/// Achieved power for unequal group sizes using pooled normal approximation.
pub fn power_two_proportion(
    n_control: u32,
    n_treatment: u32,
    p_control: f64,
    p_treatment: f64,
    alpha: f64,
    alternative: Alternative,
) -> f64 {
    if n_control == n_treatment {
        return power_prop_test_equal(n_control as f64, p_control, p_treatment, alpha, alternative);
    }

    let diff = (p_treatment - p_control).abs();
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let n_c = n_control as f64;
    let n_t = n_treatment as f64;
    let p_pool = (n_c * p_control + n_t * p_treatment) / (n_c + n_t);
    let se_null = (p_pool * (1.0 - p_pool) * (1.0 / n_c + 1.0 / n_t)).sqrt();
    let se_alt =
        (p_control * (1.0 - p_control) / n_c + p_treatment * (1.0 - p_treatment) / n_t).sqrt();
    normal::cdf((diff - z_alpha * se_null) / se_alt)
}

/// Control-group sample size for unequal allocation (Chow et al. 2003).
pub fn chow_control_n_unequal(
    p_control: f64,
    p_treatment: f64,
    allocation_ratio: f64,
    alpha: f64,
    power: f64,
    alternative: Alternative,
) -> f64 {
    let diff = (p_treatment - p_control).abs();
    let k = allocation_ratio;
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let z_beta = normal::quantile(power);
    let p_bar = (p_treatment + k * p_control) / (1.0 + k);
    let term_alpha = z_alpha * ((1.0 + 1.0 / k) * p_bar * (1.0 - p_bar)).sqrt();
    let term_beta =
        z_beta * (p_treatment * (1.0 - p_treatment) / k + p_control * (1.0 - p_control)).sqrt();
    (term_alpha + term_beta).powi(2) / diff.powi(2)
}

pub fn treatment_n_from_control(n_control: u32, allocation_ratio: f64) -> u32 {
    (n_control as f64 * allocation_ratio).ceil().max(2.0) as u32
}

pub fn odds_ratio(p_control: f64, p_treatment: f64) -> f64 {
    p_treatment * (1.0 - p_control) / (p_control * (1.0 - p_treatment))
}

pub fn risk_ratio(p_control: f64, p_treatment: f64) -> f64 {
    p_treatment / p_control
}

/// Control-group sample size for odds-ratio superiority (Chow et al. 2003).
pub fn chow_control_n_log_odds_ratio(
    p_control: f64,
    p_treatment: f64,
    allocation_ratio: f64,
    alpha: f64,
    power: f64,
    alternative: Alternative,
) -> f64 {
    let or = odds_ratio(p_control, p_treatment);
    chow_control_n_log_effect(
        p_control,
        p_treatment,
        allocation_ratio,
        alpha,
        power,
        alternative,
        or.ln().abs(),
    )
}

/// Control-group sample size for risk-ratio superiority (Chow et al. 2003).
pub fn chow_control_n_log_risk_ratio(
    p_control: f64,
    p_treatment: f64,
    allocation_ratio: f64,
    alpha: f64,
    power: f64,
    alternative: Alternative,
) -> f64 {
    let rr = risk_ratio(p_control, p_treatment);
    chow_control_n_log_effect(
        p_control,
        p_treatment,
        allocation_ratio,
        alpha,
        power,
        alternative,
        rr.ln().abs(),
    )
}

fn chow_control_n_log_effect(
    p_control: f64,
    p_treatment: f64,
    allocation_ratio: f64,
    alpha: f64,
    power: f64,
    alternative: Alternative,
    log_effect: f64,
) -> f64 {
    let k = allocation_ratio;
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let z_beta = normal::quantile(power);
    let variance_term =
        1.0 / (p_control * (1.0 - p_control)) + 1.0 / (k * p_treatment * (1.0 - p_treatment));
    (z_alpha + z_beta).powi(2) * variance_term / log_effect.powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn equal_power_matches_r_power_prop_test() {
        let power = power_prop_test_equal(163.0, 0.3, 0.45, 0.05, Alternative::TwoSided);
        assert_relative_eq!(power, 0.8016161, epsilon = 1e-5);
    }

    #[test]
    fn one_sided_equal_power_matches_r() {
        let power = power_prop_test_equal(163.0, 0.3, 0.45, 0.05, Alternative::Greater);
        assert_relative_eq!(power, 0.8782674, epsilon = 1e-5);
    }

    #[test]
    fn chow_unequal_treatment_n_near_trial_size() {
        let n_control = chow_control_n_unequal(0.3, 0.45, 2.0, 0.05, 0.8, Alternative::TwoSided);
        let n_treatment = treatment_n_from_control(n_control.ceil() as u32, 2.0);
        assert!((n_treatment as f64 - 232.8501).abs() / 232.8501 < 0.03);
    }

    #[test]
    fn odds_ratio_control_n_matches_trial_size() {
        let n = chow_control_n_log_odds_ratio(0.25, 0.4, 1.0, 0.05, 0.8, Alternative::TwoSided);
        assert_relative_eq!(n, 155.1959, epsilon = 1e-3);
    }
}
