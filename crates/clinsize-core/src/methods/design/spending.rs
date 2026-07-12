//! Alpha spending functions for group sequential designs.

use crate::distributions::normal;

/// Lan-DeMets spending function family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpendingFunction {
    /// Lan-DeMets O'Brien-Fleming approximation.
    ObrienFleming,
    /// Lan-DeMets Pocock approximation.
    Pocock,
}

/// Cumulative Type I error spent by information fraction `t` in `(0, 1]`.
pub fn cumulative_spent_alpha(alpha: f64, t: f64, spending: SpendingFunction) -> f64 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 {
        return 0.0;
    }

    match spending {
        SpendingFunction::ObrienFleming => {
            let z = normal::quantile(1.0 - alpha / 2.0);
            2.0 * (1.0 - normal::cdf(z / t.sqrt()))
        }
        SpendingFunction::Pocock => {
            let e = std::f64::consts::E;
            alpha * (1.0 + (e - 1.0) * t).ln()
        }
    }
}

/// Incremental alpha spent at each equally spaced look.
pub fn incremental_spends(
    alpha: f64,
    number_of_looks: u32,
    spending: SpendingFunction,
) -> Vec<f64> {
    let mut spends = Vec::with_capacity(number_of_looks as usize);
    let mut previous = 0.0;

    for look in 1..=number_of_looks {
        let t = f64::from(look) / f64::from(number_of_looks);
        let cumulative = cumulative_spent_alpha(alpha, t, spending);
        spends.push(cumulative - previous);
        previous = cumulative;
    }

    spends
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn obrien_fleming_matches_gsdesign_incremental_k3() {
        let spends = incremental_spends(0.05, 3, SpendingFunction::ObrienFleming);
        assert_relative_eq!(spends[0], 0.0006868949, epsilon = 1e-6);
        assert_relative_eq!(spends[1], 0.0156877716, epsilon = 1e-6);
        assert_relative_eq!(spends[2], 0.0336253335, epsilon = 1e-6);
        assert_relative_eq!(spends.iter().sum::<f64>(), 0.05, epsilon = 1e-6);
    }

    #[test]
    fn pocock_matches_gsdesign_incremental_k3() {
        let spends = incremental_spends(0.05, 3, SpendingFunction::Pocock);
        assert_relative_eq!(spends[0], 0.02264162, epsilon = 1e-5);
        assert_relative_eq!(spends[1], 0.01552750, epsilon = 1e-5);
        assert_relative_eq!(spends[2], 0.01183087, epsilon = 1e-5);
        assert_relative_eq!(spends.iter().sum::<f64>(), 0.05, epsilon = 1e-5);
    }

    #[test]
    fn obrien_fleming_matches_gsdesign_incremental_k3_alpha_025() {
        // gsDesign(k=3, test.type=1, alpha=0.025, sfu=sfLDOF)$upper$spend
        let spends = incremental_spends(0.025, 3, SpendingFunction::ObrienFleming);
        assert_relative_eq!(spends[0], 0.0001035057, epsilon = 1e-8);
        assert_relative_eq!(spends[1], 0.0059448834, epsilon = 1e-7);
        assert_relative_eq!(spends[2], 0.0189516109, epsilon = 1e-7);
        assert_relative_eq!(spends.iter().sum::<f64>(), 0.025, epsilon = 1e-7);
    }

    #[test]
    fn pocock_matches_gsdesign_incremental_k5_alpha_025() {
        // gsDesign(k=5, test.type=1, alpha=0.025, sfu=sfLDPocock)$upper$spend
        let spends = incremental_spends(0.025, 5, SpendingFunction::Pocock);
        assert_relative_eq!(spends[0], 0.0073848632, epsilon = 1e-7);
        assert_relative_eq!(spends[1], 0.0056935659, epsilon = 1e-7);
        assert_relative_eq!(spends[2], 0.0046343976, epsilon = 1e-7);
        assert_relative_eq!(spends[3], 0.0039081665, epsilon = 1e-7);
        assert_relative_eq!(spends[4], 0.0033790069, epsilon = 1e-7);
    }
}
