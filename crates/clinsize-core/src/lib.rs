//! Pure Rust statistical engine for ClinSize.
//!
//! No Tauri, UI, or OS dependencies belong in this crate — see the
//! handbook's `04-rust-engine-standards.md`. This crate should be usable
//! standalone from the desktop app, the CLI, and tests.

pub mod dispatch;
pub mod distributions;
pub mod error;
pub mod methods;
pub mod numerics;
pub mod registry;
pub mod reports;
pub mod types;
pub mod validation;
pub mod validation_report;

pub use error::{Error, Result};
pub use types::{
    Alternative, CalculationWarning, CorrelationStructure, SolveMode, StudyObjective,
};

/// The `clinsize-core` crate version, for embedding in exported reports
/// per `10-release-distribution.md` ("Result Reproducibility").
pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_version_is_not_empty() {
        assert!(!engine_version().is_empty());
    }
}
