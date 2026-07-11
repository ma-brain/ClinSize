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
use clinsize_core::registry::MethodDescriptor;
use clinsize_core::types::SolveMode;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            engine_info,
            list_methods,
            calculate_two_sample_ttest,
            export_two_sample_ttest_markdown,
            calculate_one_sample_ttest,
            export_one_sample_ttest_markdown,
            calculate_paired_ttest,
            export_paired_ttest_markdown,
            calculate_one_way_anova,
            export_one_way_anova_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
