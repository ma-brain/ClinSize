//! Equicorrelated multivariate normal probabilities for Dunnett adjustment.
//!
//! Uses the one-dimensional integral representation for equal correlation ρ
//! between treatment-vs-control contrasts (equal per-group sample size),
//! evaluated by 40-point Gauss-Hermite quadrature for near-exact accuracy.

use super::normal;

/// Dunnett correlation between treatment-vs-control contrasts with equal group sizes.
pub const DUNNETT_EQUAL_N_CORRELATION: f64 = 0.5;

/// 40-point Gauss-Hermite quadrature nodes for ∫ e^{-x²/2} f(x) dx (probabilist's convention).
#[allow(clippy::excessive_precision)]
const GH_NODES: [f64; 40] = [
    -1.1453377841548731e+01,
    -1.0481560534674268e+01,
    -9.6735563669340312e+00,
    -8.9495045438555536e+00,
    -8.2789406236594765e+00,
    -7.6461637645414608e+00,
    -7.0417384064538293e+00,
    -6.4594233775837671e+00,
    -5.8948056753720177e+00,
    -5.3446054457200862e+00,
    -4.8062871920938735e+00,
    -4.2778261563627495e+00,
    -3.7575597761689861e+00,
    -3.2440887329998702e+00,
    -2.7362083404654309e+00,
    -2.2328592186348719e+00,
    -1.7330905906317213e+00,
    -1.2360320047991582e+00,
    -7.4087072528593045e-01,
    -2.4683289602272435e-01,
    2.4683289602272435e-01,
    7.4087072528593045e-01,
    1.2360320047991582e+00,
    1.7330905906317213e+00,
    2.2328592186348719e+00,
    2.7362083404654309e+00,
    3.2440887329998702e+00,
    3.7575597761689861e+00,
    4.2778261563627495e+00,
    4.8062871920938735e+00,
    5.3446054457200862e+00,
    5.8948056753720177e+00,
    6.4594233775837671e+00,
    7.0417384064538293e+00,
    7.6461637645414608e+00,
    8.2789406236594765e+00,
    8.9495045438555536e+00,
    9.6735563669340312e+00,
    1.0481560534674268e+01,
    1.1453377841548731e+01,
];

/// 40-point Gauss-Hermite quadrature weights. Σwᵢ = √(2π).
#[allow(clippy::excessive_precision)]
const GH_WEIGHTS: [f64; 40] = [
    3.6642891608240561e-29,
    1.2083121235859404e-24,
    3.6311253600243057e-21,
    2.8131267653502130e-18,
    8.4971024877014071e-16,
    1.2453151178157192e-13,
    1.0120859031618208e-11,
    4.9859807389335530e-10,
    1.5856672755225378e-08,
    3.4098727770995936e-07,
    5.1358242451007391e-06,
    5.5676654302285170e-05,
    4.4385600999989806e-04,
    2.6466961983556696e-03,
    1.1965502570998531e-02,
    4.1454227729205158e-02,
    1.1097985191673822e-01,
    2.3105241960643724e-01,
    3.7579649771068080e-01,
    4.7891391574174258e-01,
    4.7891391574174258e-01,
    3.7579649771068080e-01,
    2.3105241960643724e-01,
    1.1097985191673822e-01,
    4.1454227729205158e-02,
    1.1965502570998531e-02,
    2.6466961983556696e-03,
    4.4385600999989806e-04,
    5.5676654302285170e-05,
    5.1358242451007391e-06,
    3.4098727770995936e-07,
    1.5856672755225378e-08,
    4.9859807389335530e-10,
    1.0120859031618208e-11,
    1.2453151178157192e-13,
    8.4971024877014071e-16,
    2.8131267653502130e-18,
    3.6311253600243057e-21,
    1.2083121235859404e-24,
    3.6642891608240561e-29,
];

const INV_SQRT_2PI: f64 = 0.3989422804014327;

/// Probability that all coordinates fall in the symmetric interval `[-c, c]`
/// for a `dimensions`-variate standard normal with equal pairwise correlation.
///
/// Uses the conditional decomposition into a 1-D integral weighted by `φ(z)`,
/// evaluated by 40-point Gauss-Hermite quadrature (accurate to ~1e-12).
pub fn symmetric_rectangle_probability(c: f64, dimensions: u32, correlation: f64) -> f64 {
    if c <= 0.0 {
        return 0.0;
    }
    if dimensions == 0 {
        return 1.0;
    }
    if dimensions == 1 {
        let upper = normal::cdf(c);
        let lower = normal::cdf(-c);
        return upper - lower;
    }

    let rho = correlation.clamp(0.0, 0.999_999);
    let scale = (1.0 - rho).sqrt();
    let sqrt_rho = rho.sqrt();
    let power = dimensions as i32;

    // Gauss-Hermite (probabilist's): ∫ φ(z) f(z) dz = (1/√(2π)) Σ wᵢ f(zᵢ),
    // where zᵢ, wᵢ are the nodes/weights for ∫ e^{-z²/2} g(z) dz.
    let mut sum = 0.0;
    for (node, weight) in GH_NODES.iter().zip(GH_WEIGHTS.iter()) {
        sum += weight * conditional_margin(*node, c, scale, sqrt_rho, power);
    }

    (sum * INV_SQRT_2PI).clamp(0.0, 1.0)
}

/// The integrand without the φ(z) weight (absorbed by Gauss-Hermite):
/// `[Φ((c+z√ρ)/scale) − Φ((−c+z√ρ)/scale)]^dimensions`.
fn conditional_margin(z: f64, c: f64, scale: f64, sqrt_rho: f64, power: i32) -> f64 {
    let upper = normal::cdf((c + z * sqrt_rho) / scale);
    let lower = normal::cdf((-c + z * sqrt_rho) / scale);
    let margin = (upper - lower).clamp(0.0, 1.0);
    margin.powi(power)
}

/// Two-sided Dunnett critical value controlling FWER at `1 - family_wise_alpha`.
pub fn dunnett_two_sided_critical_value(
    treatment_arms: u32,
    family_wise_alpha: f64,
) -> Option<f64> {
    if treatment_arms == 0 {
        return None;
    }
    if treatment_arms == 1 {
        return Some(normal::quantile(1.0 - family_wise_alpha / 2.0));
    }

    let target = 1.0 - family_wise_alpha;
    let mut lo = 0.0;
    let mut hi = 1.0;

    while symmetric_rectangle_probability(hi, treatment_arms, DUNNETT_EQUAL_N_CORRELATION) < target
    {
        hi *= 2.0;
        if hi > 20.0 {
            return None;
        }
    }

    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if symmetric_rectangle_probability(mid, treatment_arms, DUNNETT_EQUAL_N_CORRELATION)
            > target
        {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    Some(0.5 * (lo + hi))
}

/// Equivalent per-comparison two-sided alpha for Dunnett (many arms vs control).
pub fn dunnett_two_sided_adjusted_alpha(
    treatment_arms: u32,
    family_wise_alpha: f64,
) -> Option<f64> {
    let critical = dunnett_two_sided_critical_value(treatment_arms, family_wise_alpha)?;
    Some(2.0 * normal::cdf(-critical))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn single_dimension_matches_normal_interval() {
        let prob = symmetric_rectangle_probability(1.959964, 1, DUNNETT_EQUAL_N_CORRELATION);
        assert_relative_eq!(prob, 0.95, epsilon = 1e-8);
    }

    #[test]
    fn dunnett_two_arms_matches_r_mvtnorm_reference() {
        let alpha = dunnett_two_sided_adjusted_alpha(2, 0.05).expect("alpha");
        assert_relative_eq!(alpha, 0.026957839, epsilon = 1e-8);
    }

    #[test]
    fn dunnett_three_arms_matches_r_mvtnorm_reference() {
        let alpha = dunnett_two_sided_adjusted_alpha(3, 0.05).expect("alpha");
        assert_relative_eq!(alpha, 0.018825394, epsilon = 1e-8);
    }

    #[test]
    fn dunnett_five_arms_matches_r_mvtnorm_reference() {
        let alpha = dunnett_two_sided_adjusted_alpha(5, 0.05).expect("alpha");
        assert_relative_eq!(alpha, 0.012023187, epsilon = 1e-8);
    }

    #[test]
    fn single_arm_returns_family_wise_alpha() {
        let alpha = dunnett_two_sided_adjusted_alpha(1, 0.05).expect("alpha");
        assert_relative_eq!(alpha, 0.05, epsilon = 1e-6);
    }
}
