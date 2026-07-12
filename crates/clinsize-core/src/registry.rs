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
        id: "continuous.change_from_baseline",
        display_name: "Change from baseline",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/change-from-baseline.md"),
    },
    MethodDescriptor {
        id: "continuous.mmrm",
        display_name: "MMRM (longitudinal)",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/mmrm-longitudinal.md"),
    },
    MethodDescriptor {
        id: "continuous.mann_whitney",
        display_name: "Mann-Whitney U",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/nonparametric.md"),
    },
    MethodDescriptor {
        id: "continuous.wilcoxon_signed_rank",
        display_name: "Wilcoxon signed-rank",
        endpoint_category: "Continuous",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/nonparametric.md"),
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
        id: "binary.one_sample_binomial",
        display_name: "One-sample binomial",
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
        id: "count.negative_binomial",
        display_name: "Negative binomial",
        endpoint_category: "Count",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/count-endpoints.md"),
    },
    MethodDescriptor {
        id: "count.poisson",
        display_name: "Poisson",
        endpoint_category: "Count",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/count-endpoints.md"),
    },
    MethodDescriptor {
        id: "ordinal.proportional_odds",
        display_name: "Proportional odds",
        endpoint_category: "Ordinal",
        supported_solve_modes: &[SolveMode::SampleSize, SolveMode::Power],
        documentation_path: Some("handbook/calculations/ordinal-endpoints.md"),
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
    MethodDescriptor {
        id: "design.blinded_ssre",
        display_name: "Blinded sample size re-estimation",
        endpoint_category: "Design",
        supported_solve_modes: &[],
        documentation_path: Some("handbook/calculations/blinded-ssre.md"),
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
        assert_eq!(methods.len(), 20);
        assert_eq!(methods[0].id, "continuous.two_sample_ttest");
        assert_eq!(methods[5].id, "continuous.change_from_baseline");
        assert_eq!(methods[6].id, "continuous.mmrm");
        assert_eq!(methods[7].id, "continuous.mann_whitney");
        assert_eq!(methods[12].id, "binary.risk_ratio");
        assert_eq!(methods[13].id, "count.negative_binomial");
        assert_eq!(methods[14].id, "count.poisson");
        assert_eq!(methods[15].id, "ordinal.proportional_odds");
        assert_eq!(methods[19].id, "design.blinded_ssre");
    }
}
