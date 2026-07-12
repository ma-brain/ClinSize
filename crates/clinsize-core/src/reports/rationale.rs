//! Narrative sample size calculation rationale for exports and UI display.

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

fn alternative_phrase(alternative: Alternative) -> &'static str {
    match alternative {
        Alternative::TwoSided => "two-sided",
        Alternative::Greater => "one-sided (treatment greater than control)",
        Alternative::Less => "one-sided (treatment less than control)",
    }
}

fn allocation_phrase(ratio: f64) -> String {
    if (ratio - 1.0).abs() < f64::EPSILON {
        "1:1 (equal allocation)".into()
    } else {
        format!("{ratio:.4} treatment subjects per control subject")
    }
}

fn alpha_phrase(alpha: f64, alternative: Alternative) -> String {
    let rate = format!("{alpha:.4}");
    match alternative {
        Alternative::TwoSided => format!("{rate} (two-sided)"),
        Alternative::Greater | Alternative::Less => format!("{rate} (one-sided)"),
    }
}

fn study_objective_phrase(objective: StudyObjective) -> &'static str {
    match objective {
        StudyObjective::Superiority => "superiority",
        StudyObjective::NonInferiority => "non-inferiority",
    }
}

fn multiplicity_method_phrase(method: MultiplicityMethod) -> &'static str {
    match method {
        MultiplicityMethod::Bonferroni => "Bonferroni",
        MultiplicityMethod::Sidak => "Šidák",
        MultiplicityMethod::Dunnett => "Dunnett",
        MultiplicityMethod::Holm => "Holm step-down gatekeeping",
        MultiplicityMethod::Hochberg => "Hochberg step-up gatekeeping",
        MultiplicityMethod::Graphical => "graphical gatekeeping",
    }
}

fn spending_function_phrase(spending: SpendingFunction) -> &'static str {
    match spending {
        SpendingFunction::ObrienFleming => "O'Brien-Fleming",
        SpendingFunction::Pocock => "Pocock",
    }
}

fn append_dropout_paragraph(
    paragraphs: &mut Vec<String>,
    dropout_rate: Option<f64>,
    n_control_adjusted: u32,
    n_treatment_adjusted: u32,
    total_n_adjusted: u32,
) {
    if let Some(rate) = dropout_rate {
        paragraphs.push(format!(
            "A uniform dropout rate of {:.0}% is applied after sample-size rounding. Per-group \
             sizes are inflated by 1/(1 − dropout) and rounded up, yielding {} control and {} \
             treatment subjects to randomize (total N = {}).",
            rate * 100.0,
            n_control_adjusted,
            n_treatment_adjusted,
            total_n_adjusted,
        ));
    }
}

/// Narrative rationale for a two-sample t-test calculation.
pub fn two_sample_ttest_rationale(
    input: &TwoSampleTTestInput,
    result: &TwoSampleTTestResult,
) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a two-group superiority design for a continuous endpoint, \
         comparing treatment and control using an equal-variance two-sample t-test. The assumed \
         treatment effect is a mean difference of {:.4} (treatment minus control), with a common \
         within-group standard deviation of {:.4}. The standardized effect size (Cohen's d) is {:.4}.",
        input.mean_difference,
        input.standard_deviation,
        result.effect_size,
    ));

    paragraphs.push(format!(
        "Statistical power is derived from the noncentral t distribution with degrees of freedom \
         ν = n_control + n_treatment − 2. The Type I error rate is α = {}. The alternative \
         hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize identifies \
                 the smallest integer control-group size such that the rounded treatment-group size \
                 (n_treatment = ⌈n_control × allocation ratio⌉) achieves at least the target power.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
            ));
            paragraphs.push(format!(
                "The resulting per-group sample sizes are {} control and {} treatment subjects \
                 (total N = {}). After integer rounding, the achieved power is {:.2}%.",
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed control-group size of {} and allocation ratio of {}, the rounded \
                 treatment-group size is {} (total N = {}). The achieved power under the stated \
                 assumptions is {:.2}%.",
                control_n,
                allocation_phrase(input.allocation_ratio),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for two-sample t-test"),
    }

    append_dropout_paragraph(
        &mut paragraphs,
        input.dropout_rate,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
    );

    paragraphs.push(
        "The method assumes independent observations, approximately normal endpoint distributions \
         (or adequate large-sample behavior), and a common within-group variance across arms. \
         When multiple endpoints or comparisons are tested, replace the nominal α above with the \
         per-comparison alpha from a multiplicity adjustment. Dropout inflation, when used, \
         applies the same rate to both arms and does not model differential dropout."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a one-sample t-test calculation.
pub fn one_sample_ttest_rationale(
    input: &OneSampleTTestInput,
    result: &OneSampleTTestResult,
) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a single-group continuous endpoint tested against a reference \
         mean using a one-sample t-test. The assumed mean difference from the reference is {:.4}, \
         with a within-subject standard deviation of {:.4}. The standardized effect size (Cohen's d) \
         is {:.4}.",
        input.mean_difference,
        input.standard_deviation,
        result.effect_size,
    ));

    paragraphs.push(format!(
        "Statistical power is derived from the noncentral t distribution with degrees of freedom \
         ν = n − 1. The Type I error rate is α = {}. The alternative hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative).replace("treatment", "the sample mean"),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. ClinSize identifies the smallest integer sample size \
                 n such that the achieved power is at least the target after rounding up.",
                target * 100.0,
            ));
            paragraphs.push(format!(
                "The resulting sample size is {} subjects. After integer rounding, the achieved \
                 power is {:.2}%.",
                result.n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let n = input.n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed sample size of {} subjects, the achieved power under the stated \
                 assumptions is {:.2}%.",
                n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for one-sample t-test"),
    }

    if let Some(rate) = input.dropout_rate {
        paragraphs.push(format!(
            "A uniform dropout rate of {:.0}% is applied after sample-size rounding. The sample \
             size is inflated by 1/(1 − dropout) and rounded up, yielding {} subjects to \
             randomize.",
            rate * 100.0,
            result.n_adjusted,
        ));
    }

    paragraphs.push(
        "The method assumes independent observations and an approximately normal endpoint \
         distribution (or adequate large-sample behavior). When multiple endpoints are tested, \
         replace the nominal α above with the per-comparison alpha from a multiplicity adjustment."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a paired t-test calculation.
pub fn paired_ttest_rationale(input: &PairedTTestInput, result: &PairedTTestResult) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a paired or repeated-measures continuous endpoint using a \
         paired t-test on within-subject differences. The assumed mean paired difference is \
         {:.4}, with a standard deviation of paired differences of {:.4}. The standardized effect \
         size (Cohen's d) is {:.4}.",
        input.mean_difference, input.standard_deviation, result.effect_size,
    ));

    paragraphs.push(format!(
        "Statistical power is derived from the noncentral t distribution applied to the paired \
         differences with degrees of freedom ν = n_pairs − 1. The Type I error rate is α = {}. \
         The alternative hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative).replace("treatment", "the mean difference"),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. ClinSize identifies the smallest integer number of \
                 pairs such that the achieved power is at least the target after rounding up.",
                target * 100.0,
            ));
            paragraphs.push(format!(
                "The resulting number of pairs is {}. After integer rounding, the achieved power \
                 is {:.2}%.",
                result.n_pairs,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let n_pairs = input.n_pairs.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed number of {} pairs, the achieved power under the stated assumptions \
                 is {:.2}%.",
                n_pairs,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for paired t-test"),
    }

    if let Some(rate) = input.dropout_rate {
        paragraphs.push(format!(
            "A uniform dropout rate of {:.0}% is applied after sample-size rounding. The number \
             of pairs is inflated by 1/(1 − dropout) and rounded up, yielding {} pairs to enroll.",
            rate * 100.0,
            result.n_pairs_adjusted,
        ));
    }

    paragraphs.push(
        "The method assumes paired differences are independent across subjects and approximately \
         normally distributed (or adequate large-sample behavior). Dropout inflation, when used, \
         applies a uniform rate and does not model differential attrition between paired visits."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a balanced one-way ANOVA calculation.
pub fn one_way_anova_rationale(input: &OneWayAnovaInput, result: &OneWayAnovaResult) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a balanced multi-group continuous endpoint design with {} \
         groups, using a one-way ANOVA F-test. The assumed between-group variance component is \
         {:.4} and the within-group variance component is {:.4}. The standardized effect size \
         (Cohen's f) is {:.4}.",
        input.n_groups, input.between_variance, input.within_variance, result.effect_size,
    ));

    paragraphs.push(format!(
        "Statistical power is derived from the noncentral F distribution with numerator degrees \
         of freedom k − 1 = {} and denominator degrees of freedom k(n − 1), where k is the \
         number of groups and n is the per-group sample size. The Type I error rate is α = {:.4} \
         (two-sided omnibus test).",
        input.n_groups - 1,
        input.alpha,
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. ClinSize identifies the smallest integer per-group \
                 sample size such that the achieved power is at least the target after rounding up.",
                target * 100.0,
            ));
            paragraphs.push(format!(
                "The resulting per-group sample size is {} (total N = {}). After integer rounding, \
                 the achieved power is {:.2}%.",
                result.n_per_group,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let n_per_group = input.n_per_group.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed per-group sample size of {} (total N = {}), the achieved power \
                 under the stated assumptions is {:.2}%.",
                n_per_group,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for one-way ANOVA"),
    }

    if let Some(rate) = input.dropout_rate {
        paragraphs.push(format!(
            "A uniform dropout rate of {:.0}% is applied after sample-size rounding. Per-group \
             sizes are inflated by 1/(1 − dropout) and rounded up, yielding {} subjects per group \
             (total N = {}).",
            rate * 100.0,
            result.n_per_group_adjusted,
            result.total_n_adjusted,
        ));
    }

    paragraphs.push(
        "The method assumes independent observations, equal per-group sample sizes, and \
         approximately normal within-group distributions with a common variance. Post-hoc pairwise \
         comparisons require a separate multiplicity adjustment beyond this omnibus calculation."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a two-way ANOVA calculation.
pub fn two_way_anova_rationale(input: &TwoWayAnovaInput, result: &TwoWayAnovaResult) -> String {
    let mut paragraphs = Vec::new();

    let effect_label = match input.primary_effect {
        AnovaEffect::MainA => "main effect of factor A",
        AnovaEffect::MainB => "main effect of factor B",
        AnovaEffect::Interaction => "A × B interaction",
    };

    paragraphs.push(format!(
        "This calculation addresses a balanced two-way ANOVA with factor A at {} levels and \
         factor B at {} levels. The sample size is driven by the {effect_label}, using exact \
         noncentral-F power. Variance components are σ²_A = {:.4}, σ²_B = {:.4}, σ²_AB = {:.4}, \
         and within-cell error σ²_error = {:.4}.",
        input.n_levels_a,
        input.n_levels_b,
        input.variance_a,
        input.variance_b,
        input.variance_interaction,
        input.within_variance,
    ));

    paragraphs.push(format!(
        "The Type I error rate is α = {:.4}. Cohen's f for the primary effect is {:.4}.",
        input.alpha, result.effect_size,
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}% for the {effect_label}. ClinSize searches for the \
                 smallest integer per-cell sample size n meeting that power via the noncentral-F \
                 distribution.",
                target * 100.0,
            ));
            paragraphs.push(format!(
                "The resulting per-cell size is n = {} (total N = {}). After integer rounding, \
                 the achieved power is {:.2}%.",
                result.n_per_cell,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let n = input.n_per_cell.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed per-cell size of n = {} (total N = {}), the achieved power for the \
                 {effect_label} is {:.2}%.",
                n,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for two-way ANOVA"),
    }

    if let Some(rate) = input.dropout_rate {
        paragraphs.push(format!(
            "A uniform dropout rate of {:.0}% is applied after sample-size rounding. Per-cell \
             sizes are inflated by 1/(1 − dropout) and rounded up, yielding {} subjects per cell \
             (total N = {}).",
            rate * 100.0,
            result.n_per_cell_adjusted,
            result.total_n_adjusted,
        ));
    }

    paragraphs.push(
        "The method assumes independent observations, equal per-cell sample sizes, and \
         approximately normal within-cell distributions with a common variance. Power is reported \
         only for the selected primary effect; the other two effects may have different power at \
         this sample size."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a two-sample ANCOVA calculation.
pub fn ancova_two_sample_rationale(
    input: &AncovaTwoSampleInput,
    result: &AncovaTwoSampleResult,
) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a two-group parallel continuous endpoint with one baseline \
         covariate, using an approximate ANCOVA variance reduction. The assumed treatment effect is \
         a mean difference of {:.4} (treatment minus control). The unadjusted outcome standard \
         deviation is {:.4}; with baseline-outcome correlation ρ = {:.4}, the adjusted standard \
         deviation is {:.4} (variance reduction factor 1 − ρ² = {:.4}). Cohen's d using the \
         unadjusted SD is {:.4}.",
        input.mean_difference,
        input.standard_deviation,
        input.baseline_outcome_correlation,
        result.adjusted_standard_deviation,
        result.variance_reduction_factor,
        result.effect_size,
    ));

    paragraphs.push(format!(
        "Sample size is computed via the equal-variance two-sample t-test using the adjusted \
         standard deviation σ_adj = σ_y × √(1 − ρ²). Statistical power follows the noncentral t \
         distribution. The Type I error rate is α = {}. The alternative hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize identifies \
                 the smallest integer control-group size such that the rounded treatment-group size \
                 achieves at least the target power under the adjusted variance.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
            ));
            paragraphs.push(format!(
                "The resulting per-group sample sizes are {} control and {} treatment subjects \
                 (total N = {}). After integer rounding, the achieved power is {:.2}%.",
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed control-group size of {} and allocation ratio of {}, the rounded \
                 treatment-group size is {} (total N = {}). The achieved power under the stated \
                 assumptions is {:.2}%.",
                control_n,
                allocation_phrase(input.allocation_ratio),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for ANCOVA"),
    }

    append_dropout_paragraph(
        &mut paragraphs,
        input.dropout_rate,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
    );

    paragraphs.push(
        "This is an approximate single-covariate adjustment, not a full model-based ANCOVA. It \
         assumes independent observations, a common within-group variance, and that the baseline \
         covariate explains outcome variability through the stated correlation. When multiple \
         endpoints are tested, replace the nominal α with a multiplicity-adjusted per-comparison \
         alpha."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a change-from-baseline calculation.
pub fn change_from_baseline_rationale(
    input: &ChangeFromBaselineInput,
    result: &ChangeFromBaselineResult,
) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a two-group parallel continuous endpoint analyzed on \
         change-from-baseline (CFB) scores. The assumed treatment effect is a mean CFB difference \
         of {:.4} (treatment minus control). The common outcome standard deviation is {:.4}; with \
         baseline-outcome correlation ρ = {:.4}, the change-score standard deviation is {:.4} \
         (σ_cfb = σ × √(2(1 − ρ))). Cohen's d using the unadjusted SD is {:.4}.",
        input.mean_difference,
        input.standard_deviation,
        input.baseline_outcome_correlation,
        result.change_score_standard_deviation,
        result.effect_size,
    ));

    paragraphs.push(format!(
        "Sample size is computed via the equal-variance two-sample t-test using σ_cfb. Statistical \
         power follows the noncentral t distribution. The Type I error rate is α = {}. The \
         alternative hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize identifies \
                 the smallest integer control-group size such that the rounded treatment-group size \
                 achieves at least the target power under the change-score variance.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
            ));
            paragraphs.push(format!(
                "The resulting per-group sample sizes are {} control and {} treatment subjects \
                 (total N = {}). After integer rounding, the achieved power is {:.2}%.",
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed control-group size of {} and allocation ratio of {}, the rounded \
                 treatment-group size is {} (total N = {}). The achieved power under the stated \
                 assumptions is {:.2}%.",
                control_n,
                allocation_phrase(input.allocation_ratio),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for CFB"),
    }

    append_dropout_paragraph(
        &mut paragraphs,
        input.dropout_rate,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
    );

    paragraphs.push(
        "This method assumes normality of CFB scores, equal baseline and follow-up SD, and equal \
         correlation across arms. It does not model missing data patterns beyond a uniform dropout \
         inflation factor."
            .into(),
    );

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

/// Narrative rationale for an MMRM calculation.
pub fn mmrm_rationale(input: &MmrmInput, result: &MmrmResult) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a two-group parallel longitudinal continuous endpoint analyzed \
         with a mixed model for repeated measures (MMRM). The assumed treatment effect at the \
         final post-baseline visit is δ = {:.4} (treatment minus control). The final-visit \
         standard deviation is σ = {:.4}, with {} correlation structure (ρ = {:.4}) across k = {} \
         post-baseline visits. Under the Lu, Luo & Chen (2008) method the variance factor for the \
         final-visit contrast is φ = {:.4} (φ = 1 with complete data).",
        input.treatment_effect,
        input.residual_standard_deviation,
        correlation_structure_label(input.correlation_structure),
        input.correlation,
        input.n_post_baseline_visits,
        result.variance_factor,
    ));

    paragraphs.push(format!(
        "Sample size uses the normal approximation n_arm = (z_α + z_β)² × φ × σ² × \
         (1 + 1/λ) / δ² for the treatment arm (λ = allocation ratio), per Lu, Luo & Chen (2008) \
         and R longpower::power.mmrm. The Type I error rate is α = {}. The alternative \
         hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize identifies \
                 the smallest integer randomized control-group size such that the rounded \
                 treatment-group size achieves at least the target power.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
            ));
            paragraphs.push(format!(
                "The resulting randomized per-group sample sizes are {} control and {} treatment \
                 subjects (total N = {}). After integer rounding, the achieved power is {:.2}%.",
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed randomized control-group size of {} and allocation ratio of {}, the \
                 rounded treatment-group size is {} (total N = {}). The achieved power under the \
                 stated assumptions is {:.2}%.",
                control_n,
                allocation_phrase(input.allocation_ratio),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for MMRM"),
    }

    if let Some(rate) = input.per_visit_dropout_rate {
        paragraphs.push(format!(
            "A per-visit dropout rate of {:.0}% over {} visits yields final-visit retention \
             {:.1}% (cumulative dropout {:.1}%). Dropout is modeled as monotone missingness \
             inside the MMRM variance: subjects who discontinue still contribute their observed \
             visits, so the reported sample sizes are the numbers to randomize and no separate \
             enrollment inflation applies.",
            rate * 100.0,
            input.n_post_baseline_visits,
            result.final_retention * 100.0,
            result.cumulative_dropout * 100.0,
        ));
    }

    paragraphs.push(
        "This method assumes MMRM with visit as a categorical factor, a single-ρ compound \
         symmetry or AR(1) within-subject correlation, equal variance, correlation, and \
         retention across arms, and monotone dropout with a constant per-visit rate."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a negative binomial calculation.
pub fn negative_binomial_rationale(
    input: &NegativeBinomialInput,
    result: &NegativeBinomialResult,
) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a two-group comparison of recurrent event counts using a \
         negative binomial model with NB2 variance Var(Y) = μ + kμ². The control event rate is \
         λ₁ = {:.4} and the treatment rate is λ₂ = {:.4} (rate ratio {:.4}) over exposure time \
         {:.4}. The common dispersion parameter is k = {:.4}.",
        input.control_rate,
        input.treatment_rate,
        result.rate_ratio,
        input.exposure_time,
        input.dispersion,
    ));

    paragraphs.push(format!(
        "Sample size follows Zhu & Lakkis (2014) Wald test for the log rate ratio. The Type I \
         error rate is α = {}. The alternative hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize computes \
                 control-group size from the closed-form formula and verifies power by iterative \
                 check after rounding treatment-group size.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
            ));
            paragraphs.push(format!(
                "The resulting sample sizes are {} control and {} treatment subjects (total N = \
                 {}). After integer rounding, the achieved power is {:.2}%.",
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed control-group size of {} and allocation ratio of {}, the rounded \
                 treatment-group size is {} (total N = {}). The achieved power is {:.2}%.",
                control_n,
                allocation_phrase(input.allocation_ratio),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for negative binomial"),
    }

    append_dropout_paragraph(
        &mut paragraphs,
        input.dropout_rate,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a Poisson calculation.
pub fn poisson_rationale(input: &PoissonInput, result: &PoissonResult) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a two-group comparison of event counts using a Poisson model \
         with variance Var(Y) = μ (no overdispersion). The control event rate is λ₁ = {:.4} and \
         the treatment rate is λ₂ = {:.4} (rate ratio {:.4}) over exposure time {:.4}.",
        input.control_rate, input.treatment_rate, result.rate_ratio, input.exposure_time,
    ));

    paragraphs.push(format!(
        "Sample size follows the Signorini (1991) Wald test for the log rate ratio. The Type I \
         error rate is α = {}. The alternative hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize computes \
                 control-group size from the closed-form formula and verifies power by iterative \
                 check after rounding treatment-group size.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
            ));
            paragraphs.push(format!(
                "The resulting sample sizes are {} control and {} treatment subjects (total N = \
                 {}). After integer rounding, the achieved power is {:.2}%.",
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed control-group size of {} and allocation ratio of {}, the rounded \
                 treatment-group size is {} (total N = {}). The achieved power is {:.2}%.",
                control_n,
                allocation_phrase(input.allocation_ratio),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for Poisson"),
    }

    append_dropout_paragraph(
        &mut paragraphs,
        input.dropout_rate,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
    );

    join_paragraphs(paragraphs)
}
pub fn proportional_odds_rationale(
    input: &ProportionalOddsInput,
    result: &ProportionalOddsResult,
) -> String {
    let mut paragraphs = Vec::new();
    let probs: Vec<String> = input
        .category_probabilities
        .iter()
        .map(|p| format!("{p:.4}"))
        .collect();

    paragraphs.push(format!(
        "This calculation addresses a two-group ordinal endpoint under the proportional odds \
         model (Whitehead 1993). Control-group category probabilities (best to worst) are [{}]. \
         The target odds ratio is {:.4} with treatment fraction {:.4}. The efficiency factor \
         ps = 1 − Σpᵢ³ = {:.4}.",
        probs.join(", "),
        input.odds_ratio,
        input.treatment_fraction,
        result.efficiency,
    ));

    paragraphs.push(format!(
        "Sample size uses the Hmisc `posamsize` formula with a two-sided Type I error rate α = \
         {:.4}. Power is evaluated via the Hmisc `popower` variance approximation.",
        input.alpha,
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. ClinSize computes continuous total N from the \
                 Whitehead formula, rounds up, and allocates subjects by treatment fraction.",
                target * 100.0,
            ));
            paragraphs.push(format!(
                "The resulting sample sizes are {} control and {} treatment subjects (total N = \
                 {}). After integer rounding, the achieved power is {:.2}%.",
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed control-group size of {} (treatment fraction {:.4}), the treatment \
                 group size is {} (total N = {}). The achieved power is {:.2}%.",
                control_n,
                input.treatment_fraction,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for proportional odds"),
    }

    append_dropout_paragraph(
        &mut paragraphs,
        input.dropout_rate,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a two-proportion difference calculation.
pub fn two_proportion_difference_rationale(
    input: &TwoProportionDifferenceInput,
    result: &TwoProportionDifferenceResult,
) -> String {
    let mut paragraphs = Vec::new();

    let objective = study_objective_phrase(input.study_objective);
    paragraphs.push(format!(
        "This calculation addresses a two-group {} design for a binary endpoint, comparing event \
         rates between treatment and control using a normal approximation to the difference in \
         proportions. The assumed control event rate is {:.4} and the treatment event rate is {:.4} \
         (rate difference {:.4}).",
        objective,
        input.control_rate,
        input.treatment_rate,
        result.rate_difference,
    ));

    if input.study_objective == StudyObjective::NonInferiority {
        let margin = input
            .noninferiority_margin
            .expect("validated for non-inferiority");
        paragraphs.push(format!(
            "The non-inferiority margin is {:.4} (treatment rate may be up to this amount below \
             control). The Type I error rate is α = {}. The alternative hypothesis is {}.",
            margin,
            alpha_phrase(input.alpha, input.alternative),
            alternative_phrase(input.alternative),
        ));
    } else {
        paragraphs.push(format!(
            "The Type I error rate is α = {}. The alternative hypothesis is {}.",
            alpha_phrase(input.alpha, input.alternative),
            alternative_phrase(input.alternative),
        ));
    }

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize identifies \
                 the smallest integer control-group size such that the rounded treatment-group size \
                 achieves at least the target power.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
            ));
            paragraphs.push(format!(
                "The resulting per-group sample sizes are {} control and {} treatment subjects \
                 (total N = {}). After integer rounding, the achieved power is {:.2}%.",
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let control_n = input.control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed control-group size of {} and allocation ratio of {}, the rounded \
                 treatment-group size is {} (total N = {}). The achieved power under the stated \
                 assumptions is {:.2}%.",
                control_n,
                allocation_phrase(input.allocation_ratio),
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => {
            unreachable!("not implemented for two-proportion difference")
        }
    }

    append_dropout_paragraph(
        &mut paragraphs,
        input.dropout_rate,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
    );

    paragraphs.push(
        "The method assumes independent binomial observations and adequate sample size for the \
         normal approximation. When multiple binary endpoints are tested, replace the nominal α \
         with a multiplicity-adjusted per-comparison alpha."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for an odds-ratio superiority calculation.
pub fn odds_ratio_rationale(input: &OddsRatioInput, result: &OddsRatioResult) -> String {
    binary_effect_rationale(
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

/// Narrative rationale for a risk-ratio superiority calculation.
pub fn risk_ratio_rationale(input: &RiskRatioInput, result: &RiskRatioResult) -> String {
    binary_effect_rationale(
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
fn binary_effect_rationale(
    effect_label: &str,
    effect_value: f64,
    test_statistic: &str,
    solve_mode: SolveMode,
    alpha: f64,
    power: Option<f64>,
    control_n: Option<u32>,
    control_rate: f64,
    treatment_rate: f64,
    allocation_ratio: f64,
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

    paragraphs.push(format!(
        "This calculation addresses a two-group superiority design for a binary endpoint, testing \
         the {effect_label} using a normal approximation to the {test_statistic}. The assumed \
         control event rate is {control_rate:.4} and the treatment event rate is {treatment_rate:.4}. \
         The resulting {effect_label} is {effect_value:.4}."
    ));

    paragraphs.push(format!(
        "The Type I error rate is α = {}. The alternative hypothesis is {}.",
        alpha_phrase(alpha, alternative),
        alternative_phrase(alternative),
    ));

    match solve_mode {
        SolveMode::SampleSize => {
            let target = power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize identifies \
                 the smallest integer control-group size such that the rounded treatment-group size \
                 achieves at least the target power.",
                target * 100.0,
                allocation_phrase(allocation_ratio),
            ));
            paragraphs.push(format!(
                "The resulting per-group sample sizes are {} control and {} treatment subjects \
                 (total N = {}). After integer rounding, the achieved power is {:.2}%.",
                n_control,
                n_treatment,
                total_n,
                achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let fixed_control_n = control_n.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed control-group size of {} and allocation ratio of {}, the rounded \
                 treatment-group size is {} (total N = {}). The achieved power under the stated \
                 assumptions is {:.2}%.",
                fixed_control_n,
                allocation_phrase(allocation_ratio),
                n_treatment,
                total_n,
                achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for binary effect measures"),
    }

    append_dropout_paragraph(
        &mut paragraphs,
        dropout_rate,
        n_control_adjusted,
        n_treatment_adjusted,
        total_n_adjusted,
    );

    paragraphs.push(format!(
        "The method assumes independent binomial observations and adequate sample size for the \
         normal approximation to the {test_statistic}. When multiple binary endpoints are tested, \
         replace the nominal α with a multiplicity-adjusted per-comparison alpha."
    ));

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a two-arm log-rank calculation.
pub fn log_rank_rationale(input: &LogRankInput, result: &LogRankResult) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation addresses a two-arm superiority survival design using the log-rank test \
         (Schoenfeld approximation). The assumed hazard ratio (treatment / control) is {:.4}. \
         Statistical power depends on the total number of events, not enrolled subjects alone.",
        input.hazard_ratio,
    ));

    paragraphs.push(format!(
        "The Type I error rate is α = {}. The alternative hypothesis is {}.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));

    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated for sample size mode");
            paragraphs.push(format!(
                "The target power is {:.0}%. With an allocation ratio of {}, ClinSize identifies \
                 the smallest integer total event count such that the split between arms achieves \
                 at least the target power.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
            ));
            paragraphs.push(format!(
                "The required total events are {}, with expected events of {} in control and {} in \
                 treatment. After integer rounding, the achieved power is {:.2}%.",
                result.required_events,
                result.events_control,
                result.events_treatment,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            let total_events = input.total_events.expect("validated for power mode");
            paragraphs.push(format!(
                "For a fixed total of {} events with allocation ratio {}, the expected split is \
                 {} control and {} treatment events. The achieved power under the stated \
                 assumptions is {:.2}%.",
                total_events,
                allocation_phrase(input.allocation_ratio),
                result.events_control,
                result.events_treatment,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!("not implemented for log-rank"),
    }

    if let (Some(total_n), Some(n_control), Some(n_treatment)) =
        (result.total_n, result.n_control, result.n_treatment)
    {
        paragraphs.push(format!(
            "Under the stated accrual and follow-up assumptions, the required enrollment is {} \
             control and {} treatment subjects (total N = {}).",
            n_control, n_treatment, total_n
        ));
        if let (Some(p_control), Some(p_treatment)) = (
            result.probability_event_control,
            result.probability_event_treatment,
        ) {
            paragraphs.push(format!(
                "The implied event probabilities are {:.4} in control and {:.4} in treatment.",
                p_control, p_treatment
            ));
        }
    }

    paragraphs.push(
        "The method assumes proportional hazards, independent censoring, and the Schoenfeld \
         large-sample approximation. Accrual-based enrollment sizing uses uniform accrual and \
         exponential event hazards; replace the nominal α with a multiplicity-adjusted alpha when \
         multiple survival endpoints are tested."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a family-wise alpha adjustment.
pub fn multiplicity_rationale(input: &MultiplicityInput, result: &MultiplicityResult) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation converts a family-wise Type I error rate into a per-comparison alpha for \
         sample size planning across {} comparisons. The family-wise α is {:.6}. The selected \
         adjustment is {}.",
        input.number_of_comparisons,
        input.family_wise_alpha,
        multiplicity_method_phrase(input.adjustment_method),
    ));

    match input.adjustment_method {
        MultiplicityMethod::Bonferroni => {
            paragraphs.push(
                "Bonferroni divides the family-wise alpha equally across all comparisons, \
                 controlling the family-wise error rate under any dependence structure."
                    .into(),
            );
        }
        MultiplicityMethod::Sidak => {
            paragraphs.push(
                "Šidák adjustment assumes independent comparisons and solves \
                 1 − (1 − α_adj)^m = α_family for the per-comparison alpha."
                    .into(),
            );
        }
        MultiplicityMethod::Dunnett => {
            paragraphs.push(
                "Dunnett adjustment applies to multiple treatment arms compared with a common \
                 control under equal group sizes, using an equicorrelated multivariate normal \
                 approximation."
                    .into(),
            );
        }
        MultiplicityMethod::Holm | MultiplicityMethod::Hochberg | MultiplicityMethod::Graphical => {
            let gate = input.gate_position.expect("validated for gatekeeping");
            paragraphs.push(format!(
                "Gatekeeping is applied at position {} in a fixed-order hypothesis sequence of \
                 length {}.",
                gate, input.number_of_comparisons
            ));
            if input.adjustment_method == MultiplicityMethod::Graphical {
                if let Some(weight) = result.comparison_weight {
                    paragraphs.push(format!(
                        "The normalized alpha weight at this gate position is {:.6}.",
                        weight
                    ));
                }
            }
        }
    }

    paragraphs.push(format!(
        "The adjusted per-comparison alpha is {:.6}, representing {:.2}% of the naive \
         per-comparison rate (family-wise α / m). Use this value as the `alpha` input in endpoint \
         sample size calculations for the comparison at the gate position.",
        result.adjusted_alpha,
        result.alpha_reduction_factor * 100.0,
    ));

    paragraphs.push(
        "This adjustment addresses multiplicity in hypothesis testing; it does not by itself \
         inflate sample size. Apply the adjusted alpha when computing power for each comparison \
         that remains in the family."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a group sequential design.
pub fn group_sequential_rationale(
    input: &GroupSequentialInput,
    result: &GroupSequentialResult,
) -> String {
    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation plans a group sequential design with {} equally spaced interim looks \
         (including the final analysis) using the {} spending function. The one-sided family-wise \
         Type I error rate spent on the efficacy boundary is {:.6} and the target power is {:.0}%.",
        input.number_of_looks,
        spending_function_phrase(input.spending_function),
        input.alpha,
        input.target_power * 100.0,
    ));

    paragraphs.push(format!(
        "Interim efficacy boundaries are derived from the alpha spending increments at each \
         information fraction. The fixed-design drift at the nominal alpha is {:.4}; the group \
         sequential design requires drift {:.4} to achieve the target power.",
        result.fixed_design_drift, result.required_drift,
    ));

    let final_look = result.looks.last().expect("at least one look");
    paragraphs.push(format!(
        "At the final look (information {:.0}%), the upper Z boundary is {:.4} and cumulative α \
         spent is {:.6}. The achieved power under this boundary set is {:.2}%.",
        final_look.information_fraction * 100.0,
        final_look.upper_z_boundary,
        final_look.cumulative_alpha_spent,
        result.achieved_power * 100.0,
    ));

    paragraphs.push(format!(
        "The sample size inflation factor is {:.4}. Multiply a fixed-design sample size by this \
         factor to obtain the maximum sample size under this group sequential plan.",
        result.sample_size_inflation_factor,
    ));

    paragraphs.push(
        "This summary provides planning boundaries and inflation; it does not simulate operational \
         characteristics under futility stopping or sample size adaptation. Apply the inflation \
         factor to the fixed-design sample size from the relevant endpoint calculation."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for blinded sample size re-estimation.
pub fn blinded_ssre_rationale(input: &BlindedSsreInput, result: &BlindedSsreResult) -> String {
    let blinded_sd = input
        .blinded_interim_standard_deviation
        .unwrap_or(input.planned_standard_deviation);

    let mut paragraphs = Vec::new();

    paragraphs.push(format!(
        "This calculation plans blinded sample size re-estimation (SSR) for a continuous two-group \
         superiority design using an equal-variance two-sample t-test. The planned treatment effect \
         is a mean difference of {:.4} with assumed common SD σ₀ = {:.4}. The blinded interim \
         pooled SD is s_b = {:.4}.",
        input.mean_difference,
        input.planned_standard_deviation,
        blinded_sd,
    ));

    paragraphs.push(format!(
        "The Type I error rate is α = {}. The alternative hypothesis is {}. The target power for \
         the initial plan is {:.0}%.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
        input.target_power * 100.0,
    ));

    paragraphs.push(format!(
        "The planned per-group sample sizes are {} control and {} treatment subjects (total N = \
         {}). At interim fraction {:.0}% of planned enrollment, {} control and {} treatment \
         subjects are expected to have been observed.",
        result.planned_n_control,
        result.planned_n_treatment,
        result.planned_total_n,
        input.interim_fraction * 100.0,
        result.interim_n_control,
        result.interim_n_treatment,
    ));

    paragraphs.push(format!(
        "The variance ratio (s_b/σ₀)² is {:.4}. Re-estimation inflates the planned per-arm sample \
         size by this factor, yielding {} control and {} treatment subjects (total N = {}).",
        result.variance_ratio,
        result.re_estimated_n_control,
        result.re_estimated_n_treatment,
        result.re_estimated_total_n,
    ));

    if result.was_capped {
        paragraphs.push(format!(
            "A pre-specified cap of {:.1}× the planned per-arm size was applied. The capped \
             enrollment is {} control and {} treatment subjects (total N = {}), with inflation \
             factor {:.4}. Achieved power at the capped allocation is {:.2}% under the planned \
             SD, but only {:.2}% under the blinded interim SD — the realistic estimate given \
             that the interim data motivated the re-estimation. The cap therefore leaves the \
             design underpowered if the interim SD reflects the true variability.",
            input.max_sample_size_multiplier,
            result.capped_n_control,
            result.capped_n_treatment,
            result.capped_total_n,
            result.capped_inflation_factor,
            result.achieved_power_at_capped * 100.0,
            result.achieved_power_at_capped_interim_sd * 100.0,
        ));
    } else {
        paragraphs.push(format!(
            "No cap was applied (maximum multiplier {:.1}×). Achieved power at the re-estimated \
             allocation is {:.2}% under the blinded interim SD ({:.2}% under the planned SD).",
            input.max_sample_size_multiplier,
            result.achieved_power_at_capped_interim_sd * 100.0,
            result.achieved_power_at_capped * 100.0,
        ));
    }

    paragraphs.push(
        "The Friede-Kieser blinded SSR rule holds the planned treatment effect fixed and updates \
         only the variance estimate from blinded pooled data at one interim look. It does not \
         simulate Type I error inflation from re-estimation; operational characteristics should be \
         validated separately for the final protocol."
            .into(),
    );

    join_paragraphs(paragraphs)
}

/// Narrative rationale for a one-sample binomial calculation.
pub fn one_sample_binomial_rationale(
    input: &OneSampleBinomialInput,
    result: &OneSampleBinomialResult,
) -> String {
    let mut paragraphs = vec![format!(
        "This calculation addresses a single-arm binary endpoint, testing whether the response \
         rate differs from a reference proportion of {:.0}%. The hypothesized response rate is \
         {:.0}% (difference {:.0} percentage points).",
        input.reference_rate * 100.0,
        input.response_rate * 100.0,
        result.rate_difference * 100.0,
    )];
    paragraphs.push(format!(
        "Sample size uses a normal approximation to the binomial distribution with Type I error \
         α = {} and a {} alternative.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));
    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated");
            paragraphs.push(format!(
                "The target power is {:.0}%. The required sample size is {} subjects. After \
                 rounding, achieved power is {:.2}%.",
                target * 100.0,
                result.n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "For {} subjects, the achieved power is {:.2}%.",
                input.n.expect("validated"),
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!(),
    }
    if let Some(rate) = input.dropout_rate {
        paragraphs.push(format!(
            "A uniform dropout rate of {:.0}% is applied after sample-size rounding. The sample \
             size is inflated by 1/(1 − dropout) and rounded up, yielding {} subjects to \
             randomize.",
            rate * 100.0,
            result.n_adjusted,
        ));
    }
    paragraphs.push(
        "The method assumes independent Bernoulli outcomes and a fixed reference rate. Exact \
         binomial tests are not implemented in this release."
            .into(),
    );
    join_paragraphs(paragraphs)
}

/// Narrative rationale for a Mann-Whitney U calculation.
pub fn mann_whitney_rationale(input: &MannWhitneyInput, result: &MannWhitneyResult) -> String {
    let mut paragraphs = vec![format!(
        "This calculation addresses a two-group nonparametric comparison using the Mann-Whitney U \
         (Wilcoxon rank-sum) test. The assumed location shift is {:.4} on the continuous scale \
         with common SD {:.4} (Cohen's d = {:.4}), corresponding to P(treatment > control) = {:.4} \
         under equal-variance normality.",
        input.mean_difference,
        input.standard_deviation,
        result.effect_size,
        result.probability_superiority,
    )];
    paragraphs.push(format!(
        "Sample size follows Noether (1987) with Type I error α = {} and a {} alternative.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));
    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated");
            paragraphs.push(format!(
                "The target power is {:.0}%. With allocation ratio {}, the required sample sizes \
                 are {} control and {} treatment (total N = {}). Achieved power is {:.2}%.",
                target * 100.0,
                allocation_phrase(input.allocation_ratio),
                result.n_control,
                result.n_treatment,
                result.total_n,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "For control N = {} and allocation ratio {}, achieved power is {:.2}% (total N = {}).",
                input.control_n.expect("validated"),
                allocation_phrase(input.allocation_ratio),
                result.achieved_power * 100.0,
                result.total_n,
            ));
        }
        SolveMode::DetectableEffect => unreachable!(),
    }
    append_dropout_paragraph(
        &mut paragraphs,
        input.dropout_rate,
        result.n_control_adjusted,
        result.n_treatment_adjusted,
        result.total_n_adjusted,
    );
    paragraphs.push(
        "The method assumes continuous data without ties and uses a normal approximation to the \
         Mann-Whitney statistic. It is the nonparametric counterpart to the two-sample t-test."
            .into(),
    );
    join_paragraphs(paragraphs)
}

/// Narrative rationale for a Wilcoxon signed-rank calculation.
pub fn wilcoxon_signed_rank_rationale(
    input: &WilcoxonSignedRankInput,
    result: &WilcoxonSignedRankResult,
) -> String {
    let mut paragraphs = vec![format!(
        "This calculation addresses a within-subject nonparametric comparison using the Wilcoxon \
         signed-rank test. The expected paired difference is {:.4} with SD {:.4} (Cohen's d = {:.4}), \
         giving P(difference > 0) = {:.4} under normality.",
        input.mean_difference,
        input.standard_deviation,
        result.effect_size,
        result.probability_positive_difference,
    )];
    paragraphs.push(format!(
        "Sample size follows Noether (1987) with Type I error α = {} and a {} alternative.",
        alpha_phrase(input.alpha, input.alternative),
        alternative_phrase(input.alternative),
    ));
    match input.solve_mode {
        SolveMode::SampleSize => {
            let target = input.power.expect("validated");
            paragraphs.push(format!(
                "The target power is {:.0}%. The required number of pairs is {} with achieved \
                 power {:.2}%.",
                target * 100.0,
                result.n_pairs,
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::Power => {
            paragraphs.push(format!(
                "For {} pairs, achieved power is {:.2}%.",
                input.n_pairs.expect("validated"),
                result.achieved_power * 100.0,
            ));
        }
        SolveMode::DetectableEffect => unreachable!(),
    }
    if let Some(rate) = input.dropout_rate {
        paragraphs.push(format!(
            "A uniform dropout rate of {:.0}% is applied after sample-size rounding. The number \
             of pairs is inflated by 1/(1 − dropout) and rounded up, yielding {} pairs to enroll.",
            rate * 100.0,
            result.n_pairs_adjusted,
        ));
    }
    paragraphs.push(
        "The method assumes continuous paired differences without ties. It is the nonparametric \
         counterpart to the paired t-test."
            .into(),
    );
    join_paragraphs(paragraphs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::binary::odds_ratio::calculate as calculate_odds_ratio;
    use crate::methods::binary::risk_ratio::calculate as calculate_risk_ratio;
    use crate::methods::binary::two_proportion_difference::calculate as calculate_two_proportion;
    use crate::methods::continuous::ancova_two_sample::calculate as calculate_ancova;
    use crate::methods::continuous::one_sample_ttest::calculate as calculate_one_sample;
    use crate::methods::continuous::one_way_anova::calculate as calculate_anova;
    use crate::methods::continuous::paired_ttest::calculate as calculate_paired;
    use crate::methods::continuous::two_sample_ttest::calculate;
    use crate::methods::design::blinded_ssre::calculate as calculate_ssre;
    use crate::methods::design::group_sequential::calculate as calculate_gs;
    use crate::methods::design::multiplicity::{
        calculate as calculate_multiplicity, MultiplicityMethod,
    };
    use crate::methods::survival::log_rank::calculate as calculate_log_rank;
    use crate::types::{Alternative, SolveMode, StudyObjective};

    #[test]
    fn sample_size_rationale_includes_primary_quantities() {
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
        let text = two_sample_ttest_rationale(&input, &result);

        assert!(text.contains("superiority design"));
        assert!(text.contains("total N = 34"));
        assert!(text.contains("target power is 80%"));
        assert!(text.contains("noncentral t"));
    }

    #[test]
    fn one_sample_rationale_mentions_reference_mean() {
        let input = OneSampleTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            n: None,
            mean_difference: 1.0,
            standard_deviation: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate_one_sample(input.clone()).expect("calculate");
        let text = one_sample_ttest_rationale(&input, &result);

        assert!(text.contains("reference mean"));
        assert!(text.contains("sample size is 10"));
    }

    #[test]
    fn paired_rationale_mentions_pairs() {
        let input = PairedTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            n_pairs: None,
            mean_difference: 1.0,
            standard_deviation: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate_paired(input.clone()).expect("calculate");
        let text = paired_ttest_rationale(&input, &result);

        assert!(text.contains("paired"));
        assert!(text.contains("number of pairs"));
    }

    #[test]
    fn anova_rationale_mentions_cohens_f() {
        let input = OneWayAnovaInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            n_per_group: None,
            n_groups: 3,
            between_variance: 1.0,
            within_variance: 1.0,
            dropout_rate: None,
        };
        let result = calculate_anova(input.clone()).expect("calculate");
        let text = one_way_anova_rationale(&input, &result);

        assert!(text.contains("Cohen's f"));
        assert!(text.contains("3 groups"));
    }

    #[test]
    fn ancova_rationale_mentions_variance_reduction() {
        let input = AncovaTwoSampleInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            mean_difference: 1.0,
            standard_deviation: 1.0,
            baseline_outcome_correlation: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate_ancova(input.clone()).expect("calculate");
        let text = ancova_two_sample_rationale(&input, &result);

        assert!(text.contains("variance reduction"));
        assert!(text.contains("baseline-outcome correlation"));
    }

    #[test]
    fn two_proportion_rationale_mentions_superiority() {
        let input = TwoProportionDifferenceInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            control_rate: 0.3,
            treatment_rate: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            study_objective: StudyObjective::Superiority,
            noninferiority_margin: None,
            dropout_rate: None,
        };
        let result = calculate_two_proportion(input.clone()).expect("calculate");
        let text = two_proportion_difference_rationale(&input, &result);

        assert!(text.contains("superiority"));
        assert!(text.contains("difference in proportions"));
    }

    #[test]
    fn odds_ratio_rationale_mentions_effect() {
        let input = OddsRatioInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            control_rate: 0.3,
            treatment_rate: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate_odds_ratio(input.clone()).expect("calculate");
        let text = odds_ratio_rationale(&input, &result);

        assert!(text.contains("odds ratio"));
        assert!(text.contains("target power is 80%"));
    }

    #[test]
    fn risk_ratio_rationale_mentions_effect() {
        let input = RiskRatioInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            control_n: None,
            control_rate: 0.3,
            treatment_rate: 0.5,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            dropout_rate: None,
        };
        let result = calculate_risk_ratio(input.clone()).expect("calculate");
        let text = risk_ratio_rationale(&input, &result);

        assert!(text.contains("risk ratio"));
        assert!(text.contains("log risk ratio"));
    }

    #[test]
    fn log_rank_rationale_mentions_events() {
        let input = LogRankInput {
            solve_mode: SolveMode::SampleSize,
            alpha: 0.05,
            power: Some(0.8),
            total_events: None,
            hazard_ratio: 0.6,
            allocation_ratio: 1.0,
            alternative: Alternative::TwoSided,
            control_hazard_rate: None,
            accrual_duration: None,
            minimum_follow_up: None,
            dropout_hazard_rate: None,
        };
        let result = calculate_log_rank(input.clone()).expect("calculate");
        let text = log_rank_rationale(&input, &result);

        assert!(text.contains("log-rank"));
        assert!(text.contains("total events"));
    }

    #[test]
    fn multiplicity_rationale_mentions_adjusted_alpha() {
        let input = MultiplicityInput {
            family_wise_alpha: 0.05,
            number_of_comparisons: 2,
            adjustment_method: MultiplicityMethod::Bonferroni,
            gate_position: None,
            comparison_weights: None,
        };
        let result = calculate_multiplicity(input.clone()).expect("calculate");
        let text = multiplicity_rationale(&input, &result);

        assert!(text.contains("Bonferroni"));
        assert!(text.contains("adjusted per-comparison alpha"));
    }

    #[test]
    fn group_sequential_rationale_mentions_inflation() {
        let input = GroupSequentialInput {
            alpha: 0.05,
            target_power: 0.8,
            number_of_looks: 3,
            spending_function: SpendingFunction::ObrienFleming,
        };
        let result = calculate_gs(input.clone()).expect("calculate");
        let text = group_sequential_rationale(&input, &result);

        assert!(text.contains("inflation factor"));
        assert!(text.contains("O'Brien-Fleming"));
    }

    #[test]
    fn blinded_ssre_rationale_mentions_re_estimation() {
        let input = BlindedSsreInput {
            alpha: 0.05,
            target_power: 0.8,
            mean_difference: 1.0,
            planned_standard_deviation: 1.0,
            blinded_interim_standard_deviation: Some(1.2),
            interim_fraction: 0.5,
            allocation_ratio: 1.0,
            max_sample_size_multiplier: 1.5,
            alternative: Alternative::TwoSided,
        };
        let result = calculate_ssre(input.clone()).expect("calculate");
        let text = blinded_ssre_rationale(&input, &result);

        assert!(text.contains("blinded"));
        assert!(text.contains("variance ratio"));
    }
}
