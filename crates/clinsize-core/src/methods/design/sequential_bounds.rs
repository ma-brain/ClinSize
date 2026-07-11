//! Boundary solving and sample size inflation for group sequential designs.

use crate::distributions::normal;
use crate::error::{Error, Result};

const MAX_LOOKS: usize = 10;
const MVN_MAXPTS: usize = 12_000;

fn look_correlation(timing_i: f64, timing_j: f64) -> f64 {
    (timing_i.min(timing_j) / timing_i.max(timing_j)).sqrt()
}

fn mean_at_look(drift: f64, inflation: f64, timing: f64) -> f64 {
    drift * (inflation * timing).sqrt()
}

fn build_correlation_matrix(timing: &[f64], n: usize) -> [[f64; MAX_LOOKS]; MAX_LOOKS] {
    let mut corr = [[0.0; MAX_LOOKS]; MAX_LOOKS];
    for i in 0..n {
        corr[i][i] = 1.0;
        for j in (i + 1)..n {
            let rho = look_correlation(timing[i], timing[j]);
            corr[i][j] = rho;
            corr[j][i] = rho;
        }
    }
    corr
}

fn halton(index: usize, base: usize) -> f64 {
    let mut f = 1.0;
    let mut r = 0.0;
    let mut i = index + 1;
    let b = base as f64;
    while i > 0 {
        f /= b;
        r += f * (i % base) as f64;
        i /= base;
    }
    r.clamp(1e-12, 1.0 - 1e-12)
}

fn cholesky_lower(
    n: usize,
    corr: &[[f64; MAX_LOOKS]; MAX_LOOKS],
    chol: &mut [[f64; MAX_LOOKS]; MAX_LOOKS],
) {
    for i in 0..n {
        for j in 0..=i {
            let mut sum = corr[i][j];
            for k in 0..j {
                sum -= chol[i][k] * chol[j][k];
            }
            if i == j {
                chol[i][j] = sum.max(0.0).sqrt();
            } else {
                chol[i][j] = if chol[j][j].abs() > 1e-15 {
                    sum / chol[j][j]
                } else {
                    0.0
                };
            }
        }
    }
}

/// Genz QMC for P(lower[i] < X_i < upper[i]) with X ~ N(0, corr).
fn mvn_rectangle_probability(
    n: usize,
    lower: &[f64],
    upper: &[f64],
    corr: &[[f64; MAX_LOOKS]; MAX_LOOKS],
) -> f64 {
    if n == 0 {
        return 1.0;
    }
    if n == 1 {
        let lo = if lower[0].is_finite() {
            normal::cdf(lower[0])
        } else {
            0.0
        };
        let hi = if upper[0].is_finite() {
            normal::cdf(upper[0])
        } else {
            1.0
        };
        return (hi - lo).clamp(0.0, 1.0);
    }

    let mut lo = [0.0; MAX_LOOKS];
    let mut hi = [0.0; MAX_LOOKS];
    for i in 0..n {
        lo[i] = lower[i];
        hi[i] = upper[i];
    }

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        let pa = normal::cdf(hi[a]) - normal::cdf(lo[a]);
        let pb = normal::cdf(hi[b]) - normal::cdf(lo[b]);
        pa.partial_cmp(&pb).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut permuted = [[0.0; MAX_LOOKS]; MAX_LOOKS];
    let mut lo_p = [0.0; MAX_LOOKS];
    let mut hi_p = [0.0; MAX_LOOKS];
    for i in 0..n {
        lo_p[i] = lo[order[i]];
        hi_p[i] = hi[order[i]];
        for j in 0..n {
            permuted[i][j] = corr[order[i]][order[j]];
        }
    }

    let mut chol = [[0.0; MAX_LOOKS]; MAX_LOOKS];
    cholesky_lower(n, &permuted, &mut chol);

    let ct0 = chol[0][0];
    let ci = normal::cdf(lo_p[0] / ct0);
    let dci = normal::cdf(hi_p[0] / ct0) - ci;

    let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
    let mut total = 0.0;

    for rep in 0..MVN_MAXPTS {
        let mut y = [0.0; MAX_LOOKS];
        let mut c = ci;
        let mut dc = dci;
        let mut pv = dc;

        for i in 1..n {
            let ui = halton(rep, primes[(i - 1) % primes.len()]);
            let x = (2.0 * ui - 1.0).abs();
            y[i - 1] = normal::quantile((c + x * dc).clamp(1e-12, 1.0 - 1e-12));

            let mut s = 0.0;
            for j in 0..i {
                s += chol[i][j] * y[j];
            }
            let ct = chol[i][i];
            c = normal::cdf((lo_p[i] - s) / ct);
            let d = normal::cdf((hi_p[i] - s) / ct);
            dc = d - c;
            if dc <= 1e-15 {
                pv = 0.0;
                break;
            }
            pv *= dc;
        }

        total += pv;
    }

    (total / MVN_MAXPTS as f64).clamp(0.0, 1.0)
}

fn all_below_prob(k: usize, bounds: &[f64], timing: &[f64], drift: f64, inflation: f64) -> f64 {
    if k == 0 {
        return 1.0;
    }

    let lower = [f64::NEG_INFINITY; MAX_LOOKS];
    let mut upper = [0.0; MAX_LOOKS];
    for i in 0..k {
        upper[i] = bounds[i] - mean_at_look(drift, inflation, timing[i]);
    }

    let corr = build_correlation_matrix(timing, k);
    mvn_rectangle_probability(k, &lower, &upper, &corr)
}

/// Probability of stopping at exactly `stop_at`.
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
        return (1.0 - all_below_prob(1, bounds, timing, drift, inflation)).clamp(0.0, 1.0);
    }

    let below_before = all_below_prob(stop_at, bounds, timing, drift, inflation);
    let below_including = all_below_prob(stop_at + 1, bounds, timing, drift, inflation);
    (below_before - below_including).clamp(0.0, 1.0)
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
        let mut hi = if look_idx == 0 {
            6.0
        } else {
            bounds[look_idx - 1].max(0.5)
        };

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
pub fn power_under_drift(bounds: &[f64], timing: &[f64], theta: f64, inflation: f64) -> f64 {
    let mut power = 0.0;
    for look_idx in 0..bounds.len() {
        power += prob_stop_at_exact(bounds, timing, look_idx, theta, inflation);
    }
    power.clamp(0.0, 1.0)
}

/// Solve inflation factor relative to a fixed-design drift.
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

    Ok((fixed_drift, 0.5 * (lo + hi)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::design::spending::{incremental_spends, SpendingFunction};
    use approx::assert_relative_eq;

    fn timing(k: u32) -> Vec<f64> {
        (1..=k).map(|look| f64::from(look) / f64::from(k)).collect()
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
        assert_relative_eq!(inflation, 1.020305, epsilon = 0.02);
    }

    #[test]
    fn pocock_inflation_matches_gsdesign_k3() {
        let spends = incremental_spends(0.05, 3, SpendingFunction::Pocock);
        let t = timing(3);
        let bounds = solve_upper_bounds(&spends, &t).expect("bounds");
        let drift = fixed_drift_gsdesign(0.05, 0.8);
        let (_, inflation) = sample_size_inflation(&bounds, &t, drift, 0.8).expect("inflation");
        assert_relative_eq!(inflation, 1.176743, epsilon = 0.02);
    }

    #[test]
    fn pocock_inflation_matches_gsdesign_k5() {
        let spends = incremental_spends(0.05, 5, SpendingFunction::Pocock);
        let t = timing(5);
        let bounds = solve_upper_bounds(&spends, &t).expect("bounds");
        let drift = fixed_drift_gsdesign(0.05, 0.8);
        let (_, inflation) = sample_size_inflation(&bounds, &t, drift, 0.8).expect("inflation");
        assert_relative_eq!(inflation, 1.221578, epsilon = 0.02);
    }
}
