//! Shared survival-endpoint helpers.

use crate::distributions::normal;
use crate::numerics::{find_minimum_integer, MAX_SAMPLE_SIZE_SEARCH};
use crate::types::Alternative;

/// Control-group proportion from treatment:control allocation ratio.
pub fn control_proportion(allocation_ratio: f64) -> f64 {
    1.0 / (1.0 + allocation_ratio)
}

fn tail_multiplier(alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => 2.0,
        Alternative::Greater | Alternative::Less => 1.0,
    }
}

fn log_hazard_ratio_magnitude(hazard_ratio: f64) -> f64 {
    hazard_ratio.ln().abs()
}

/// Schoenfeld (1981) total events required for a two-arm log-rank test.
pub fn schoenfeld_events(
    hazard_ratio: f64,
    alpha: f64,
    power: f64,
    allocation_ratio: f64,
    alternative: Alternative,
) -> f64 {
    let p = control_proportion(allocation_ratio);
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let z_beta = normal::quantile(power);
    let log_hr = log_hazard_ratio_magnitude(hazard_ratio);
    (z_alpha + z_beta).powi(2) / (p * (1.0 - p) * log_hr.powi(2))
}

/// Achieved power for a fixed total event count under Schoenfeld's approximation.
pub fn schoenfeld_power(
    total_events: u32,
    hazard_ratio: f64,
    alpha: f64,
    allocation_ratio: f64,
    alternative: Alternative,
) -> f64 {
    let p = control_proportion(allocation_ratio);
    let tside = tail_multiplier(alternative);
    let z_alpha = normal::upper_tail_critical(alpha / tside);
    let log_hr = log_hazard_ratio_magnitude(hazard_ratio);
    let z = (total_events as f64 * p * (1.0 - p)).sqrt() * log_hr - z_alpha;
    normal::cdf(z)
}

/// Split total events across arms using the control-group proportion.
pub fn split_events(total_events: u32, allocation_ratio: f64) -> (u32, u32) {
    let p = control_proportion(allocation_ratio);
    let events_control = (total_events as f64 * p).round() as u32;
    let events_treatment = total_events.saturating_sub(events_control);
    (events_control, events_treatment)
}

/// Event probability under uniform accrual and exponential failure/dropout hazards.
///
/// Uses the Lachin and Foulkes (1986) competing-risk integral with study duration
/// `accrual_duration + minimum_follow_up`.
pub fn event_probability_uniform_accrual(
    event_hazard: f64,
    dropout_hazard: f64,
    accrual_duration: f64,
    minimum_follow_up: f64,
) -> f64 {
    let combined = event_hazard + dropout_hazard;
    let accrual_term = (1.0 - (-combined * accrual_duration).exp()) / (combined * accrual_duration);
    event_hazard / combined * (1.0 - (-combined * minimum_follow_up).exp() * accrual_term)
}

/// Expected events for an integer total enrollment across both arms.
pub fn expected_events_from_enrollment(
    total_enrollment: u32,
    probability_event_control: f64,
    probability_event_treatment: f64,
    allocation_ratio: f64,
) -> f64 {
    let (n_control, n_treatment) = split_enrollment(total_enrollment, allocation_ratio);
    n_control as f64 * probability_event_control + n_treatment as f64 * probability_event_treatment
}

/// Split total enrollment across arms using the control-group proportion.
pub fn split_enrollment(total_enrollment: u32, allocation_ratio: f64) -> (u32, u32) {
    let p = control_proportion(allocation_ratio);
    let n_control = (total_enrollment as f64 * p).round() as u32;
    let n_treatment = total_enrollment.saturating_sub(n_control);
    (n_control, n_treatment)
}

/// Minimum total enrollment needed to observe at least `required_events`.
pub fn solve_enrolled_subjects(
    required_events: u32,
    probability_event_control: f64,
    probability_event_treatment: f64,
    allocation_ratio: f64,
) -> u32 {
    let continuous = required_events as f64 * (1.0 + allocation_ratio)
        / (probability_event_control + allocation_ratio * probability_event_treatment);
    let min_enrollment = continuous.ceil().max(2.0) as u32;

    find_minimum_integer(min_enrollment, MAX_SAMPLE_SIZE_SEARCH, |total| {
        expected_events_from_enrollment(
            total,
            probability_event_control,
            probability_event_treatment,
            allocation_ratio,
        ) >= required_events as f64
    })
    .unwrap_or(min_enrollment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn schoenfeld_events_matches_gsdesign_two_sided() {
        let events = schoenfeld_events(0.5, 0.05, 0.8, 1.0, Alternative::TwoSided);
        assert_relative_eq!(events, 65.345_66, epsilon = 1e-4);
    }

    #[test]
    fn schoenfeld_events_matches_gsdesign_one_sided() {
        let events = schoenfeld_events(0.5, 0.05, 0.8, 1.0, Alternative::Greater);
        assert_relative_eq!(events, 51.472_73, epsilon = 1e-4);
    }

    #[test]
    fn schoenfeld_events_matches_gsdesign_unequal_allocation() {
        let events = schoenfeld_events(0.5, 0.05, 0.8, 2.0, Alternative::TwoSided);
        assert_relative_eq!(events, 73.513_87, epsilon = 1e-4);
    }

    #[test]
    fn schoenfeld_power_matches_gsdesign() {
        let power = schoenfeld_power(66, 0.5, 0.05, 1.0, Alternative::TwoSided);
        assert_relative_eq!(power, 0.803_894_1, epsilon = 1e-5);
    }

    #[test]
    fn event_probability_matches_gsdesign_without_dropout() {
        let lambda_c = 2.0_f64.ln() / 6.0;
        let probability = event_probability_uniform_accrual(lambda_c, 0.0, 12.0, 18.0);
        assert_relative_eq!(probability, 0.932_373_7, epsilon = 1e-5);
    }

    #[test]
    fn event_probability_matches_gsdesign_with_dropout() {
        let lambda_c = 2.0_f64.ln() / 6.0;
        let probability = event_probability_uniform_accrual(lambda_c, 0.001, 12.0, 18.0);
        assert_relative_eq!(probability, 0.925_873_7, epsilon = 1e-5);
    }

    #[test]
    fn enrolled_subjects_matches_gsdesign_schoenfeld() {
        let lambda_c = 2.0_f64.ln() / 6.0;
        let p_control = event_probability_uniform_accrual(lambda_c, 0.0, 12.0, 18.0);
        let p_treatment = event_probability_uniform_accrual(lambda_c * 0.5, 0.0, 12.0, 18.0);
        let total = solve_enrolled_subjects(66, p_control, p_treatment, 1.0);
        assert_eq!(total, 79);
    }
}
