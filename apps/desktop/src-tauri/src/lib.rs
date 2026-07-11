mod project;

use clinsize_core::methods::binary::odds_ratio::{self, OddsRatioInput, OddsRatioResult};
use clinsize_core::methods::binary::risk_ratio::{self, RiskRatioInput, RiskRatioResult};
use clinsize_core::methods::binary::two_proportion_difference::{
    self, TwoProportionDifferenceInput, TwoProportionDifferenceResult,
};
use clinsize_core::methods::continuous::ancova_two_sample::{
    self, AncovaTwoSampleInput, AncovaTwoSampleResult,
};
use clinsize_core::methods::continuous::one_sample_ttest::{
    self, OneSampleTTestInput, OneSampleTTestResult,
};
use clinsize_core::methods::continuous::one_way_anova::{
    self, OneWayAnovaInput, OneWayAnovaResult,
};
use clinsize_core::methods::continuous::paired_ttest::{self, PairedTTestInput, PairedTTestResult};
use clinsize_core::methods::continuous::two_sample_ttest::{
    self, TwoSampleTTestInput, TwoSampleTTestResult,
};
use clinsize_core::methods::design::blinded_ssre::{self, BlindedSsreInput, BlindedSsreResult};
use clinsize_core::methods::design::group_sequential::{
    self, GroupSequentialInput, GroupSequentialResult,
};
use clinsize_core::methods::design::multiplicity::{self, MultiplicityInput, MultiplicityResult};
use clinsize_core::methods::survival::log_rank::{self, LogRankInput, LogRankResult};
use clinsize_core::registry::MethodDescriptor;
use clinsize_core::types::SolveMode;
use project::ProjectFile;
use serde::Serialize;

/// UI-facing error returned from Tauri commands.
#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl From<clinsize_core::Error> for AppError {
    fn from(err: clinsize_core::Error) -> Self {
        let code = match &err {
            clinsize_core::Error::InvalidInput { .. } => "invalid_input",
            clinsize_core::Error::UnsupportedMethod(_) => "unsupported_method",
            clinsize_core::Error::ConvergenceFailure(_) => "convergence_failure",
            clinsize_core::Error::Internal(_) => "internal",
            clinsize_core::Error::Export(_) => "export",
        };
        Self {
            code: code.into(),
            message: err.to_string(),
        }
    }
}

/// Serializable method metadata for the Svelte UI.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MethodDescriptorDto {
    id: String,
    display_name: String,
    endpoint_category: String,
    supported_solve_modes: Vec<SolveMode>,
    documentation_path: Option<String>,
}

impl From<&MethodDescriptor> for MethodDescriptorDto {
    fn from(method: &MethodDescriptor) -> Self {
        Self {
            id: method.id.into(),
            display_name: method.display_name.into(),
            endpoint_category: method.endpoint_category.into(),
            supported_solve_modes: method.supported_solve_modes.to_vec(),
            documentation_path: method.documentation_path.map(str::to_string),
        }
    }
}

// Tauri commands are thin wrappers per `05-svelte-tauri-standards.md`:
// no calculation logic belongs here, only calls into `clinsize-core`.
#[tauri::command]
fn engine_info() -> String {
    clinsize_core::engine_version().to_string()
}

#[tauri::command]
fn list_methods() -> Vec<MethodDescriptorDto> {
    clinsize_core::registry::list_methods()
        .iter()
        .map(MethodDescriptorDto::from)
        .collect()
}

#[tauri::command]
fn calculate_two_sample_ttest(
    input: TwoSampleTTestInput,
) -> Result<TwoSampleTTestResult, AppError> {
    two_sample_ttest::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_two_sample_ttest_markdown(
    input: TwoSampleTTestInput,
    result: TwoSampleTTestResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::two_sample_ttest_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_one_sample_ttest(
    input: OneSampleTTestInput,
) -> Result<OneSampleTTestResult, AppError> {
    one_sample_ttest::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_one_sample_ttest_markdown(
    input: OneSampleTTestInput,
    result: OneSampleTTestResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::one_sample_ttest_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_paired_ttest(input: PairedTTestInput) -> Result<PairedTTestResult, AppError> {
    paired_ttest::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_paired_ttest_markdown(
    input: PairedTTestInput,
    result: PairedTTestResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::paired_ttest_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_one_way_anova(input: OneWayAnovaInput) -> Result<OneWayAnovaResult, AppError> {
    one_way_anova::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_one_way_anova_markdown(
    input: OneWayAnovaInput,
    result: OneWayAnovaResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::one_way_anova_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_ancova_two_sample(
    input: AncovaTwoSampleInput,
) -> Result<AncovaTwoSampleResult, AppError> {
    ancova_two_sample::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_ancova_two_sample_markdown(
    input: AncovaTwoSampleInput,
    result: AncovaTwoSampleResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::ancova_two_sample_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_two_proportion_difference(
    input: TwoProportionDifferenceInput,
) -> Result<TwoProportionDifferenceResult, AppError> {
    two_proportion_difference::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_two_proportion_difference_markdown(
    input: TwoProportionDifferenceInput,
    result: TwoProportionDifferenceResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::two_proportion_difference_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_odds_ratio(input: OddsRatioInput) -> Result<OddsRatioResult, AppError> {
    odds_ratio::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_odds_ratio_markdown(
    input: OddsRatioInput,
    result: OddsRatioResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::odds_ratio_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_risk_ratio(input: RiskRatioInput) -> Result<RiskRatioResult, AppError> {
    risk_ratio::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_risk_ratio_markdown(
    input: RiskRatioInput,
    result: RiskRatioResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::risk_ratio_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_log_rank(input: LogRankInput) -> Result<LogRankResult, AppError> {
    log_rank::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_log_rank_markdown(
    input: LogRankInput,
    result: LogRankResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::log_rank_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_multiplicity(input: MultiplicityInput) -> Result<MultiplicityResult, AppError> {
    multiplicity::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_multiplicity_markdown(
    input: MultiplicityInput,
    result: MultiplicityResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::multiplicity_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_group_sequential(
    input: GroupSequentialInput,
) -> Result<GroupSequentialResult, AppError> {
    group_sequential::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_group_sequential_markdown(
    input: GroupSequentialInput,
    result: GroupSequentialResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::group_sequential_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn calculate_blinded_ssre(input: BlindedSsreInput) -> Result<BlindedSsreResult, AppError> {
    blinded_ssre::calculate(input).map_err(AppError::from)
}

#[tauri::command]
fn export_blinded_ssre_markdown(
    input: BlindedSsreInput,
    result: BlindedSsreResult,
) -> Result<String, AppError> {
    Ok(clinsize_core::reports::blinded_ssre_markdown(
        &input,
        &result,
        clinsize_core::engine_version(),
    ))
}

#[tauri::command]
fn create_project(name: String) -> ProjectFile {
    ProjectFile::new(name)
}

#[tauri::command]
fn read_project_file(path: String) -> Result<ProjectFile, AppError> {
    let content = std::fs::read_to_string(&path).map_err(|err| AppError {
        code: "export".into(),
        message: err.to_string(),
    })?;
    serde_json::from_str(&content).map_err(|err| AppError {
        code: "export".into(),
        message: err.to_string(),
    })
}

#[tauri::command]
fn write_project_file(path: String, project: ProjectFile) -> Result<(), AppError> {
    let content = serde_json::to_string_pretty(&project).map_err(|err| AppError {
        code: "export".into(),
        message: err.to_string(),
    })?;
    std::fs::write(path, content).map_err(|err| AppError {
        code: "export".into(),
        message: err.to_string(),
    })
}

#[tauri::command]
fn export_markdown_as_html(markdown: String, title: String) -> String {
    clinsize_core::reports::html::markdown_to_html_document(&markdown, &title)
}

#[tauri::command]
fn export_markdown_as_docx(markdown: String, title: String) -> Result<Vec<u8>, AppError> {
    clinsize_core::reports::docx::markdown_to_docx_bytes(&markdown, &title).map_err(AppError::from)
}

#[tauri::command]
fn export_markdown_as_pdf(markdown: String, title: String) -> Result<Vec<u8>, AppError> {
    clinsize_core::reports::pdf::markdown_to_pdf_bytes(&markdown, &title).map_err(AppError::from)
}

#[tauri::command]
fn generate_validation_report(method_id: String) -> Result<String, AppError> {
    let validation_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../validation");
    clinsize_core::validation_report::generate_markdown(&method_id, &validation_root)
        .map_err(AppError::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            engine_info,
            list_methods,
            create_project,
            read_project_file,
            write_project_file,
            export_markdown_as_html,
            export_markdown_as_docx,
            export_markdown_as_pdf,
            generate_validation_report,
            calculate_two_sample_ttest,
            export_two_sample_ttest_markdown,
            calculate_one_sample_ttest,
            export_one_sample_ttest_markdown,
            calculate_paired_ttest,
            export_paired_ttest_markdown,
            calculate_one_way_anova,
            export_one_way_anova_markdown,
            calculate_ancova_two_sample,
            export_ancova_two_sample_markdown,
            calculate_two_proportion_difference,
            export_two_proportion_difference_markdown,
            calculate_odds_ratio,
            export_odds_ratio_markdown,
            calculate_risk_ratio,
            export_risk_ratio_markdown,
            calculate_log_rank,
            export_log_rank_markdown,
            calculate_multiplicity,
            export_multiplicity_markdown,
            calculate_group_sequential,
            export_group_sequential_markdown,
            calculate_blinded_ssre,
            export_blinded_ssre_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
