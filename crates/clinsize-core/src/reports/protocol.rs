//! Protocol-ready narrative text for sample size sections (distinct from technical rationale).

use crate::distributions::normal;
use crate::methods::binary::odds_ratio::{OddsRatioInput, OddsRatioResult};
use crate::methods::binary::one_sample_binomial::{
    OneSampleBinomialInput, OneSampleBinomialResult,
};
use crate::methods::binary::risk_ratio::{RiskRatioInput, RiskRatioResult};
use crate::methods::binary::two_proportion_difference::{
    TwoProportionDifferenceInput, TwoProportionDifferenceResult,
};
use crate::methods::continuous::ancova_two_sample::{AncovaTwoSampleInput, AncovaTwoSampleResult};
use crate::methods::continuous::change_from_baseline::{
    ChangeFromBaselineInput, ChangeFromBaselineResult,
};
use crate::methods::continuous::mann_whitney::{MannWhitneyInput, MannWhitneyResult};
use crate::methods::continuous::mmrm::{MmrmInput, MmrmResult};
use crate::methods::continuous::one_sample_ttest::{OneSampleTTestInput, OneSampleTTestResult};
use crate::methods::continuous::one_way_anova::{OneWayAnovaInput, OneWayAnovaResult};
use crate::methods::continuous::paired_ttest::{PairedTTestInput, PairedTTestResult};
use crate::methods::continuous::two_sample_ttest::{TwoSampleTTestInput, TwoSampleTTestResult};
use crate::methods::continuous::two_way_anova::{AnovaEffect, TwoWayAnovaInput, TwoWayAnovaResult};
use crate::methods::continuous::wilcoxon_signed_rank::{
    WilcoxonSignedRankInput, WilcoxonSignedRankResult,
};
use crate::methods::count::negative_binomial::{NegativeBinomialInput, NegativeBinomialResult};
use crate::methods::count::poisson::{PoissonInput, PoissonResult};
use crate::methods::design::blinded_ssre::{BlindedSsreInput, BlindedSsreResult};
use crate::methods::design::group_sequential::{GroupSequentialInput, GroupSequentialResult};
use crate::methods::design::multiplicity::{
    MultiplicityInput, MultiplicityMethod, MultiplicityResult,
};
use crate::methods::design::spending::SpendingFunction;
use crate::methods::ordinal::proportional_odds::{ProportionalOddsInput, ProportionalOddsResult};
use crate::methods::survival::log_rank::{LogRankInput, LogRankResult};
use crate::types::{Alternative, CorrelationStructure, SolveMode, StudyObjective};

fn join_paragraphs(paragraphs: Vec<String>) -> String {
    paragraphs.join("\n\n")
}

fn significance_level_phrase(alternative: Alternative) -> &'static str {
    match alternative {
        Alternative::TwoSided => "two-sided significance level",
        Alternative::Greater | Alternative::Less => "one-sided significance level",
    }
}

fn z_alpha_label_and_value(alpha: f64, alternative: Alternative) -> (&'static str, f64) {
    match alternative {
        Alternative::TwoSided => ("zα/2", normal::upper_tail_critical(alpha / 2.0)),
        Alternative::Greater | Alternative::Less => ("zα", normal::upper_tail_critical(alpha)),
    }
}

fn z_beta_value(power: f64) -> f64 {
    normal::quantile(power)
}

fn format_z_criticals(alpha: f64, alternative: Alternative, power: f64) -> String {
    let (label, z_alpha) = z_alpha_label_and_value(alpha, alternative);
    format!("{label} = {z_alpha:.3}, zβ = {:.3}", z_beta_value(power))
}

fn format_percent_int(rate: f64) -> String {
    format!("{:.0}%", rate * 100.0)
}

fn format_power_percent(power: f64) -> String {
    format!("{:.0} %", power * 100.0)
}

fn cohens_h(p_alt: f64, p_null: f64) -> f64 {
    2.0 * (p_alt.sqrt().asin() - p_null.sqrt().asin())
}

fn append_dropout_enrollment_sentence(
    paragraph: &mut String,
    dropout_rate: Option<f64>,
    evaluable_phrase: &str,
    enrollable_phrase: &str,
) {
    paragraph.push_str(&format!(
        " The required number of evaluable {evaluable_phrase}."
    ));
    if let Some(rate) = dropout_rate {
        paragraph.push_str(&format!(
            " Allowing for an anticipated withdrawal rate of {}, {enrollable_phrase} will be enrollable.",
            format_percent_int(rate),
        ));
    }
}

fn two_group_sample_size_phrase(n_control: u32, n_treatment: u32, total: u32) -> String {
    if n_control == n_treatment {
        format!(
            "{n_control} subjects per arm ({total} subjects in total)",
            n_control = n_control,
            total = total,
        )
    } else {
        format!(
            "{n_control} control and {n_treatment} treatment subjects ({total} subjects in total)",
            n_control = n_control,
            n_treatment = n_treatment,
            total = total,
        )
    }
}

fn two_group_enrollable_phrase(n_control: u32, n_treatment: u32, total: u32) -> String {
    if n_control == n_treatment {
        format!("{n_control} subjects per arm ({total} in total)")
    } else {
        format!("{n_control} control and {n_treatment} treatment subjects ({total} in total)")
    }
}

/// Protocol text for a two-sample t-test calculation.
pub fn two_sample_ttest_protocol(
    input: &TwoSampleTTestInput,
    result: &TwoSampleTTestResult,
) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(input.alpha, input.alternative);
            let enroll = two_group_sample_size_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            paragraphs.push(format!(
                "A sample size of {enroll} is required to detect a treatment difference of \
                 Δ = {:.2} (pooled SD = {:.2}, Cohen's d = {:.2}) with {} power at a {} of α = \
                 {:.2} ({}).",
                input.mean_difference,
                input.standard_deviation,
                result.effect_size,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
            ));

            let mut p2 = String::from(
                "The primary analysis will use an unadjusted two-sample t-test on change from \
                 baseline, assuming normality and a common pooled SD across arms.",
            );
            let evaluable = if result.n_control == result.n_treatment {
                format!("per arm is {}", result.n_control)
            } else {
                format!(
                    "subjects are {} control and {} treatment",
                    result.n_control, result.n_treatment
                )
            };
            let enrollable = two_group_enrollable_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &evaluable,
                &enrollable,
            );
            p2.push_str(&format!(
                " Sample size was computed using the formula n = 2·({z_label} + zβ)²·SD² / Δ² \
                 in accordance with ICH E9."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "With {} control and {} treatment subjects enrolled (total N = {}), the study \
                 achieves {:.0} % power to detect a treatment difference of Δ = {:.2} (pooled SD \
                 = {:.2}, Cohen's d = {:.2}) at a {} of α = {:.2} ({}).",
                control_n,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                input.mean_difference,
                input.standard_deviation,
                result.effect_size,
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, result.achieved_power),
            ));
            paragraphs.push(
                "The primary analysis will use an unadjusted two-sample t-test on change from \
                 baseline, assuming normality and a common pooled SD across arms. Sample size was \
                 computed using the equal-variance two-sample t-test formula in accordance with \
                 ICH E9."
                    .into(),
            );
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for two-sample t-test"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a one-sample t-test calculation.
pub fn one_sample_ttest_protocol(
    input: &OneSampleTTestInput,
    result: &OneSampleTTestResult,
) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(input.alpha, input.alternative);
            let enroll_n = result.n_adjusted;
            paragraphs.push(format!(
                "A sample size of {enroll_n} subjects (single arm) is required to detect a mean \
                 difference of Δ = {:.2} from the reference value (SD = {:.2}, Cohen's d = {:.2}) \
                 with {} power at a {} of α = {:.2} ({}).",
                input.mean_difference,
                input.standard_deviation,
                result.effect_size,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
            ));

            let mut p2 = "The primary analysis will use a one-sample t-test against the reference \
                          mean, assuming normality."
                .to_string();
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &format!("subjects is {}", result.n),
                &format!("{enroll_n} subjects in total"),
            );
            p2.push_str(&format!(
                " Sample size was computed using the formula n = (({z_label} + zβ)²·SD²) / Δ² \
                 in accordance with ICH E9."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let n = input.n.expect("validated for power mode");
            paragraphs.push(format!(
                "With {n} enrolled subjects (single arm), the study achieves {:.0} % power to \
                 detect a mean difference of Δ = {:.2} from the reference value (SD = {:.2}, \
                 Cohen's d = {:.2}) at a {} of α = {:.2} ({}).",
                result.achieved_power * 100.0,
                input.mean_difference,
                input.standard_deviation,
                result.effect_size,
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, result.achieved_power),
            ));
            paragraphs.push(
                "The primary analysis will use a one-sample t-test against the reference mean, \
                 assuming normality."
                    .into(),
            );
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for one-sample t-test"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a paired t-test calculation.
pub fn paired_ttest_protocol(input: &PairedTTestInput, result: &PairedTTestResult) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(input.alpha, input.alternative);
            let enroll_pairs = result.n_pairs_adjusted;
            paragraphs.push(format!(
                "A sample size of {enroll_pairs} paired subjects is required to detect a mean \
                 paired difference of Δ = {:.2} (SD of differences = {:.2}, Cohen's d = {:.2}) \
                 with {} power at a {} of α = {:.2} ({}).",
                input.mean_difference,
                input.standard_deviation,
                result.effect_size,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
            ));

            let mut p2 = "The primary analysis will use a paired t-test on within-subject \
                          differences, assuming normality of paired differences."
                .to_string();
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &format!("pairs is {}", result.n_pairs),
                &format!("{enroll_pairs} pairs"),
            );
            p2.push_str(&format!(
                " Sample size was computed using the formula n = (({z_label} + zβ)²·SD²) / Δ² \
                 for paired differences."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let n_pairs = input.n_pairs.expect("validated for power mode");
            paragraphs.push(format!(
                "With {n_pairs} paired subjects enrolled, the study achieves {:.0} % power to \
                 detect a mean paired difference of Δ = {:.2} (SD of differences = {:.2}, \
                 Cohen's d = {:.2}) at a {} of α = {:.2} ({}).",
                result.achieved_power * 100.0,
                input.mean_difference,
                input.standard_deviation,
                result.effect_size,
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, result.achieved_power),
            ));
            paragraphs.push(
                "The primary analysis will use a paired t-test on within-subject differences, \
                 assuming normality of paired differences."
                    .into(),
            );
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for paired t-test"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a balanced one-way ANOVA calculation.
pub fn one_way_anova_protocol(input: &OneWayAnovaInput, result: &OneWayAnovaResult) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "A sample size of {} subjects per group ({} groups; total N = {}) is required \
                 to detect the specified between-group variance component (Cohen's f = {:.2}) \
                 with {} power at a two-sided omnibus significance level of α = {:.2}.",
                result.n_per_group_adjusted,
                input.n_groups,
                result.total_n_adjusted,
                result.effect_size,
                format_power_percent(target),
                input.alpha,
            ));

            let mut p2 = format!(
                "The primary analysis will use a one-way ANOVA F-test across {} balanced groups, \
                 assuming independent observations and a common within-group variance.",
                input.n_groups,
            );
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &format!("subjects per group is {}", result.n_per_group),
                &format!(
                    "{} subjects per group ({} in total)",
                    result.n_per_group_adjusted, result.total_n_adjusted
                ),
            );
            p2.push_str(
                " Sample size was computed using the noncentral F distribution for a balanced \
                 one-way layout.",
            );
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let n_per_group = input.n_per_group.expect("validated for power mode");
            paragraphs.push(format!(
                "With {n_per_group} subjects per group ({} groups; total N = {}), the omnibus \
                 one-way ANOVA achieves {:.0} % power (Cohen's f = {:.2}) at α = {:.2}.",
                input.n_groups,
                result.total_n,
                result.achieved_power * 100.0,
                result.effect_size,
                input.alpha,
            ));
            paragraphs.push(format!(
                "The primary analysis will use a one-way ANOVA F-test across {} balanced groups, \
                 assuming independent observations and a common within-group variance.",
                input.n_groups,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for one-way ANOVA"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a two-way ANOVA calculation.
pub fn two_way_anova_protocol(input: &TwoWayAnovaInput, result: &TwoWayAnovaResult) -> String {
    let mut paragraphs = Vec::new();

    let effect_label = match input.primary_effect {
        AnovaEffect::MainA => "the main effect of factor A",
        AnovaEffect::MainB => "the main effect of factor B",
        AnovaEffect::Interaction => "the A × B interaction",
    };

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "A balanced two-way design with factor A at {} levels and factor B at {} levels \
                 requires {} subjects per cell (total N = {}) to detect {effect_label} with \
                 Cohen's f = {:.2} at {} power and a two-sided α of {:.2}.",
                input.n_levels_a,
                input.n_levels_b,
                result.n_per_cell_adjusted,
                result.total_n_adjusted,
                result.effect_size,
                format_power_percent(target),
                input.alpha,
            ));
            paragraphs.push(format!(
                "The primary analysis is a two-way fixed-effects ANOVA F-test for {effect_label}.",
            ));
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "With {} subjects per cell (total N = {}), the design achieves {:.0}% power to \
                 detect {effect_label} with Cohen's f = {:.2} at a two-sided α of {:.2}.",
                input.n_per_cell.expect("validated"),
                result.total_n,
                result.achieved_power * 100.0,
                result.effect_size,
                input.alpha,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for two-way ANOVA"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a two-sample ANCOVA calculation.
pub fn ancova_two_sample_protocol(
    input: &AncovaTwoSampleInput,
    result: &AncovaTwoSampleResult,
) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(input.alpha, input.alternative);
            let enroll = two_group_sample_size_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            paragraphs.push(format!(
                "A sample size of {enroll} is required to detect a treatment difference of \
                 Δ = {:.2} after adjusting for a baseline covariate (unadjusted SD = {:.2}, \
                 adjusted SD = {:.2}, baseline-outcome correlation ρ = {:.2}, Cohen's d = {:.2}) \
                 with {} power at a {} of α = {:.2} ({}).",
                input.mean_difference,
                input.standard_deviation,
                result.adjusted_standard_deviation,
                input.baseline_outcome_correlation,
                result.effect_size,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
            ));

            let mut p2 = format!(
                "The primary analysis will use an analysis of covariance (ANCOVA) with one \
                 baseline covariate, assuming normality and a common within-group variance. \
                 Variance reduction uses σ_adj = σ_y × √(1 − ρ²) = {:.2}.",
                result.adjusted_standard_deviation,
            );
            let evaluable = if result.n_control == result.n_treatment {
                format!("per arm is {}", result.n_control)
            } else {
                format!(
                    "subjects are {} control and {} treatment",
                    result.n_control, result.n_treatment
                )
            };
            let enrollable = two_group_enrollable_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &evaluable,
                &enrollable,
            );
            p2.push_str(&format!(
                " Sample size was computed using the equal-variance two-sample formula with \
                 adjusted SD and n = 2·({z_label} + zβ)²·σ_adj² / Δ²."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "With {} control and {} treatment subjects enrolled (total N = {}), the \
                 ANCOVA-adjusted design achieves {:.0} % power to detect Δ = {:.2} (adjusted SD \
                 = {:.2}) at a {} of α = {:.2}.",
                control_n,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                input.mean_difference,
                result.adjusted_standard_deviation,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will use an analysis of covariance (ANCOVA) with one \
                 baseline covariate, assuming normality and a common within-group variance."
                    .into(),
            );
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for ANCOVA"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a change-from-baseline calculation.
pub fn change_from_baseline_protocol(
    input: &ChangeFromBaselineInput,
    result: &ChangeFromBaselineResult,
) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let enroll = two_group_sample_size_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            paragraphs.push(format!(
                "A sample size of {enroll} is required to detect a mean change-from-baseline \
                 difference of Δ = {:.2} (outcome SD = {:.2}, change-score SD σ_cfb = {:.2}, \
                 baseline-outcome correlation ρ = {:.2}) with {} power at a {} of α = {:.2}.",
                input.mean_difference,
                input.standard_deviation,
                result.change_score_standard_deviation,
                input.baseline_outcome_correlation,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will compare treatment and control mean change-from-baseline \
                 scores using an equal-variance two-sample t-test, assuming normality of CFB and \
                 common baseline/follow-up variance."
                    .into(),
            );
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "With {} control and {} treatment subjects (total N = {}), the design achieves \
                 {:.0} % power to detect a mean CFB difference of {:.2} (σ_cfb = {:.2}) at a {} \
                 of α = {:.2}.",
                input.control_n.expect("validated"),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                input.mean_difference,
                result.change_score_standard_deviation,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for CFB"),
    }

    join_paragraphs(paragraphs)
}

fn correlation_structure_label(structure: CorrelationStructure) -> &'static str {
    match structure {
        CorrelationStructure::Unstructured => "unstructured",
        CorrelationStructure::Ar1 => "AR(1)",
        CorrelationStructure::CompoundSymmetry => "compound symmetry",
        CorrelationStructure::Toeplitz => "Toeplitz",
        CorrelationStructure::Csh => "CSH",
    }
}

/// Protocol text for an MMRM calculation.
pub fn mmrm_protocol(input: &MmrmInput, result: &MmrmResult) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let randomize =
                two_group_sample_size_phrase(result.n_control, result.n_treatment, result.total_n);
            let mut paragraph = format!(
                "A sample size of {randomize} randomized is required to detect a treatment \
                 effect of δ = {:.2} at the final post-baseline visit (final-visit SD σ = {:.2}, \
                 {} correlation ρ = {:.2}, k = {} post-baseline visits) with {} power at a {} of \
                 α = {:.3}, based on the mixed model for repeated measures method of Lu, Luo & \
                 Chen (2008).",
                input.treatment_effect,
                input.residual_standard_deviation,
                correlation_structure_label(input.correlation_structure),
                input.correlation,
                input.n_post_baseline_visits,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
            );
            if let Some(per_visit_dropout_rate) = input.per_visit_dropout_rate {
                paragraph.push_str(&format!(
                    " This accounts for an anticipated per-visit withdrawal rate of {:.0}% over \
                     {} visits (final-visit retention {:.0}%): subjects who discontinue \
                     contribute their observed visits to the analysis, so no additional \
                     enrollment inflation is applied.",
                    per_visit_dropout_rate * 100.0,
                    input.n_post_baseline_visits,
                    result.final_retention * 100.0,
                ));
            }
            paragraphs.push(paragraph);
            paragraphs.push(
                "The primary analysis will compare treatment and control at the final \
                 post-baseline visit using a mixed model for repeated measures (MMRM) with visit \
                 as a categorical factor, assuming equal variance and within-subject correlation \
                 across arms and monotone missingness."
                    .into(),
            );
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "With {} randomized control and {} randomized treatment subjects (total N = {}), \
                 the design achieves {:.0} % power to detect a treatment effect of δ = {:.2} at \
                 the final post-baseline visit (Lu-Luo-Chen variance factor φ = {:.3}) at a {} \
                 of α = {:.3}.",
                input.control_n.expect("validated"),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                input.treatment_effect,
                result.variance_factor,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for MMRM"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a negative binomial calculation.
pub fn negative_binomial_protocol(
    input: &NegativeBinomialInput,
    result: &NegativeBinomialResult,
) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let enroll = two_group_sample_size_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            paragraphs.push(format!(
                "A sample size of {enroll} is required to detect a rate ratio of {:.2} between \
                 treatment (λ₂ = {:.2}) and control (λ₁ = {:.2}) over exposure {:.2}, assuming \
                 NB2 dispersion k = {:.2}, with {} power at a {} of α = {:.2}.",
                result.rate_ratio,
                input.treatment_rate,
                input.control_rate,
                input.exposure_time,
                input.dispersion,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will compare event rates using a Wald test on the log rate \
                 ratio under a negative binomial model with fixed exposure."
                    .into(),
            );
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "With {} control and {} treatment subjects (total N = {}), the design achieves \
                 {:.0} % power to detect rate ratio {:.2} at a {} of α = {:.2}.",
                input.control_n.expect("validated"),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                result.rate_ratio,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for negative binomial"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a Poisson calculation.
pub fn poisson_protocol(input: &PoissonInput, result: &PoissonResult) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let enroll = two_group_sample_size_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            paragraphs.push(format!(
                "A sample size of {enroll} is required to detect a rate ratio of {:.2} between \
                 treatment (λ₂ = {:.2}) and control (λ₁ = {:.2}) over exposure {:.2}, assuming \
                 Poisson counts with no overdispersion, with {} power at a {} of α = {:.2}.",
                result.rate_ratio,
                input.treatment_rate,
                input.control_rate,
                input.exposure_time,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will compare event rates using a Wald test on the log rate \
                 ratio under a Poisson model with fixed exposure."
                    .into(),
            );
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "With {} control and {} treatment subjects (total N = {}), the design achieves \
                 {:.0} % power to detect rate ratio {:.2} at a {} of α = {:.2}.",
                input.control_n.expect("validated"),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                result.rate_ratio,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for Poisson"),
    }

    join_paragraphs(paragraphs)
}
pub fn proportional_odds_protocol(
    input: &ProportionalOddsInput,
    result: &ProportionalOddsResult,
) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let enroll = two_group_sample_size_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            paragraphs.push(format!(
                "A sample size of {enroll} is required to detect an ordinal odds ratio of {:.2} \
                 (efficiency ps = {:.3}) with treatment fraction {:.0}% and {} power at a \
                 two-sided significance level of α = {:.2}.",
                input.odds_ratio,
                result.efficiency,
                input.treatment_fraction * 100.0,
                format_power_percent(target),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will compare ordinal outcomes between treatment and control \
                 using a proportional odds model."
                    .into(),
            );
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "With {} control and {} treatment subjects (total N = {}), the design achieves \
                 {:.0} % power to detect odds ratio {:.2} at α = {:.2} (two-sided).",
                input.control_n.expect("validated"),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                input.odds_ratio,
                input.alpha,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for proportional odds"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a two-proportion difference calculation.
pub fn two_proportion_difference_protocol(
    input: &TwoProportionDifferenceInput,
    result: &TwoProportionDifferenceResult,
) -> String {
    let mut paragraphs = Vec::new();
    let objective = match input.study_objective {
        StudyObjective::Superiority => "superiority",
        StudyObjective::NonInferiority => "non-inferiority",
    };

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(input.alpha, input.alternative);
            let enroll = two_group_sample_size_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            let rate_phrase = if input.study_objective == StudyObjective::NonInferiority {
                let margin = input
                    .noninferiority_margin
                    .expect("validated for non-inferiority");
                format!(
                    "control rate p₀ = {}, treatment rate p₁ = {} (non-inferiority margin = {:.0}%)",
                    format_percent_int(input.control_rate),
                    format_percent_int(input.treatment_rate),
                    margin * 100.0,
                )
            } else {
                format!(
                    "control rate p₀ = {}, treatment rate p₁ = {} (risk difference {:.0}%)",
                    format_percent_int(input.control_rate),
                    format_percent_int(input.treatment_rate),
                    result.rate_difference.abs() * 100.0,
                )
            };
            paragraphs.push(format!(
                "A sample size of {enroll} is required for a two-group {objective} comparison \
                 of binary endpoints, assuming {rate_phrase}, with {} power at a {} of α = \
                 {:.2} ({}).",
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
            ));

            let mut p2 = format!(
                "The primary analysis will use a {objective} test for the difference in \
                 proportions between treatment and control."
            );
            let evaluable = if result.n_control == result.n_treatment {
                format!("per arm is {}", result.n_control)
            } else {
                format!(
                    "subjects are {} control and {} treatment",
                    result.n_control, result.n_treatment
                )
            };
            let enrollable = two_group_enrollable_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &evaluable,
                &enrollable,
            );
            p2.push_str(&format!(
                " Sample size was computed using a normal approximation to the difference in \
                 proportions with n derived from ({z_label} + zβ)² × variance / Δ²."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "With {} control and {} treatment subjects enrolled (total N = {}), the study \
                 achieves {:.0} % power for a {objective} comparison of proportions (control \
                 {:.0}%, treatment {:.0}%) at a {} of α = {:.2}.",
                control_n,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                input.control_rate * 100.0,
                input.treatment_rate * 100.0,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(format!(
                "The primary analysis will use a {objective} test for the difference in \
                 proportions between treatment and control."
            ));
        }
        SolveMode::DetectableEffect => {
            unreachable!("not implemented for two-proportion difference")
        }
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for an odds-ratio superiority calculation.
pub fn odds_ratio_protocol(input: &OddsRatioInput, result: &OddsRatioResult) -> String {
    binary_effect_protocol(
        "odds ratio",
        result.odds_ratio,
        "log odds ratio",
        input.solve_mode,
        input.alpha,
        input.power,
        input.control_n,
        input.control_rate,
        input.treatment_rate,
        input.allocation_ratio,
        input.alternative,
        input.dropout_rate,
        result.n_control,
        result.n_treatment,
        result.total_n,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
        result.achieved_power,
    )
}

/// Protocol text for a risk-ratio superiority calculation.
pub fn risk_ratio_protocol(input: &RiskRatioInput, result: &RiskRatioResult) -> String {
    binary_effect_protocol(
        "risk ratio",
        result.risk_ratio,
        "log risk ratio",
        input.solve_mode,
        input.alpha,
        input.power,
        input.control_n,
        input.control_rate,
        input.treatment_rate,
        input.allocation_ratio,
        input.alternative,
        input.dropout_rate,
        result.n_control,
        result.n_treatment,
        result.total_n,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
        result.achieved_power,
    )
}

#[allow(clippy::too_many_arguments)]
fn binary_effect_protocol(
    effect_label: &str,
    effect_value: f64,
    test_statistic: &str,
    solve_mode: SolveMode,
    alpha: f64,
    power: Option<f64>,
    control_n: Option<u32>,
    control_rate: f64,
    treatment_rate: f64,
    _allocation_ratio: f64,
    alternative: Alternative,
    dropout_rate: Option<f64>,
    n_control: u32,
    n_treatment: u32,
    total_n: u32,
    n_control_adjusted: u32,
    n_treatment_adjusted: u32,
    total_n_adjusted: u32,
    achieved_power: f64,
) -> String {
    let mut paragraphs = Vec::new();

    match solve_mode {
        SolveMode::SampleSize => {
            let target = power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(alpha, alternative);
            let enroll = two_group_sample_size_phrase(
                n_control_adjusted,
                n_treatment_adjusted,
                total_n_adjusted,
            );
            paragraphs.push(format!(
                "A sample size of {enroll} is required to detect a {effect_label} of \
                 {effect_value:.2} between treatment and control (control rate p₀ = {}, \
                 treatment rate p₁ = {}) with {} power at a {} of α = {:.2} ({}).",
                format_percent_int(control_rate),
                format_percent_int(treatment_rate),
                format_power_percent(target),
                significance_level_phrase(alternative),
                alpha,
                format_z_criticals(alpha, alternative, target),
            ));

            let mut p2 = format!(
                "The primary analysis will use a superiority test based on the {effect_label} \
                 ({test_statistic} normal approximation)."
            );
            let evaluable = if n_control == n_treatment {
                format!("per arm is {n_control}")
            } else {
                format!("subjects are {n_control} control and {n_treatment} treatment")
            };
            let enrollable = two_group_enrollable_phrase(
                n_control_adjusted,
                n_treatment_adjusted,
                total_n_adjusted,
            );
            append_dropout_enrollment_sentence(&mut p2, dropout_rate, &evaluable, &enrollable);
            p2.push_str(&format!(
                " Sample size was computed using a normal approximation to the {test_statistic} \
                 with n derived from ({z_label} + zβ)² × variance / (log effect)²."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let fixed_control_n = control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "With {fixed_control_n} control and {n_treatment} treatment subjects enrolled \
                 (total N = {total_n}), the study achieves {:.0} % power to detect a \
                 {effect_label} of {effect_value:.2} (control {:.0}%, treatment {:.0}%) at a {} \
                 of α = {:.2}.",
                achieved_power * 100.0,
                control_rate * 100.0,
                treatment_rate * 100.0,
                significance_level_phrase(alternative),
                alpha,
            ));
            paragraphs.push(format!(
                "The primary analysis will use a superiority test based on the {effect_label} \
                 ({test_statistic} normal approximation)."
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for binary effect measures"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a one-sample binomial calculation.
pub fn one_sample_binomial_protocol(
    input: &OneSampleBinomialInput,
    result: &OneSampleBinomialResult,
) -> String {
    let mut paragraphs = Vec::new();
    let h = cohens_h(input.response_rate, input.reference_rate);

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let alt_phrase = match input.alternative {
                Alternative::Greater => "exceeds",
                Alternative::Less => "is below",
                Alternative::TwoSided => "differs from",
            };
            paragraphs.push(format!(
                "A sample size of {} subjects (single arm) is required to show that the true \
                 response rate {} the benchmark rate of p₀ = {}, assuming an anticipated response \
                 rate of pA = {} (risk difference {:.0}%, Cohen's h = {:.2}) with {} power at a \
                 {} of α = {:.2} ({}).",
                result.n_adjusted,
                alt_phrase,
                format_percent_int(input.reference_rate),
                format_percent_int(input.response_rate),
                result.rate_difference.abs() * 100.0,
                h,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
            ));

            let mut p2 = match input.alternative {
                Alternative::Greater | Alternative::Less => {
                    "The primary analysis will use a one-sample binomial superiority test against \
                     the benchmark response rate."
                        .to_string()
                }
                Alternative::TwoSided => {
                    "The primary analysis will use a one-sample binomial test against the benchmark \
                     response rate."
                        .to_string()
                }
            };
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &format!("subjects is {}", result.n),
                &format!("{} subjects in total", result.n_adjusted),
            );
            p2.push_str(
                " Sample size was computed using the Fleiss arcsine approximation for a one-sample \
                 binomial test.",
            );
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let n = input.n.expect("validated for power mode");
            paragraphs.push(format!(
                "With {n} enrolled subjects (single arm), the study achieves {:.0} % power to \
                 show that the response rate differs from the benchmark p₀ = {} (anticipated \
                 pA = {}, Cohen's h = {:.2}) at a {} of α = {:.2}.",
                result.achieved_power * 100.0,
                format_percent_int(input.reference_rate),
                format_percent_int(input.response_rate),
                h,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will use a one-sample binomial test against the benchmark \
                 response rate."
                    .into(),
            );
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for one-sample binomial"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a Mann-Whitney U calculation.
pub fn mann_whitney_protocol(input: &MannWhitneyInput, result: &MannWhitneyResult) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(input.alpha, input.alternative);
            let enroll = two_group_sample_size_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            paragraphs.push(format!(
                "A sample size of {enroll} is required to detect a location shift of Δ = {:.2} \
                 (SD = {:.2}, Cohen's d = {:.2}, P(treatment > control) = {:.2}) with {} power \
                 at a {} of α = {:.2} ({}).",
                input.mean_difference,
                input.standard_deviation,
                result.effect_size,
                result.probability_superiority,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
            ));

            let mut p2 = "The primary analysis will use the Mann-Whitney U (Wilcoxon rank-sum) \
                          test as a nonparametric comparison of continuous endpoints."
                .to_string();
            let evaluable = if result.n_control == result.n_treatment {
                format!("per arm is {}", result.n_control)
            } else {
                format!(
                    "subjects are {} control and {} treatment",
                    result.n_control, result.n_treatment
                )
            };
            let enrollable = two_group_enrollable_phrase(
                result.n_control_adjusted,
                result.n_treatment_adjusted,
                result.total_n_adjusted,
            );
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &evaluable,
                &enrollable,
            );
            p2.push_str(&format!(
                " Sample size was computed using the Noether (1987) formula with \
                 ({z_label} + zβ)² / P(treatment > control)."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "With {control_n} control and {} treatment subjects enrolled (total N = {}), \
                 the Mann-Whitney design achieves {:.0} % power (P(treatment > control) = {:.2}) \
                 at a {} of α = {:.2}.",
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
                result.probability_superiority,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will use the Mann-Whitney U (Wilcoxon rank-sum) test as a \
                 nonparametric comparison of continuous endpoints."
                    .into(),
            );
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for Mann-Whitney"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a Wilcoxon signed-rank calculation.
pub fn wilcoxon_signed_rank_protocol(
    input: &WilcoxonSignedRankInput,
    result: &WilcoxonSignedRankResult,
) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(input.alpha, input.alternative);
            let enroll_pairs = result.n_pairs_adjusted;
            paragraphs.push(format!(
                "A sample size of {enroll_pairs} paired subjects is required to detect a \
                 paired difference of Δ = {:.2} (SD = {:.2}, Cohen's d = {:.2}, P(difference > 0) \
                 = {:.2}) with {} power at a {} of α = {:.2} ({}).",
                input.mean_difference,
                input.standard_deviation,
                result.effect_size,
                result.probability_positive_difference,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
            ));

            let mut p2 = "The primary analysis will use the Wilcoxon signed-rank test on \
                          within-subject paired differences."
                .to_string();
            append_dropout_enrollment_sentence(
                &mut p2,
                input.dropout_rate,
                &format!("pairs is {}", result.n_pairs),
                &format!("{enroll_pairs} pairs"),
            );
            p2.push_str(&format!(
                " Sample size was computed using the Noether (1987) signed-rank formula with \
                 ({z_label} + zβ)² / P(difference > 0)."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let n_pairs = input.n_pairs.expect("validated for power mode");
            paragraphs.push(format!(
                "With {n_pairs} paired subjects enrolled, the Wilcoxon signed-rank design \
                 achieves {:.0} % power (P(difference > 0) = {:.2}) at a {} of α = {:.2}.",
                result.achieved_power * 100.0,
                result.probability_positive_difference,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will use the Wilcoxon signed-rank test on within-subject \
                 paired differences."
                    .into(),
            );
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for Wilcoxon signed-rank"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a two-arm log-rank calculation.
pub fn log_rank_protocol(input: &LogRankInput, result: &LogRankResult) -> String {
    let mut paragraphs = Vec::new();

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            let (z_label, _) = z_alpha_label_and_value(input.alpha, input.alternative);
            paragraphs.push(format!(
                "A total of {} events is required to detect a hazard ratio (treatment / control) \
                 of {:.2} with {} power at a {} of α = {:.2} ({}), yielding expected events of \
                 {} in control and {} in treatment.",
                result.required_events,
                input.hazard_ratio,
                format_power_percent(target),
                significance_level_phrase(input.alternative),
                input.alpha,
                format_z_criticals(input.alpha, input.alternative, target),
                result.events_control,
                result.events_treatment,
            ));

            let mut p2 =
                "The primary analysis will use a two-arm log-rank test under proportional \
                          hazards, based on the Schoenfeld (1981) event-driven approximation."
                    .to_string();
            if let (Some(total_n), Some(n_control), Some(n_treatment)) =
                (result.total_n, result.n_control, result.n_treatment)
            {
                p2.push_str(&format!(
                    " Under the stated accrual assumptions, enrollment of {n_control} control and \
                     {n_treatment} treatment subjects (total N = {total_n}) is required to \
                     observe the target number of events."
                ));
            }
            p2.push_str(&format!(
                " Event count was computed using D = (({z_label} + zβ)² × p(1 − p)) / (ln HR)²."
            ));
            paragraphs.push(p2);
        }
        SolveMode::Power => {
            let total_events = input.total_events.expect("validated for power mode");
            paragraphs.push(format!(
                "With {total_events} total events ({} control, {} treatment), the log-rank \
                 design achieves {:.0} % power to detect a hazard ratio of {:.2} at a {} of \
                 α = {:.2}.",
                result.events_control,
                result.events_treatment,
                result.achieved_power * 100.0,
                input.hazard_ratio,
                significance_level_phrase(input.alternative),
                input.alpha,
            ));
            paragraphs.push(
                "The primary analysis will use a two-arm log-rank test under proportional hazards."
                    .into(),
            );
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for log-rank"),
    }

    join_paragraphs(paragraphs)
}

/// Protocol text for a family-wise alpha adjustment.
pub fn multiplicity_protocol(input: &MultiplicityInput, result: &MultiplicityResult) -> String {
    let method = match input.adjustment_method {
        MultiplicityMethod::Bonferroni => "Bonferroni",
        MultiplicityMethod::Sidak => "Šidák",
        MultiplicityMethod::Dunnett => "Dunnett",
        MultiplicityMethod::Holm => "Holm step-down gatekeeping",
        MultiplicityMethod::Hochberg => "Hochberg step-up gatekeeping",
        MultiplicityMethod::Graphical => "graphical gatekeeping",
    };

    let paragraphs = vec![
        format!(
            "A family-wise Type I error rate of α = {:.4} across {} comparisons will be \
             controlled using the {method} procedure, yielding an adjusted per-comparison \
             significance level of α_adj = {:.6} ({:.1}% of the naive α/m).",
            input.family_wise_alpha,
            input.number_of_comparisons,
            result.adjusted_alpha,
            result.alpha_reduction_factor * 100.0,
        ),
        format!(
            "Endpoint sample size calculations for comparisons within this family should use \
             α_adj = {:.6} as the Type I error rate. This adjustment addresses multiplicity in \
             hypothesis testing and does not by itself inflate sample size.",
            result.adjusted_alpha,
        ),
    ];

    join_paragraphs(paragraphs)
}

/// Protocol text for a group sequential design.
pub fn group_sequential_protocol(
    input: &GroupSequentialInput,
    result: &GroupSequentialResult,
) -> String {
    let spending = match input.spending_function {
        SpendingFunction::ObrienFleming => "O'Brien-Fleming",
        SpendingFunction::Pocock => "Pocock",
    };
    let final_look = result.looks.last().expect("at least one look");

    let paragraphs = vec![
        format!(
            "The study will employ a group sequential design with {} equally spaced interim \
             analyses (including the final analysis) using the {spending} alpha spending \
             function. The one-sided family-wise significance level for the efficacy boundary is \
             α = {:.4} and the target power is {:.0} %.",
            input.number_of_looks,
            input.alpha,
            input.target_power * 100.0,
        ),
        format!(
            "At the final analysis (information {:.0}%), the upper efficacy boundary is Z = \
             {:.3} with cumulative α spent {:.6}. The sample size inflation factor relative to a \
             fixed design is {:.4}; multiply the fixed-design sample size from the endpoint \
             calculation by this factor to obtain the maximum sample size under this plan.",
            final_look.information_fraction * 100.0,
            final_look.upper_z_boundary,
            final_look.cumulative_alpha_spent,
            result.sample_size_inflation_factor,
        ),
    ];

    join_paragraphs(paragraphs)
}

/// Protocol text for blinded sample size re-estimation.
pub fn blinded_ssre_protocol(input: &BlindedSsreInput, result: &BlindedSsreResult) -> String {
    let blinded_sd = input
        .blinded_interim_standard_deviation
        .unwrap_or(input.planned_standard_deviation);

    let mut paragraphs = vec![format!(
        "The study will include one blinded sample size re-estimation at {:.0} % of planned \
         enrollment, based on a blinded pooled standard deviation of {:.2} (planned σ₀ = {:.2}) \
         for a continuous two-group superiority design with mean difference Δ = {:.2}.",
        input.interim_fraction * 100.0,
        blinded_sd,
        input.planned_standard_deviation,
        input.mean_difference,
    )];

    let cap_sentence = if result.was_capped {
        format!(
            "Re-estimation inflates the planned per-arm sample size by a factor of {:.4}, but a \
             pre-specified cap of {:.1}× limits enrollment to {} control and {} treatment \
             subjects (total N = {}). At the capped enrollment the design provides {:.0} % power \
             under the blinded interim standard deviation.",
            result.variance_ratio,
            input.max_sample_size_multiplier,
            result.capped_n_control,
            result.capped_n_treatment,
            result.capped_total_n,
            result.achieved_power_at_capped_interim_sd * 100.0,
        )
    } else {
        format!(
            "Re-estimation inflates the planned per-arm sample size by a factor of {:.4}, \
             yielding {} control and {} treatment subjects (total N = {}).",
            result.variance_ratio,
            result.re_estimated_n_control,
            result.re_estimated_n_treatment,
            result.re_estimated_total_n,
        )
    };
    paragraphs.push(cap_sentence);

    paragraphs.push(format!(
        "The initial planned enrollment is {} control and {} treatment subjects (total N = {}) at \
         target power {:.0} % and α = {:.2}. The blinded SSR rule follows Friede and Kieser (2006); \
         operational Type I error control should be confirmed separately in the statistical \
         analysis plan.",
        result.planned_n_control,
        result.planned_n_treatment,
        result.planned_total_n,
        input.target_power * 100.0,
        input.alpha,
    ));

    join_paragraphs(paragraphs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::binary::one_sample_binomial::calculate as calculate_binomial;
    use crate::methods::continuous::two_sample_ttest::calculate as calculate_ttest;
    use crate::types::{Alternative, SolveMode};

    #[test]
    fn two_sample_ttest_protocol_matches_reference_example() {
        let input = TwoSampleTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            mean_difference: 2.0,
            standard_deviation: 2.0,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: Some(0.15),
        };
        let result = calculate_ttest(input.clone()).expect("calculate");
        let text = two_sample_ttest_protocol(&input, &result);

        assert!(text.contains(&format!("{} subjects per arm", result.n_control_adjusted)));
        assert!(text.contains(&format!("{} subjects in total", result.total_n_adjusted)));
        assert!(text.contains("Δ = 2.00"));
        assert!(text.contains("Cohen's d = 1.00"));
        assert!(text.contains("80 % power"));
        assert!(text.contains("zα/2 = 1.960"));
        assert!(text.contains("zβ = 0.842"));
        assert!(text.contains(&format!("evaluable per arm is {}", result.n_control)));
        assert!(text.contains("withdrawal rate of 15%"));
        assert!(text.contains("ICH E9"));
        assert!(text.contains("change from baseline"));
    }

    #[test]
    fn one_sample_binomial_protocol_matches_reference_example() {
        let input = OneSampleBinomialInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            n: None,
            reference_rate: 0.3,
            response_rate: 0.5,
            alternative: Alternative::Greater,
            dropout_rate: Some(0.15),
        };
        let result = calculate_binomial(input.clone()).expect("calculate");
        let text = one_sample_binomial_protocol(&input, &result);

        assert!(text.contains(&format!("{} subjects (single arm)", result.n_adjusted)));
        assert!(text.contains("p₀ = 30%"));
        assert!(text.contains("pA = 50%"));
        assert!(text.contains("risk difference 20%"));
        assert!(text.contains("Cohen's h = 0.41"));
        assert!(text.contains("80 % power"));
        assert!(text.contains("one-sided significance level"));
        assert!(text.contains("zα = 1.645"));
        assert!(text.contains("zβ = 0.842"));
        assert!(text.contains(&format!("evaluable subjects is {}", result.n)));
        assert!(text.contains("withdrawal rate of 15%"));
        assert!(text.contains("Fleiss arcsine"));
    }
}
