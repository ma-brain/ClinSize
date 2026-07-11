//! Shared survival-endpoint helpers.

use crate::distributions::normal;
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
}
