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

const METHODS: &[MethodDescriptor] = &[
    MethodDescriptor {
        id: "continuous.two_sample_ttest",
        display_name: "Two-sample t-test",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/continuous-ttest.md"),
    },
    MethodDescriptor {
        id: "continuous.one_sample_ttest",
        display_name: "One-sample t-test",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/continuous-ttest.md"),
    },
    MethodDescriptor {
        id: "continuous.paired_ttest",
        display_name: "Paired t-test",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/continuous-ttest.md"),
    },
    MethodDescriptor {
        id: "continuous.one_way_anova",
        display_name: "One-way ANOVA",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/anova-ancova.md"),
    },
    MethodDescriptor {
        id: "continuous.ancova_two_sample",
        display_name: "Two-sample ANCOVA",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/anova-ancova.md"),
    },
    MethodDescriptor {
        id: "binary.two_proportion_difference",
        display_name: "Difference in proportions",
        endpoint_category: "Binary",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/binary-endpoints.md"),
    },
    MethodDescriptor {
        id: "binary.odds_ratio",
        display_name: "Odds ratio",
        endpoint_category: "Binary",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/binary-endpoints.md"),
    },
    MethodDescriptor {
        id: "binary.risk_ratio",
        display_name: "Risk ratio",
        endpoint_category: "Binary",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/binary-endpoints.md"),
    },
    MethodDescriptor {
        id: "survival.log_rank",
        display_name: "Log-rank test",
        endpoint_category: "Survival",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/survival-endpoints.md"),
    },
    MethodDescriptor {
        id: "design.multiplicity",
        display_name: "Multiplicity adjustment",
        endpoint_category: "Design",
        supported_solve_modes: &[],
        documentation_path: Some("handbook/calculations/multiplicity.md"),
    },
    MethodDescriptor {
        id: "design.group_sequential",
        display_name: "Group sequential design",
        endpoint_category: "Design",
        supported_solve_modes: &[],
        documentation_path: Some("handbook/calculations/group-sequential.md"),
    },
];

/// Return all registered methods.
pub fn list_methods() -> &'static [MethodDescriptor] {
    METHODS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_methods_includes_continuous_ttests() {
        let methods = list_methods();
        assert_eq!(methods.len(), 11);
        assert_eq!(methods[0].id, "continuous.two_sample_ttest");
        assert_eq!(methods[5].id, "binary.two_proportion_difference");
        assert_eq!(methods[6].id, "binary.odds_ratio");
        assert_eq!(methods[7].id, "binary.risk_ratio");
        assert_eq!(methods[8].id, "survival.log_rank");
        assert_eq!(methods[9].id, "design.multiplicity");
        assert_eq!(methods[10].id, "design.group_sequential");
    }
}
