//! Boundary solving and sample size inflation for group sequential designs.

use crate::distributions::normal;
use crate::error::{Error, Result};

const MAX_LOOKS: usize = 10;
const QUADRATURE_ORDER: usize = 32;

/// Correlation between standardized interim statistics at looks `i` and `j`.
fn look_correlation(timing_i: f64, timing_j: f64) -> f64 {
    (timing_i.min(timing_j) / timing_i.max(timing_j)).sqrt()
}

fn normal_pdf(z: f64) -> f64 {
    (-0.5 * z * z).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

fn mean_at_look(drift: f64, inflation: f64, timing: f64) -> f64 {
    drift * (inflation * timing).sqrt()
}

/// Conditional upper-tail probability at `look_idx` given earlier standardized values.
fn conditional_upper_tail(
    bounds: &[f64],
    timing: &[f64],
    look_idx: usize,
    z_prev: &[f64; MAX_LOOKS],
    drift: f64,
    inflation: f64,
) -> f64 {
    let mut mean = mean_at_look(drift, inflation, timing[look_idx]);
    let mut variance = 1.0;

    for idx in 0..look_idx {
        let rho = look_correlation(timing[idx], timing[look_idx]);
        mean += rho * (z_prev[idx] - mean_at_look(drift, inflation, timing[idx]));
        variance -= rho * rho;
    }

    let sd = variance.max(1e-12).sqrt();
    1.0 - normal::cdf((bounds[look_idx] - mean) / sd)
}

/// Probability of stopping at exactly `stop_at` under drift `drift` and inflation `inflation`.
pub fn prob_stop_at_exact(
    bounds: &[f64],
    timing: &[f64],
    stop_at: usize,
    drift: f64,
    inflation: f64,
) -> f64 {
    if stop_at >= bounds.len() {
        return 0.0;
    }

    if stop_at == 0 {
        let mean = mean_at_look(drift, inflation, timing[0]);
        return 1.0 - normal::cdf(bounds[0] - mean);
    }

    let mut z_prev = [0.0; MAX_LOOKS];
    integrate_stop(bounds, timing, 0, stop_at, drift, inflation, &mut z_prev)
        .clamp(0.0, 1.0)
}

fn integrate_stop(
    bounds: &[f64],
    timing: &[f64],
    depth: usize,
    stop_at: usize,
    drift: f64,
    inflation: f64,
    z_prev: &mut [f64; MAX_LOOKS],
) -> f64 {
    if depth == stop_at {
        return conditional_upper_tail(bounds, timing, stop_at, z_prev, drift, inflation);
    }

    let mean_d = mean_at_look(drift, inflation, timing[depth]);
    let upper = bounds[depth];
    let (nodes, weights) = scaled_quadrature(-6.0, upper.min(6.0));

    let mut integral = 0.0;
    for (node, weight) in nodes.iter().zip(weights.iter()) {
        z_prev[depth] = *node;
        let density = normal_pdf(node - mean_d)
            * integrate_stop(bounds, timing, depth + 1, stop_at, drift, inflation, z_prev);
        integral += weight * density;
    }

    integral
}

fn scaled_quadrature(lower: f64, upper: f64) -> (Vec<f64>, Vec<f64>) {
    let (nodes, weights) = gauss_legendre_quadrature();
    let half_span = 0.5 * (upper - lower);
    let midpoint = 0.5 * (upper + lower);
    let scaled_nodes: Vec<f64> = nodes
        .iter()
        .map(|node| midpoint + half_span * node)
        .collect();
    let scaled_weights: Vec<f64> = weights.iter().map(|weight| half_span * weight).collect();
    (scaled_nodes, scaled_weights)
}

fn gauss_legendre_quadrature() -> (Vec<f64>, Vec<f64>) {
    match QUADRATURE_ORDER {
        32 => gauss_legendre_32(),
        _ => gauss_legendre_32(),
    }
}

fn gauss_legendre_32() -> (Vec<f64>, Vec<f64>) {
    // Abscissae and weights on (-1, 1) from Golub-Welsch tables.
    let nodes = [
        -0.997_263_861_877_481, -0.988_819_378_752_316, -0.976_147_541_318_949,
        -0.959_258_958_818_634, -0.938_274_029_026_189, -0.913_311_583_172_699,
        -0.884_499_347_997_496, -0.851_985_031_527_199, -0.815_925_618_427_359,
        -0.776_467_123_458_669, -0.733_792_571_391_425, -0.688_092_689_258_369,
        -0.639_580_971_795_240, -0.588_475_010_511_470, -0.534_997_619_887_097,
        -0.479_387_861_234_747, -0.422_884_285_458_445, -0.365_725_118_758_206,
        -0.308_133_449_497_838, -0.250_352_116_867_777, -0.192_632_653_179_719,
        -0.135_222_941_392_347, -0.078_376_322_311_476, -0.022_352_539_053_078,
        0.022_352_539_053_078, 0.078_376_322_311_476, 0.135_222_941_392_347,
        0.192_632_653_179_719, 0.250_352_116_867_777, 0.308_133_449_497_838,
        0.365_725_118_758_206, 0.422_884_285_458_445,
    ];
    let weights = [
        0.007_640_172_204_127, 0.017_369_396_766_458, 0.027_185_394_947_700,
        0.037_069_862_586_038, 0.047_019_425_986_794, 0.057_028_382_748_405,
        0.067_089_391_691_267, 0.077_195_156_278_747, 0.087_337_534_408_069,
        0.097_508_699_364_981, 0.107_700_398_034_608, 0.117_903_511_227_708,
        0.128_109_758_705_615, 0.138_310_017_197_021, 0.148_495_720_113_576,
        0.158_658_283_283_936, 0.168_788_371_994_768, 0.178_877_169_958_781,
        0.188_916_202_593_450, 0.198_898_110_123_659, 0.208_816_482_345_581,
        0.218_664_102_500_162, 0.228_434_418_089_304, 0.238_120_945_531_848,
        0.247_717_308_587_312, 0.257_217_423_451_485, 0.266_615_219_505_123,
        0.275_904_374_583_873, 0.285_079_227_469_278, 0.294_124_661_599_903,
        0.303_035_269_630_234, 0.311_805_837_743_095,
    ];
    (nodes.to_vec(), weights.to_vec())
}

/// Solve symmetric upper efficacy boundaries for equally spaced looks.
pub fn solve_upper_bounds(incremental_spends: &[f64], timing: &[f64]) -> Result<Vec<f64>> {
    if incremental_spends.len() != timing.len() {
        return Err(Error::Internal(
            "incremental spends and timing must have equal length".into(),
        ));
    }

    let mut bounds: Vec<f64> = Vec::with_capacity(incremental_spends.len());

    for look_idx in 0..incremental_spends.len() {
        let target = incremental_spends[look_idx];
        let mut lo = 0.0;
        let mut hi = if look_idx == 0 { 6.0 } else { bounds[look_idx - 1].max(0.5) };

        while prob_stop_at_exact(&append_bound(&bounds, hi), timing, look_idx, 0.0, 1.0) > target {
            hi += 1.0;
            if hi > 12.0 {
                return Err(Error::ConvergenceFailure(
                    "failed to bracket group sequential boundary".into(),
                ));
            }
        }

        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            let trial = append_bound(&bounds, mid);
            if prob_stop_at_exact(&trial, timing, look_idx, 0.0, 1.0) > target {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        bounds.push(0.5 * (lo + hi));
    }

    Ok(bounds)
}

fn append_bound(bounds: &[f64], candidate: f64) -> Vec<f64> {
    let mut trial = bounds.to_vec();
    trial.push(candidate);
    trial
}

/// Power under drift `theta` for upper-boundary group sequential design.
pub fn power_under_drift(
    bounds: &[f64],
    timing: &[f64],
    theta: f64,
    inflation: f64,
) -> f64 {
    let mut power = 0.0;
    for look_idx in 0..bounds.len() {
        power += prob_stop_at_exact(bounds, timing, look_idx, theta, inflation);
    }
    power.clamp(0.0, 1.0)
}

/// Solve inflation factor relative to a fixed-design drift.
///
/// Matches gsDesign `n.I[k]` for symmetric two-sided spending (test.type = 4):
/// find the multiplier `w` such that equally spaced information `w * t_i`
/// achieves target power at the fixed-design drift.
pub fn sample_size_inflation(
    bounds: &[f64],
    timing: &[f64],
    fixed_drift: f64,
    target_power: f64,
) -> Result<(f64, f64)> {
    let mut lo = 1.0;
    let mut hi = 1.5;

    while power_under_drift(bounds, timing, fixed_drift, hi) < target_power {
        hi = hi * 1.1 + 0.05;
        if hi > 5.0 {
            return Err(Error::ConvergenceFailure(
                "failed to bracket group sequential sample size inflation".into(),
            ));
        }
    }

    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if power_under_drift(bounds, timing, fixed_drift, mid) >= target_power {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    let inflation = 0.5 * (lo + hi);
    let required_drift = fixed_drift;
    Ok((required_drift, inflation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::design::spending::{incremental_spends, SpendingFunction};
    use approx::assert_relative_eq;

    fn timing(k: u32) -> Vec<f64> {
        (1..=k)
            .map(|look| f64::from(look) / f64::from(k))
            .collect()
    }

    fn fixed_drift_gsdesign(alpha: f64, power: f64) -> f64 {
        normal::quantile(1.0 - alpha) + normal::quantile(power)
    }

    #[test]
    fn obrien_fleming_bounds_match_gsdesign_k3() {
        let spends = incremental_spends(0.05, 3, SpendingFunction::ObrienFleming);
        let bounds = solve_upper_bounds(&spends, &timing(3)).expect("bounds");
        assert_relative_eq!(bounds[0], 3.200102, epsilon = 0.02);
        assert_relative_eq!(bounds[1], 2.140815, epsilon = 0.02);
        assert_relative_eq!(bounds[2], 1.694812, epsilon = 0.02);
    }

    #[test]
    fn pocock_bounds_match_gsdesign_k3() {
        let spends = incremental_spends(0.05, 3, SpendingFunction::Pocock);
        let bounds = solve_upper_bounds(&spends, &timing(3)).expect("bounds");
        assert_relative_eq!(bounds[0], 2.002014, epsilon = 0.02);
        assert_relative_eq!(bounds[1], 1.993797, epsilon = 0.02);
        assert_relative_eq!(bounds[2], 1.980304, epsilon = 0.02);
    }

    #[test]
    fn obrien_fleming_inflation_matches_gsdesign_k3() {
        let spends = incremental_spends(0.05, 3, SpendingFunction::ObrienFleming);
        let t = timing(3);
        let bounds = solve_upper_bounds(&spends, &t).expect("bounds");
        let drift = fixed_drift_gsdesign(0.05, 0.8);
        let (_, inflation) = sample_size_inflation(&bounds, &t, drift, 0.8).expect("inflation");
        assert_relative_eq!(inflation, 1.07867, epsilon = 0.02);
    }

    #[test]
    fn pocock_inflation_matches_gsdesign_k3() {
        let spends = incremental_spends(0.05, 3, SpendingFunction::Pocock);
        let t = timing(3);
        let bounds = solve_upper_bounds(&spends, &t).expect("bounds");
        let drift = fixed_drift_gsdesign(0.05, 0.8);
        let (_, inflation) = sample_size_inflation(&bounds, &t, drift, 0.8).expect("inflation");
        assert_relative_eq!(inflation, 1.228415, epsilon = 0.02);
    }

    #[test]
    fn pocock_inflation_matches_gsdesign_k5() {
        let spends = incremental_spends(0.05, 5, SpendingFunction::Pocock);
        let t = timing(5);
        let bounds = solve_upper_bounds(&spends, &t).expect("bounds");
        let drift = fixed_drift_gsdesign(0.05, 0.8);
        let (_, inflation) = sample_size_inflation(&bounds, &t, drift, 0.8).expect("inflation");
        assert_relative_eq!(inflation, 1.290963, epsilon = 0.02);
    }
}
