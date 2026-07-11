//! Method registry — describes available calculations without implementing them.
//!
//! See `02-architecture.md` ("Method Registry"). The registry routes and
//! documents methods; `methods/` holds the calculation logic.

use crate::types::SolveMode;

/// Metadata for a calculation method exposed to the UI and CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub endpoint_category: &'static str,
    pub supported_solve_modes: &'static [SolveMode],
    /// Handbook-relative path to the method note, when one exists.
    pub documentation_path: Option<&'static str>,
}

const METHODS: &[MethodDescriptor] = &[MethodDescriptor {
    id: "continuous.two_sample_ttest",
    display_name: "Two-sample t-test",
    endpoint_category: "Continuous",
    supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
    documentation_path: Some("handbook/calculations/continuous-ttest.md"),
}];

/// Return all registered methods.
pub fn list_methods() -> &'static [MethodDescriptor] {
    METHODS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_methods_includes_two_sample_ttest() {
        let methods = list_methods();
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].id, "continuous.two_sample_ttest");
    }
}
