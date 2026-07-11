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

/// Return all registered methods. Empty until Phase 1 adds the first method.
pub fn list_methods() -> &'static [MethodDescriptor] {
    &[]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_methods_is_empty_during_phase_0() {
        assert!(list_methods().is_empty());
    }
}
