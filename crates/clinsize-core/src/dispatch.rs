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
use crate::methods::continuous::mann_whitney::{self, MannWhitneyInput, MannWhitneyResult};
use crate::methods::continuous::mmrm::{self, MmrmInput, MmrmResult};
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
use crate::methods::design::blinded_ssre::{self, BlindedSsreInput, BlindedSsreResult};
use crate::methods::design::group_sequential::{self, GroupSequentialInput, GroupSequentialResult};
use crate::methods::design::multiplicity::{self, MultiplicityInput, MultiplicityResult};
use crate::methods::ordinal::proportional_odds::{
    self, ProportionalOddsInput, ProportionalOddsResult,
};
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

/// Generates the four public dispatch functions (`calculate_json`,
/// `report_markdown_json`, `rationale_json`, `protocol_text_json`) from a single
/// list of registered methods. Adding a method means adding one line here — the
/// four match arms are produced automatically, so the "add a method, forget a
/// site" failure mode is structurally eliminated.
macro_rules! dispatch_methods {
    ( $( $id:literal => $module:ident : $input:ident / $result:ident ),+ $(,)? ) => {
        paste::paste! {
            /// Run a registered method from a JSON input document and return JSON results.
            pub fn calculate_json(method_id: &str, input_json: &str) -> Result<String> {
                match method_id {
                    $( $id => calculate(input_json, $module::calculate), )+
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
                    $( $id => {
                        let input = parse_input_json::<$input>(input_json)?;
                        let result = parse_result_json::<$result>(result_json)?;
                        Ok(reports::[<$module _markdown>](&input, &result, version))
                    }, )+
                    other => Err(Error::UnsupportedMethod(other.into())),
                }
            }

            /// Render narrative sample size calculation rationale from JSON input and result documents.
            pub fn rationale_json(
                method_id: &str,
                input_json: &str,
                result_json: &str,
            ) -> Result<String> {
                match method_id {
                    $( $id => {
                        let input = parse_input_json::<$input>(input_json)?;
                        let result = parse_result_json::<$result>(result_json)?;
                        Ok(reports::rationale::[<$module _rationale>](&input, &result))
                    }, )+
                    other => Err(Error::UnsupportedMethod(other.into())),
                }
            }

            /// Render protocol-ready narrative text from JSON input and result documents.
            pub fn protocol_text_json(
                method_id: &str,
                input_json: &str,
                result_json: &str,
            ) -> Result<String> {
                match method_id {
                    $( $id => {
                        let input = parse_input_json::<$input>(input_json)?;
                        let result = parse_result_json::<$result>(result_json)?;
                        Ok(reports::protocol::[<$module _protocol>](&input, &result))
                    }, )+
                    other => Err(Error::UnsupportedMethod(other.into())),
                }
            }
        }
    };
}

dispatch_methods! {
    "continuous.two_sample_ttest"      => two_sample_ttest       : TwoSampleTTestInput / TwoSampleTTestResult,
    "continuous.one_sample_ttest"      => one_sample_ttest       : OneSampleTTestInput / OneSampleTTestResult,
    "continuous.paired_ttest"          => paired_ttest           : PairedTTestInput / PairedTTestResult,
    "continuous.one_way_anova"         => one_way_anova          : OneWayAnovaInput / OneWayAnovaResult,
    "continuous.ancova_two_sample"     => ancova_two_sample      : AncovaTwoSampleInput / AncovaTwoSampleResult,
    "continuous.change_from_baseline"  => change_from_baseline   : ChangeFromBaselineInput / ChangeFromBaselineResult,
    "continuous.mmrm"                  => mmrm                   : MmrmInput / MmrmResult,
    "continuous.mann_whitney"          => mann_whitney           : MannWhitneyInput / MannWhitneyResult,
    "continuous.wilcoxon_signed_rank"  => wilcoxon_signed_rank   : WilcoxonSignedRankInput / WilcoxonSignedRankResult,
    "binary.two_proportion_difference" => two_proportion_difference : TwoProportionDifferenceInput / TwoProportionDifferenceResult,
    "binary.odds_ratio"                => odds_ratio             : OddsRatioInput / OddsRatioResult,
    "binary.one_sample_binomial"       => one_sample_binomial    : OneSampleBinomialInput / OneSampleBinomialResult,
    "binary.risk_ratio"                => risk_ratio             : RiskRatioInput / RiskRatioResult,
    "count.negative_binomial"          => negative_binomial      : NegativeBinomialInput / NegativeBinomialResult,
    "ordinal.proportional_odds"        => proportional_odds      : ProportionalOddsInput / ProportionalOddsResult,
    "survival.log_rank"                => log_rank               : LogRankInput / LogRankResult,
    "design.multiplicity"              => multiplicity           : MultiplicityInput / MultiplicityResult,
    "design.group_sequential"          => group_sequential       : GroupSequentialInput / GroupSequentialResult,
    "design.blinded_ssre"              => blinded_ssre           : BlindedSsreInput / BlindedSsreResult,
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

    /// Guards against the "add a method, forget a dispatch site" failure mode:
    /// every method in the registry must resolve to a dispatch arm (i.e. must
    /// not return `UnsupportedMethod`). An empty input is expected to fail with
    /// a parse error, which still proves the arm exists.
    #[test]
    fn every_registered_method_is_dispatchable() {
        for desc in crate::registry::list_methods() {
            let err = calculate_json(desc.id, "{}").unwrap_err();
            assert!(
                !matches!(err, Error::UnsupportedMethod(_)),
                "{} is registered but has no dispatch arm",
                desc.id
            );
        }
    }
}
