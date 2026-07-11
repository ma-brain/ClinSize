//! Probability distribution support for power/sample-size calculations.
//!
//! `statrs` has no noncentral t, noncentral F, or noncentral chi-square
//! support, so exact (non-approximated) power calculations need those
//! implemented here directly, validated against R's `pt`/`pf`/`pchisq`
//! with `ncp` set. See design record for the decision to implement these
//! in pure Rust rather than depend on an external numerical engine.
