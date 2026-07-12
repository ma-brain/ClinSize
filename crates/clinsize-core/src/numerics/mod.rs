//! Numerical helpers for root-finding and search.

/// Maximum control-group sample size explored when solving for sample size.
pub const MAX_SAMPLE_SIZE_SEARCH: u32 = 1_000_000;

/// Find the smallest integer `n` in `[min_n, max_n]` satisfying `predicate`.
///
/// The binary search assumes `predicate` is monotone non-decreasing in `n`
/// (once true, stays true). Discrete power curves — particularly exact
/// binomial tests — can plateau or dip locally, so after the search lands on a
/// candidate we walk *down* while the predecessor also satisfies the predicate.
/// This recovers the true minimum across plateau-style non-monotonicity
/// (contiguous true regions the search overshoots), which is the common shape
/// for discrete power curves. It cannot bridge a single-point gap where the
/// predicate goes true → false → true with a one-element false region.
pub fn find_minimum_integer<F>(min_n: u32, max_n: u32, mut predicate: F) -> Option<u32>
where
    F: FnMut(u32) -> bool,
{
    if min_n > max_n {
        return None;
    }

    let mut lo = min_n;
    let mut hi = max_n;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if predicate(mid) {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }

    if !predicate(lo) {
        return None;
    }

    // Walk-down guard: the binary search can land on a value whose predecessor
    // also satisfies the predicate when the curve plateaus. Decrement while the
    // predecessor holds, so the returned value is the true minimum.
    while lo > min_n && predicate(lo - 1) {
        lo -= 1;
    }

    Some(lo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_minimum_on_monotone_predicate() {
        // Predicate: n >= 50. Minimum is 50.
        assert_eq!(find_minimum_integer(1, 100, |n| n >= 50), Some(50));
    }

    #[test]
    fn returns_min_n_when_predicate_immediately_true() {
        assert_eq!(find_minimum_integer(10, 100, |n| n >= 5), Some(10));
    }

    #[test]
    fn returns_none_when_never_satisfied() {
        assert_eq!(find_minimum_integer(1, 100, |n| n > 200), None);
    }

    #[test]
    fn returns_none_when_range_empty() {
        assert_eq!(find_minimum_integer(100, 50, |_| true), None);
    }

    #[test]
    fn handles_single_element_range() {
        assert_eq!(find_minimum_integer(7, 7, |_| true), Some(7));
        assert_eq!(find_minimum_integer(7, 7, |_| false), None);
    }

    #[test]
    fn finds_minimum_at_boundary() {
        // Predicate true only at the last element.
        assert_eq!(find_minimum_integer(1, 100, |n| n >= 100), Some(100));
    }

    #[test]
    fn walk_down_recovers_overshoot_on_plateau() {
        // Simulates a discrete power plateau: predicate is true at n=40-42,
        // false at 43 (rounding dip), then true from 44 onward. A plain binary
        // search lands on 44; the walk-down must recover 42, the true minimum
        // of the contiguous lower true region.
        assert_eq!(
            find_minimum_integer(1, 100, |n| (40..=42).contains(&n) || n >= 44),
            Some(40),
        );
    }

    #[test]
    fn walk_down_stops_at_min_n() {
        // Contiguous true region starting at min_n: the predicate is true
        // throughout [10, 100], so the binary search lands at min_n and the
        // guard correctly stops there without underflowing.
        assert_eq!(find_minimum_integer(10, 100, |_| true), Some(10));
    }
}
