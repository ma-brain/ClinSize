//! Report data assembly for exported calculation summaries.

use crate::methods::continuous::one_sample_ttest::{OneSampleTTestInput, OneSampleTTestResult};
use crate::methods::continuous::one_way_anova::{OneWayAnovaInput, OneWayAnovaResult};
use crate::methods::continuous::paired_ttest::{PairedTTestInput, PairedTTestResult};
use crate::methods::continuous::two_sample_ttest::{TwoSampleTTestInput, TwoSampleTTestResult};

fn append_warnings(lines: &mut Vec<String>, warnings: &[crate::types::CalculationWarning]) {
    lines.push("## Assumptions and warnings".into());
    for warning in warnings {
        lines.push(format!("- **{}:** {}", warning.code, warning.message));
    }
}

fn append_reproducibility(lines: &mut Vec<String>, engine_version: &str) {
    lines.push(String::new());
    lines.push("## Reproducibility".into());
    lines.push(format!("- **Engine version:** {engine_version}"));
    lines.push("- **Validation source:** R `power.t.test` (stats package)".into());
}

/// Render a two-sample t-test calculation summary as Markdown.
pub fn two_sample_ttest_markdown(
    input: &TwoSampleTTestInput,
    result: &TwoSampleTTestResult,
    engine_version: &str,
) -> String {
    let dropout = input
        .dropout_rate
        .map(|rate| format!("{rate:.4}"))
        .unwrap_or_else(|| "none".into());

    let mut lines = vec![
        "# ClinSize calculation summary".into(),
        String::new(),
        "## Method".into(),
        "- **Method:** Two-sample t-test (equal variance)".into(),
        "- **Endpoint:** Continuous".into(),
        format!("- **Solve mode:** {:?}", input.solve_mode),
        format!("- **Alternative:** {:?}", input.alternative),
        String::new(),
        "## Inputs".into(),
        format!("- **Alpha:** {:.4}", input.alpha),
        format!(
            "- **Target power:** {}",
            input
                .power
                .map(|p| format!("{p:.4}"))
                .unwrap_or_else(|| "n/a".into())
        ),
        format!(
            "- **Control N (given):** {}",
            input
                .control_n
                .map(|n| n.to_string())
                .unwrap_or_else(|| "n/a".into())
        ),
        format!("- **Mean difference:** {:.4}", input.mean_difference),
        format!("- **Standard deviation:** {:.4}", input.standard_deviation),
        format!("- **Allocation ratio:** {:.4}", input.allocation_ratio),
        format!("- **Dropout rate:** {dropout}"),
        String::new(),
        "## Results".into(),
        format!("- **Control N:** {}", result.n_control),
        format!("- **Treatment N:** {}", result.n_treatment),
        format!("- **Total N:** {}", result.total_n),
        format!(
            "- **Dropout-adjusted control N:** {}",
            result.n_control_adjusted
        ),
        format!(
            "- **Dropout-adjusted treatment N:** {}",
            result.n_treatment_adjusted
        ),
        format!(
            "- **Dropout-adjusted total N:** {}",
            result.total_n_adjusted
        ),
        format!("- **Achieved power:** {:.4}", result.achieved_power),
        format!("- **Effect size (Cohen's d):** {:.4}", result.effect_size),
    ];

    append_warnings(&mut lines, &result.warnings);
    append_reproducibility(&mut lines, engine_version);

    lines.join("\n")
}

/// Render a one-sample t-test calculation summary as Markdown.
pub fn one_sample_ttest_markdown(
    input: &OneSampleTTestInput,
    result: &OneSampleTTestResult,
    engine_version: &str,
) -> String {
    let mut lines = vec![
        "# ClinSize calculation summary".into(),
        String::new(),
        "## Method".into(),
        "- **Method:** One-sample t-test".into(),
        "- **Endpoint:** Continuous".into(),
        format!("- **Solve mode:** {:?}", input.solve_mode),
        format!("- **Alternative:** {:?}", input.alternative),
        String::new(),
        "## Inputs".into(),
        format!("- **Alpha:** {:.4}", input.alpha),
        format!("- **Mean difference:** {:.4}", input.mean_difference),
        format!("- **Standard deviation:** {:.4}", input.standard_deviation),
        String::new(),
        "## Results".into(),
        format!("- **N:** {}", result.n),
        format!("- **Dropout-adjusted N:** {}", result.n_adjusted),
        format!("- **Achieved power:** {:.4}", result.achieved_power),
        format!("- **Effect size (Cohen's d):** {:.4}", result.effect_size),
    ];

    append_warnings(&mut lines, &result.warnings);
    append_reproducibility(&mut lines, engine_version);

    lines.join("\n")
}

/// Render a paired t-test calculation summary as Markdown.
pub fn paired_ttest_markdown(
    input: &PairedTTestInput,
    result: &PairedTTestResult,
    engine_version: &str,
) -> String {
    let mut lines = vec![
        "# ClinSize calculation summary".into(),
        String::new(),
        "## Method".into(),
        "- **Method:** Paired t-test".into(),
        "- **Endpoint:** Continuous".into(),
        format!("- **Solve mode:** {:?}", input.solve_mode),
        format!("- **Alternative:** {:?}", input.alternative),
        String::new(),
        "## Inputs".into(),
        format!("- **Alpha:** {:.4}", input.alpha),
        format!("- **Mean difference:** {:.4}", input.mean_difference),
        format!("- **Standard deviation:** {:.4}", input.standard_deviation),
        String::new(),
        "## Results".into(),
        format!("- **Pairs:** {}", result.n_pairs),
        format!("- **Dropout-adjusted pairs:** {}", result.n_pairs_adjusted),
        format!("- **Achieved power:** {:.4}", result.achieved_power),
        format!("- **Effect size (Cohen's d):** {:.4}", result.effect_size),
    ];

    append_warnings(&mut lines, &result.warnings);
    append_reproducibility(&mut lines, engine_version);

    lines.join("\n")
}

/// Render a one-way ANOVA calculation summary as Markdown.
pub fn one_way_anova_markdown(
    input: &OneWayAnovaInput,
    result: &OneWayAnovaResult,
    engine_version: &str,
) -> String {
    let mut lines = vec![
        "# ClinSize calculation summary".into(),
        String::new(),
        "## Method".into(),
        "- **Method:** One-way ANOVA (balanced groups)".into(),
        "- **Endpoint:** Continuous".into(),
        format!("- **Solve mode:** {:?}", input.solve_mode),
        String::new(),
        "## Inputs".into(),
        format!("- **Alpha:** {:.4}", input.alpha),
        format!("- **Number of groups:** {}", input.n_groups),
        format!(
            "- **Between-group variance:** {:.4}",
            input.between_variance
        ),
        format!("- **Within-group variance:** {:.4}", input.within_variance),
        String::new(),
        "## Results".into(),
        format!("- **N per group:** {}", result.n_per_group),
        format!("- **Total N:** {}", result.total_n),
        format!(
            "- **Dropout-adjusted N per group:** {}",
            result.n_per_group_adjusted
        ),
        format!(
            "- **Dropout-adjusted total N:** {}",
            result.total_n_adjusted
        ),
        format!("- **Achieved power:** {:.4}", result.achieved_power),
        format!("- **Effect size (Cohen's f):** {:.4}", result.effect_size),
    ];

    append_warnings(&mut lines, &result.warnings);
    lines.push(String::new());
    lines.push("## Reproducibility".into());
    lines.push(format!("- **Engine version:** {engine_version}"));
    lines.push("- **Validation source:** R `power.anova.test` (stats package)".into());

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::continuous::two_sample_ttest::calculate;
    use crate::types::{Alternative, SolveMode};

    #[test]
    fn markdown_includes_primary_outputs() {
        let input = TwoSampleTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            mean_difference: 1.0,
            standard_deviation: 1.0,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate(input.clone()).expect("calculate");
        let markdown = two_sample_ttest_markdown(&input, &result, "0.1.0");

        assert!(markdown.contains("Control N:** 17"));
        assert!(markdown.contains("Achieved power:**"));
        assert!(markdown.contains("equal_variance"));
    }
}
