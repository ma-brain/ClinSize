//! Numerical helpers for root-finding and search.

/// Maximum control-group sample size explored when solving for sample size.
pub const MAX_SAMPLE_SIZE_SEARCH: u32 = 1_000_000;

/// Find the smallest integer `n` in `[min_n, max_n]` satisfying `predicate`.
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

    if predicate(lo) {
        Some(lo)
    } else {
        None
    }
}
