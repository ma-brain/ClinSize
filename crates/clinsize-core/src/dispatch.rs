//! JSON dispatch for calculations and Markdown reports.
//!
//! Shared by the CLI, validation scripts, and future bindings so method
//! routing stays out of application shells.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::methods::binary::odds_ratio::{self, OddsRatioInput, OddsRatioResult};
use crate::methods::binary::one_sample_binomial::{
    self, OneSampleBinomialInput, OneSampleBinomialResult,
};
use crate::methods::binary::risk_ratio::{self, RiskRatioInput, RiskRatioResult};
use crate::methods::binary::two_proportion_difference::{
    self, TwoProportionDifferenceInput, TwoProportionDifferenceResult,
};
use crate::methods::continuous::ancova_two_sample::{
    self, AncovaTwoSampleInput, AncovaTwoSampleResult,
};
use crate::methods::continuous::change_from_baseline::{
    self, ChangeFromBaselineInput, ChangeFromBaselineResult,
};
use crate::methods::continuous::mmrm::{self, MmrmInput, MmrmResult};
use crate::methods::continuous::mann_whitney::{self, MannWhitneyInput, MannWhitneyResult};
use crate::methods::continuous::one_sample_ttest::{
    self, OneSampleTTestInput, OneSampleTTestResult,
};
use crate::methods::continuous::one_way_anova::{self, OneWayAnovaInput, OneWayAnovaResult};
use crate::methods::continuous::paired_ttest::{self, PairedTTestInput, PairedTTestResult};
use crate::methods::continuous::two_sample_ttest::{
    self, TwoSampleTTestInput, TwoSampleTTestResult,
};
use crate::methods::continuous::wilcoxon_signed_rank::{
    self, WilcoxonSignedRankInput, WilcoxonSignedRankResult,
};
use crate::methods::count::negative_binomial::{
    self, NegativeBinomialInput, NegativeBinomialResult,
};
use crate::methods::ordinal::proportional_odds::{
    self, ProportionalOddsInput, ProportionalOddsResult,
};
use crate::methods::design::blinded_ssre::{self, BlindedSsreInput, BlindedSsreResult};
use crate::methods::design::group_sequential::{self, GroupSequentialInput, GroupSequentialResult};
use crate::methods::design::multiplicity::{self, MultiplicityInput, MultiplicityResult};
use crate::methods::survival::log_rank::{self, LogRankInput, LogRankResult};
use crate::reports;
use crate::{engine_version, Error, Result};

fn parse_input_json<T: DeserializeOwned>(input_json: &str) -> Result<T> {
    serde_json::from_str(input_json).map_err(|err| Error::InvalidInput {
        field: "input".into(),
        message: err.to_string(),
    })
}

fn parse_result_json<T: DeserializeOwned>(result_json: &str) -> Result<T> {
    serde_json::from_str(result_json).map_err(|err| Error::InvalidInput {
        field: "result".into(),
        message: err.to_string(),
    })
}

fn serialize_result<T: Serialize>(result: &T) -> Result<String> {
    serde_json::to_string_pretty(result).map_err(|err| Error::Export(err.to_string()))
}

fn calculate<I, R, F>(input_json: &str, calculate_fn: F) -> Result<String>
where
    I: DeserializeOwned,
    R: Serialize,
    F: FnOnce(I) -> Result<R>,
{
    let input = parse_input_json(input_json)?;
    let result = calculate_fn(input)?;
    serialize_result(&result)
}

/// Run a registered method from a JSON input document and return JSON results.
pub fn calculate_json(method_id: &str, input_json: &str) -> Result<String> {
    match method_id {
        "continuous.two_sample_ttest" => calculate(input_json, two_sample_ttest::calculate),
        "continuous.one_sample_ttest" => calculate(input_json, one_sample_ttest::calculate),
        "continuous.paired_ttest" => calculate(input_json, paired_ttest::calculate),
        "continuous.one_way_anova" => calculate(input_json, one_way_anova::calculate),
        "continuous.ancova_two_sample" => calculate(input_json, ancova_two_sample::calculate),
        "continuous.change_from_baseline" => calculate(input_json, change_from_baseline::calculate),
        "continuous.mmrm" => calculate(input_json, mmrm::calculate),
        "continuous.mann_whitney" => calculate(input_json, mann_whitney::calculate),
        "continuous.wilcoxon_signed_rank" => calculate(input_json, wilcoxon_signed_rank::calculate),
        "binary.two_proportion_difference" => {
            calculate(input_json, two_proportion_difference::calculate)
        }
        "binary.odds_ratio" => calculate(input_json, odds_ratio::calculate),
        "binary.one_sample_binomial" => calculate(input_json, one_sample_binomial::calculate),
        "binary.risk_ratio" => calculate(input_json, risk_ratio::calculate),
        "count.negative_binomial" => calculate(input_json, negative_binomial::calculate),
        "ordinal.proportional_odds" => calculate(input_json, proportional_odds::calculate),
        "survival.log_rank" => calculate(input_json, log_rank::calculate),
        "design.multiplicity" => calculate(input_json, multiplicity::calculate),
        "design.group_sequential" => calculate(input_json, group_sequential::calculate),
        "design.blinded_ssre" => calculate(input_json, blinded_ssre::calculate),
        other => Err(Error::UnsupportedMethod(other.into())),
    }
}

/// Render a Markdown calculation summary from JSON input and result documents.
pub fn report_markdown_json(
    method_id: &str,
    input_json: &str,
    result_json: &str,
) -> Result<String> {
    let version = engine_version();
    match method_id {
        "continuous.two_sample_ttest" => {
            let input = parse_input_json::<TwoSampleTTestInput>(input_json)?;
            let result = parse_result_json::<TwoSampleTTestResult>(result_json)?;
            Ok(reports::two_sample_ttest_markdown(&input, &result, version))
        }
        "continuous.one_sample_ttest" => {
            let input = parse_input_json::<OneSampleTTestInput>(input_json)?;
            let result = parse_result_json::<OneSampleTTestResult>(result_json)?;
            Ok(reports::one_sample_ttest_markdown(&input, &result, version))
        }
        "continuous.paired_ttest" => {
            let input = parse_input_json::<PairedTTestInput>(input_json)?;
            let result = parse_result_json::<PairedTTestResult>(result_json)?;
            Ok(reports::paired_ttest_markdown(&input, &result, version))
        }
        "continuous.one_way_anova" => {
            let input = parse_input_json::<OneWayAnovaInput>(input_json)?;
            let result = parse_result_json::<OneWayAnovaResult>(result_json)?;
            Ok(reports::one_way_anova_markdown(&input, &result, version))
        }
        "continuous.ancova_two_sample" => {
            let input = parse_input_json::<AncovaTwoSampleInput>(input_json)?;
            let result = parse_result_json::<AncovaTwoSampleResult>(result_json)?;
            Ok(reports::ancova_two_sample_markdown(
                &input, &result, version,
            ))
        }
        "continuous.change_from_baseline" => {
            let input = parse_input_json::<ChangeFromBaselineInput>(input_json)?;
            let result = parse_result_json::<ChangeFromBaselineResult>(result_json)?;
            Ok(reports::change_from_baseline_markdown(
                &input, &result, version,
            ))
        }
        "continuous.mmrm" => {
            let input = parse_input_json::<MmrmInput>(input_json)?;
            let result = parse_result_json::<MmrmResult>(result_json)?;
            Ok(reports::mmrm_markdown(&input, &result, version))
        }
        "continuous.mann_whitney" => {
            let input = parse_input_json::<MannWhitneyInput>(input_json)?;
            let result = parse_result_json::<MannWhitneyResult>(result_json)?;
            Ok(reports::mann_whitney_markdown(&input, &result, version))
        }
        "continuous.wilcoxon_signed_rank" => {
            let input = parse_input_json::<WilcoxonSignedRankInput>(input_json)?;
            let result = parse_result_json::<WilcoxonSignedRankResult>(result_json)?;
            Ok(reports::wilcoxon_signed_rank_markdown(
                &input, &result, version,
            ))
        }
        "binary.two_proportion_difference" => {
            let input = parse_input_json::<TwoProportionDifferenceInput>(input_json)?;
            let result = parse_result_json::<TwoProportionDifferenceResult>(result_json)?;
            Ok(reports::two_proportion_difference_markdown(
                &input, &result, version,
            ))
        }
        "binary.odds_ratio" => {
            let input = parse_input_json::<OddsRatioInput>(input_json)?;
            let result = parse_result_json::<OddsRatioResult>(result_json)?;
            Ok(reports::odds_ratio_markdown(&input, &result, version))
        }
        "binary.one_sample_binomial" => {
            let input = parse_input_json::<OneSampleBinomialInput>(input_json)?;
            let result = parse_result_json::<OneSampleBinomialResult>(result_json)?;
            Ok(reports::one_sample_binomial_markdown(
                &input, &result, version,
            ))
        }
        "binary.risk_ratio" => {
            let input = parse_input_json::<RiskRatioInput>(input_json)?;
            let result = parse_result_json::<RiskRatioResult>(result_json)?;
            Ok(reports::risk_ratio_markdown(&input, &result, version))
        }
        "count.negative_binomial" => {
            let input = parse_input_json::<NegativeBinomialInput>(input_json)?;
            let result = parse_result_json::<NegativeBinomialResult>(result_json)?;
            Ok(reports::negative_binomial_markdown(&input, &result, version))
        }
        "ordinal.proportional_odds" => {
            let input = parse_input_json::<ProportionalOddsInput>(input_json)?;
            let result = parse_result_json::<ProportionalOddsResult>(result_json)?;
            Ok(reports::proportional_odds_markdown(&input, &result, version))
        }
        "survival.log_rank" => {
            let input = parse_input_json::<LogRankInput>(input_json)?;
            let result = parse_result_json::<LogRankResult>(result_json)?;
            Ok(reports::log_rank_markdown(&input, &result, version))
        }
        "design.multiplicity" => {
            let input = parse_input_json::<MultiplicityInput>(input_json)?;
            let result = parse_result_json::<MultiplicityResult>(result_json)?;
            Ok(reports::multiplicity_markdown(&input, &result, version))
        }
        "design.group_sequential" => {
            let input = parse_input_json::<GroupSequentialInput>(input_json)?;
            let result = parse_result_json::<GroupSequentialResult>(result_json)?;
            Ok(reports::group_sequential_markdown(&input, &result, version))
        }
        "design.blinded_ssre" => {
            let input = parse_input_json::<BlindedSsreInput>(input_json)?;
            let result = parse_result_json::<BlindedSsreResult>(result_json)?;
            Ok(reports::blinded_ssre_markdown(&input, &result, version))
        }
        other => Err(Error::UnsupportedMethod(other.into())),
    }
}

/// Render narrative sample size calculation rationale from JSON input and result documents.
pub fn rationale_json(method_id: &str, input_json: &str, result_json: &str) -> Result<String> {
    match method_id {
        "continuous.two_sample_ttest" => {
            let input = parse_input_json::<TwoSampleTTestInput>(input_json)?;
            let result = parse_result_json::<TwoSampleTTestResult>(result_json)?;
            Ok(reports::rationale::two_sample_ttest_rationale(
                &input, &result,
            ))
        }
        "continuous.one_sample_ttest" => {
            let input = parse_input_json::<OneSampleTTestInput>(input_json)?;
            let result = parse_result_json::<OneSampleTTestResult>(result_json)?;
            Ok(reports::rationale::one_sample_ttest_rationale(
                &input, &result,
            ))
        }
        "continuous.paired_ttest" => {
            let input = parse_input_json::<PairedTTestInput>(input_json)?;
            let result = parse_result_json::<PairedTTestResult>(result_json)?;
            Ok(reports::rationale::paired_ttest_rationale(&input, &result))
        }
        "continuous.one_way_anova" => {
            let input = parse_input_json::<OneWayAnovaInput>(input_json)?;
            let result = parse_result_json::<OneWayAnovaResult>(result_json)?;
            Ok(reports::rationale::one_way_anova_rationale(&input, &result))
        }
        "continuous.ancova_two_sample" => {
            let input = parse_input_json::<AncovaTwoSampleInput>(input_json)?;
            let result = parse_result_json::<AncovaTwoSampleResult>(result_json)?;
            Ok(reports::rationale::ancova_two_sample_rationale(
                &input, &result,
            ))
        }
        "continuous.change_from_baseline" => {
            let input = parse_input_json::<ChangeFromBaselineInput>(input_json)?;
            let result = parse_result_json::<ChangeFromBaselineResult>(result_json)?;
            Ok(reports::rationale::change_from_baseline_rationale(
                &input, &result,
            ))
        }
        "continuous.mmrm" => {
            let input = parse_input_json::<MmrmInput>(input_json)?;
            let result = parse_result_json::<MmrmResult>(result_json)?;
            Ok(reports::rationale::mmrm_rationale(&input, &result))
        }
        "continuous.mann_whitney" => {
            let input = parse_input_json::<MannWhitneyInput>(input_json)?;
            let result = parse_result_json::<MannWhitneyResult>(result_json)?;
            Ok(reports::rationale::mann_whitney_rationale(&input, &result))
        }
        "continuous.wilcoxon_signed_rank" => {
            let input = parse_input_json::<WilcoxonSignedRankInput>(input_json)?;
            let result = parse_result_json::<WilcoxonSignedRankResult>(result_json)?;
            Ok(reports::rationale::wilcoxon_signed_rank_rationale(
                &input, &result,
            ))
        }
        "binary.two_proportion_difference" => {
            let input = parse_input_json::<TwoProportionDifferenceInput>(input_json)?;
            let result = parse_result_json::<TwoProportionDifferenceResult>(result_json)?;
            Ok(reports::rationale::two_proportion_difference_rationale(
                &input, &result,
            ))
        }
        "binary.odds_ratio" => {
            let input = parse_input_json::<OddsRatioInput>(input_json)?;
            let result = parse_result_json::<OddsRatioResult>(result_json)?;
            Ok(reports::rationale::odds_ratio_rationale(&input, &result))
        }
        "binary.one_sample_binomial" => {
            let input = parse_input_json::<OneSampleBinomialInput>(input_json)?;
            let result = parse_result_json::<OneSampleBinomialResult>(result_json)?;
            Ok(reports::rationale::one_sample_binomial_rationale(
                &input, &result,
            ))
        }
        "binary.risk_ratio" => {
            let input = parse_input_json::<RiskRatioInput>(input_json)?;
            let result = parse_result_json::<RiskRatioResult>(result_json)?;
            Ok(reports::rationale::risk_ratio_rationale(&input, &result))
        }
        "count.negative_binomial" => {
            let input = parse_input_json::<NegativeBinomialInput>(input_json)?;
            let result = parse_result_json::<NegativeBinomialResult>(result_json)?;
            Ok(reports::rationale::negative_binomial_rationale(
                &input, &result,
            ))
        }
        "ordinal.proportional_odds" => {
            let input = parse_input_json::<ProportionalOddsInput>(input_json)?;
            let result = parse_result_json::<ProportionalOddsResult>(result_json)?;
            Ok(reports::rationale::proportional_odds_rationale(
                &input, &result,
            ))
        }
        "survival.log_rank" => {
            let input = parse_input_json::<LogRankInput>(input_json)?;
            let result = parse_result_json::<LogRankResult>(result_json)?;
            Ok(reports::rationale::log_rank_rationale(&input, &result))
        }
        "design.multiplicity" => {
            let input = parse_input_json::<MultiplicityInput>(input_json)?;
            let result = parse_result_json::<MultiplicityResult>(result_json)?;
            Ok(reports::rationale::multiplicity_rationale(&input, &result))
        }
        "design.group_sequential" => {
            let input = parse_input_json::<GroupSequentialInput>(input_json)?;
            let result = parse_result_json::<GroupSequentialResult>(result_json)?;
            Ok(reports::rationale::group_sequential_rationale(
                &input, &result,
            ))
        }
        "design.blinded_ssre" => {
            let input = parse_input_json::<BlindedSsreInput>(input_json)?;
            let result = parse_result_json::<BlindedSsreResult>(result_json)?;
            Ok(reports::rationale::blinded_ssre_rationale(&input, &result))
        }
        other => Err(Error::UnsupportedMethod(other.into())),
    }
}

/// Render protocol-ready narrative text from JSON input and result documents.
pub fn protocol_text_json(method_id: &str, input_json: &str, result_json: &str) -> Result<String> {
    match method_id {
        "continuous.two_sample_ttest" => {
            let input = parse_input_json::<TwoSampleTTestInput>(input_json)?;
            let result = parse_result_json::<TwoSampleTTestResult>(result_json)?;
            Ok(reports::protocol::two_sample_ttest_protocol(&input, &result))
        }
        "continuous.one_sample_ttest" => {
            let input = parse_input_json::<OneSampleTTestInput>(input_json)?;
            let result = parse_result_json::<OneSampleTTestResult>(result_json)?;
            Ok(reports::protocol::one_sample_ttest_protocol(&input, &result))
        }
        "continuous.paired_ttest" => {
            let input = parse_input_json::<PairedTTestInput>(input_json)?;
            let result = parse_result_json::<PairedTTestResult>(result_json)?;
            Ok(reports::protocol::paired_ttest_protocol(&input, &result))
        }
        "continuous.one_way_anova" => {
            let input = parse_input_json::<OneWayAnovaInput>(input_json)?;
            let result = parse_result_json::<OneWayAnovaResult>(result_json)?;
            Ok(reports::protocol::one_way_anova_protocol(&input, &result))
        }
        "continuous.ancova_two_sample" => {
            let input = parse_input_json::<AncovaTwoSampleInput>(input_json)?;
            let result = parse_result_json::<AncovaTwoSampleResult>(result_json)?;
            Ok(reports::protocol::ancova_two_sample_protocol(&input, &result))
        }
        "continuous.change_from_baseline" => {
            let input = parse_input_json::<ChangeFromBaselineInput>(input_json)?;
            let result = parse_result_json::<ChangeFromBaselineResult>(result_json)?;
            Ok(reports::protocol::change_from_baseline_protocol(&input, &result))
        }
        "continuous.mmrm" => {
            let input = parse_input_json::<MmrmInput>(input_json)?;
            let result = parse_result_json::<MmrmResult>(result_json)?;
            Ok(reports::protocol::mmrm_protocol(&input, &result))
        }
        "continuous.mann_whitney" => {
            let input = parse_input_json::<MannWhitneyInput>(input_json)?;
            let result = parse_result_json::<MannWhitneyResult>(result_json)?;
            Ok(reports::protocol::mann_whitney_protocol(&input, &result))
        }
        "continuous.wilcoxon_signed_rank" => {
            let input = parse_input_json::<WilcoxonSignedRankInput>(input_json)?;
            let result = parse_result_json::<WilcoxonSignedRankResult>(result_json)?;
            Ok(reports::protocol::wilcoxon_signed_rank_protocol(&input, &result))
        }
        "binary.two_proportion_difference" => {
            let input = parse_input_json::<TwoProportionDifferenceInput>(input_json)?;
            let result = parse_result_json::<TwoProportionDifferenceResult>(result_json)?;
            Ok(reports::protocol::two_proportion_difference_protocol(
                &input, &result,
            ))
        }
        "binary.odds_ratio" => {
            let input = parse_input_json::<OddsRatioInput>(input_json)?;
            let result = parse_result_json::<OddsRatioResult>(result_json)?;
            Ok(reports::protocol::odds_ratio_protocol(&input, &result))
        }
        "binary.one_sample_binomial" => {
            let input = parse_input_json::<OneSampleBinomialInput>(input_json)?;
            let result = parse_result_json::<OneSampleBinomialResult>(result_json)?;
            Ok(reports::protocol::one_sample_binomial_protocol(&input, &result))
        }
        "binary.risk_ratio" => {
            let input = parse_input_json::<RiskRatioInput>(input_json)?;
            let result = parse_result_json::<RiskRatioResult>(result_json)?;
            Ok(reports::protocol::risk_ratio_protocol(&input, &result))
        }
        "count.negative_binomial" => {
            let input = parse_input_json::<NegativeBinomialInput>(input_json)?;
            let result = parse_result_json::<NegativeBinomialResult>(result_json)?;
            Ok(reports::protocol::negative_binomial_protocol(&input, &result))
        }
        "ordinal.proportional_odds" => {
            let input = parse_input_json::<ProportionalOddsInput>(input_json)?;
            let result = parse_result_json::<ProportionalOddsResult>(result_json)?;
            Ok(reports::protocol::proportional_odds_protocol(&input, &result))
        }
        "survival.log_rank" => {
            let input = parse_input_json::<LogRankInput>(input_json)?;
            let result = parse_result_json::<LogRankResult>(result_json)?;
            Ok(reports::protocol::log_rank_protocol(&input, &result))
        }
        "design.multiplicity" => {
            let input = parse_input_json::<MultiplicityInput>(input_json)?;
            let result = parse_result_json::<MultiplicityResult>(result_json)?;
            Ok(reports::protocol::multiplicity_protocol(&input, &result))
        }
        "design.group_sequential" => {
            let input = parse_input_json::<GroupSequentialInput>(input_json)?;
            let result = parse_result_json::<GroupSequentialResult>(result_json)?;
            Ok(reports::protocol::group_sequential_protocol(&input, &result))
        }
        "design.blinded_ssre" => {
            let input = parse_input_json::<BlindedSsreInput>(input_json)?;
            let result = parse_result_json::<BlindedSsreResult>(result_json)?;
            Ok(reports::protocol::blinded_ssre_protocol(&input, &result))
        }
        other => Err(Error::UnsupportedMethod(other.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_two_sample_ttest_from_json() {
        let input = r#"{
            "solveMode": "sample_size",
            "alpha": 0.05,
            "power": 0.8,
            "meanDifference": 1,
            "standardDeviation": 1,
            "allocationRatio": 1,
            "alternative": "two_sided"
        }"#;
        let json = calculate_json("continuous.two_sample_ttest", input).expect("calculate");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse result");
        assert_eq!(parsed["nControl"], 17);
        assert_eq!(parsed["nTreatment"], 17);
    }

    #[test]
    fn report_two_sample_ttest_from_json() {
        let input = r#"{
            "solveMode": "sample_size",
            "alpha": 0.05,
            "power": 0.8,
            "meanDifference": 1,
            "standardDeviation": 1,
            "allocationRatio": 1,
            "alternative": "two_sided"
        }"#;
        let result = calculate_json("continuous.two_sample_ttest", input).expect("calculate");
        let markdown =
            report_markdown_json("continuous.two_sample_ttest", input, &result).expect("report");
        assert!(markdown.contains("# ClinSize calculation summary"));
        assert!(markdown.contains("Two-sample t-test"));
    }
}
