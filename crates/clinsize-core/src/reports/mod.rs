//! Report data assembly for exported calculation summaries.

pub mod html;

use crate::methods::binary::odds_ratio::{OddsRatioInput, OddsRatioResult};
use crate::methods::binary::risk_ratio::{RiskRatioInput, RiskRatioResult};
use crate::methods::binary::two_proportion_difference::{
    TwoProportionDifferenceInput, TwoProportionDifferenceResult,
};
use crate::methods::continuous::ancova_two_sample::{AncovaTwoSampleInput, AncovaTwoSampleResult};
use crate::methods::continuous::one_sample_ttest::{OneSampleTTestInput, OneSampleTTestResult};
use crate::methods::continuous::one_way_anova::{OneWayAnovaInput, OneWayAnovaResult};
use crate::methods::continuous::paired_ttest::{PairedTTestInput, PairedTTestResult};
use crate::methods::continuous::two_sample_ttest::{TwoSampleTTestInput, TwoSampleTTestResult};
use crate::methods::design::multiplicity::{MultiplicityInput, MultiplicityResult};
use crate::methods::survival::log_rank::{LogRankInput, LogRankResult};
use crate::types::SolveMode;

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

/// Render a two-sample ANCOVA calculation summary as Markdown.
pub fn ancova_two_sample_markdown(
    input: &AncovaTwoSampleInput,
    result: &AncovaTwoSampleResult,
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
        "- **Method:** Two-sample ANCOVA (approximate variance reduction)".into(),
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
        format!(
            "- **Unadjusted standard deviation:** {:.4}",
            input.standard_deviation
        ),
        format!(
            "- **Baseline-outcome correlation:** {:.4}",
            input.baseline_outcome_correlation
        ),
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
        format!(
            "- **Effect size (Cohen's d, unadjusted SD):** {:.4}",
            result.effect_size
        ),
        format!(
            "- **Adjusted standard deviation:** {:.4}",
            result.adjusted_standard_deviation
        ),
        format!(
            "- **Variance reduction factor (1 − ρ²):** {:.4}",
            result.variance_reduction_factor
        ),
    ];

    append_warnings(&mut lines, &result.warnings);
    lines.push(String::new());
    lines.push("## Reproducibility".into());
    lines.push(format!("- **Engine version:** {engine_version}"));
    lines.push("- **Validation source:** R `power.t.test` with σ_adj = σ_y × √(1 − ρ²)".into());

    lines.join("\n")
}

/// Render a two-proportion difference calculation summary as Markdown.
pub fn two_proportion_difference_markdown(
    input: &TwoProportionDifferenceInput,
    result: &TwoProportionDifferenceResult,
    engine_version: &str,
) -> String {
    let dropout = input
        .dropout_rate
        .map(|rate| format!("{rate:.4}"))
        .unwrap_or_else(|| "none".into());
    let margin = input
        .noninferiority_margin
        .map(|m| format!("{m:.4}"))
        .unwrap_or_else(|| "n/a".into());

    let mut lines = vec![
        "# ClinSize calculation summary".into(),
        String::new(),
        "## Method".into(),
        "- **Method:** Two-sample difference in proportions".into(),
        "- **Endpoint:** Binary".into(),
        format!("- **Solve mode:** {:?}", input.solve_mode),
        format!("- **Study objective:** {:?}", input.study_objective),
        format!("- **Alternative:** {:?}", input.alternative),
        String::new(),
        "## Inputs".into(),
        format!("- **Alpha:** {:.4}", input.alpha),
        format!("- **Control event rate:** {:.4}", input.control_rate),
        format!("- **Treatment event rate:** {:.4}", input.treatment_rate),
        format!("- **Allocation ratio:** {:.4}", input.allocation_ratio),
        format!("- **Non-inferiority margin:** {margin}"),
        format!("- **Dropout rate:** {dropout}"),
        String::new(),
        "## Results".into(),
        format!("- **Control N:** {}", result.n_control),
        format!("- **Treatment N:** {}", result.n_treatment),
        format!("- **Total N:** {}", result.total_n),
        format!(
            "- **Dropout-adjusted total N:** {}",
            result.total_n_adjusted
        ),
        format!("- **Achieved power:** {:.4}", result.achieved_power),
        format!(
            "- **Rate difference (treatment − control):** {:.4}",
            result.rate_difference
        ),
    ];

    append_warnings(&mut lines, &result.warnings);
    lines.push(String::new());
    lines.push("## Reproducibility".into());
    lines.push(format!("- **Engine version:** {engine_version}"));
    lines.push(
        "- **Validation source:** R `power.prop.test` (superiority); TrialSize `TwoSampleProportion.NIS` (non-inferiority)".into(),
    );

    lines.join("\n")
}

/// Render an odds-ratio calculation summary as Markdown.
pub fn odds_ratio_markdown(
    input: &OddsRatioInput,
    result: &OddsRatioResult,
    engine_version: &str,
) -> String {
    binary_effect_markdown(&BinaryEffectReportContext {
        method_name: "Odds ratio superiority",
        solve_mode: input.solve_mode,
        alpha: input.alpha,
        control_rate: input.control_rate,
        treatment_rate: input.treatment_rate,
        allocation_ratio: input.allocation_ratio,
        dropout_rate: input.dropout_rate,
        n_control: result.n_control,
        n_treatment: result.n_treatment,
        total_n: result.total_n,
        total_n_adjusted: result.total_n_adjusted,
        achieved_power: result.achieved_power,
        effect_line: format!("- **Odds ratio:** {:.4}", result.odds_ratio),
        warnings: &result.warnings,
        engine_version,
        validation_source: "TrialSize `RelativeRisk.Equality` (log odds ratio; Chow et al. 2003)",
    })
}

/// Render a risk-ratio calculation summary as Markdown.
pub fn risk_ratio_markdown(
    input: &RiskRatioInput,
    result: &RiskRatioResult,
    engine_version: &str,
) -> String {
    binary_effect_markdown(&BinaryEffectReportContext {
        method_name: "Risk ratio superiority",
        solve_mode: input.solve_mode,
        alpha: input.alpha,
        control_rate: input.control_rate,
        treatment_rate: input.treatment_rate,
        allocation_ratio: input.allocation_ratio,
        dropout_rate: input.dropout_rate,
        n_control: result.n_control,
        n_treatment: result.n_treatment,
        total_n: result.total_n,
        total_n_adjusted: result.total_n_adjusted,
        achieved_power: result.achieved_power,
        effect_line: format!("- **Risk ratio:** {:.4}", result.risk_ratio),
        warnings: &result.warnings,
        engine_version,
        validation_source: "Chow et al. 2003 log risk-ratio normal approximation",
    })
}

struct BinaryEffectReportContext<'a> {
    method_name: &'a str,
    solve_mode: SolveMode,
    alpha: f64,
    control_rate: f64,
    treatment_rate: f64,
    allocation_ratio: f64,
    dropout_rate: Option<f64>,
    n_control: u32,
    n_treatment: u32,
    total_n: u32,
    total_n_adjusted: u32,
    achieved_power: f64,
    effect_line: String,
    warnings: &'a [crate::types::CalculationWarning],
    engine_version: &'a str,
    validation_source: &'a str,
}

fn binary_effect_markdown(ctx: &BinaryEffectReportContext<'_>) -> String {
    let dropout = ctx
        .dropout_rate
        .map(|rate| format!("{rate:.4}"))
        .unwrap_or_else(|| "none".into());

    let mut lines = vec![
        "# ClinSize calculation summary".into(),
        String::new(),
        "## Method".into(),
        format!("- **Method:** {}", ctx.method_name),
        "- **Endpoint:** Binary".into(),
        format!("- **Solve mode:** {:?}", ctx.solve_mode),
        String::new(),
        "## Inputs".into(),
        format!("- **Alpha:** {:.4}", ctx.alpha),
        format!("- **Control event rate:** {:.4}", ctx.control_rate),
        format!("- **Treatment event rate:** {:.4}", ctx.treatment_rate),
        format!("- **Allocation ratio:** {:.4}", ctx.allocation_ratio),
        format!("- **Dropout rate:** {dropout}"),
        String::new(),
        "## Results".into(),
        format!("- **Control N:** {}", ctx.n_control),
        format!("- **Treatment N:** {}", ctx.n_treatment),
        format!("- **Total N:** {}", ctx.total_n),
        format!("- **Dropout-adjusted total N:** {}", ctx.total_n_adjusted),
        format!("- **Achieved power:** {:.4}", ctx.achieved_power),
        ctx.effect_line.clone(),
    ];

    append_warnings(&mut lines, ctx.warnings);
    lines.push(String::new());
    lines.push("## Reproducibility".into());
    lines.push(format!("- **Engine version:** {}", ctx.engine_version));
    lines.push(format!(
        "- **Validation source:** {}",
        ctx.validation_source
    ));

    lines.join("\n")
}

/// Render a log-rank calculation summary as Markdown.
pub fn log_rank_markdown(
    input: &LogRankInput,
    result: &LogRankResult,
    engine_version: &str,
) -> String {
    let mut lines = vec![
        "# ClinSize calculation summary".into(),
        String::new(),
        "## Method".into(),
        "- **Method:** Two-arm log-rank test".into(),
        "- **Endpoint:** Survival".into(),
        format!("- **Solve mode:** {:?}", input.solve_mode),
        format!("- **Alternative:** {:?}", input.alternative),
        String::new(),
        "## Inputs".into(),
        format!("- **Alpha:** {:.4}", input.alpha),
        format!(
            "- **Hazard ratio (treatment / control):** {:.4}",
            input.hazard_ratio
        ),
        format!("- **Allocation ratio:** {:.4}", input.allocation_ratio),
        String::new(),
        "## Results".into(),
        format!("- **Required total events:** {}", result.required_events),
        format!("- **Expected control events:** {}", result.events_control),
        format!(
            "- **Expected treatment events:** {}",
            result.events_treatment
        ),
        format!("- **Achieved power:** {:.4}", result.achieved_power),
        format!("- **Hazard ratio:** {:.4}", result.hazard_ratio),
    ];

    if let (Some(control_hazard), Some(accrual), Some(follow_up)) = (
        input.control_hazard_rate,
        input.accrual_duration,
        input.minimum_follow_up,
    ) {
        lines.push(String::new());
        lines.push("## Accrual assumptions".into());
        lines.push(format!("- **Control hazard rate:** {control_hazard:.4}"));
        lines.push(format!("- **Accrual duration:** {accrual:.4}"));
        lines.push(format!("- **Minimum follow-up:** {follow_up:.4}"));
        let dropout = input
            .dropout_hazard_rate
            .map(|rate| format!("{rate:.4}"))
            .unwrap_or_else(|| "0".into());
        lines.push(format!("- **Dropout hazard rate:** {dropout}"));
    }

    if let (Some(total_n), Some(n_control), Some(n_treatment)) =
        (result.total_n, result.n_control, result.n_treatment)
    {
        lines.push(String::new());
        lines.push("## Enrolled subjects".into());
        lines.push(format!("- **Control N:** {n_control}"));
        lines.push(format!("- **Treatment N:** {n_treatment}"));
        lines.push(format!("- **Total N:** {total_n}"));
        if let (Some(p_control), Some(p_treatment)) = (
            result.probability_event_control,
            result.probability_event_treatment,
        ) {
            lines.push(format!("- **Control event probability:** {p_control:.4}"));
            lines.push(format!(
                "- **Treatment event probability:** {p_treatment:.4}"
            ));
        }
    }

    append_warnings(&mut lines, &result.warnings);
    lines.push(String::new());
    lines.push("## Reproducibility".into());
    lines.push(format!("- **Engine version:** {engine_version}"));
    lines.push(
        "- **Validation source:** R `gsDesign::nEvents` (Schoenfeld 1981 approximation)".into(),
    );

    lines.join("\n")
}

/// Render a multiplicity adjustment summary as Markdown.
pub fn multiplicity_markdown(
    input: &MultiplicityInput,
    result: &MultiplicityResult,
    engine_version: &str,
) -> String {
    let mut lines = vec![
        "# ClinSize calculation summary".into(),
        String::new(),
        "## Method".into(),
        "- **Method:** Family-wise alpha adjustment".into(),
        "- **Endpoint:** Design".into(),
        format!("- **Adjustment method:** {:?}", input.adjustment_method),
        String::new(),
        "## Inputs".into(),
        format!(
            "- **Family-wise alpha:** {:.6}",
            input.family_wise_alpha
        ),
        format!(
            "- **Number of comparisons:** {}",
            input.number_of_comparisons
        ),
        String::new(),
        "## Results".into(),
        format!(
            "- **Adjusted per-comparison alpha:** {:.6}",
            result.adjusted_alpha
        ),
        format!(
            "- **Alpha reduction factor:** {:.6}",
            result.alpha_reduction_factor
        ),
        String::new(),
        "## Usage".into(),
        "- Use the adjusted per-comparison alpha as the `alpha` input in endpoint sample size calculations.".into(),
    ];

    append_warnings(&mut lines, &result.warnings);
    lines.push(String::new());
    lines.push("## Reproducibility".into());
    lines.push(format!("- **Engine version:** {engine_version}"));
    lines.push(
        "- **Validation source:** Closed-form Bonferroni and Sidak formulas (Julious 2010)".into(),
    );

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
