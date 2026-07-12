mod export_file;
mod project;

use clinsize_core::registry::MethodDescriptor;
use clinsize_core::types::SolveMode;
use export_file::{write_export_file, ExportFileError};
use project::{
    open_project_at_path, save_project_at_path, save_project_to_active_path, OpenedProjectFile,
    ProjectFile, ProjectFileError, ProjectFileState,
};
use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;

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

impl From<ProjectFileError> for AppError {
    fn from(err: ProjectFileError) -> Self {
        Self {
            code: "project_file".into(),
            message: err.to_string(),
        }
    }
}

impl From<ExportFileError> for AppError {
    fn from(err: ExportFileError) -> Self {
        Self {
            code: "export".into(),
            message: err.to_string(),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ExportFileType {
    Markdown,
    Html,
    Word,
    Pdf,
    Svg,
    Png,
}

impl ExportFileType {
    fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Word => "docx",
            Self::Pdf => "pdf",
            Self::Svg => "svg",
            Self::Png => "png",
        }
    }

    fn filter_name(self) -> &'static str {
        match self {
            Self::Markdown => "Markdown",
            Self::Html => "HTML",
            Self::Word => "Word document",
            Self::Pdf => "PDF",
            Self::Svg => "SVG image",
            Self::Png => "PNG image",
        }
    }

    fn default_file_name(self, file_stem: &str) -> String {
        format!("{}.{}", sanitize_file_stem(file_stem), self.extension())
    }
}

fn sanitize_file_stem(file_stem: &str) -> String {
    let mut result = String::new();
    let mut previous_was_separator = false;

    for character in file_stem.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !result.is_empty() && !previous_was_separator {
            result.push('-');
            previous_was_separator = true;
        }
    }

    let result = result.trim_matches('-');
    if result.is_empty() {
        "clinsize-export".into()
    } else {
        result.into()
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
fn calculate_method(
    method_id: String,
    input: serde_json::Value,
) -> Result<serde_json::Value, AppError> {
    let input_json = serde_json::to_string(&input).map_err(|err| AppError {
        code: "internal".into(),
        message: err.to_string(),
    })?;
    let result_json =
        clinsize_core::dispatch::calculate_json(&method_id, &input_json).map_err(AppError::from)?;

    serde_json::from_str(&result_json).map_err(|err| AppError {
        code: "internal".into(),
        message: err.to_string(),
    })
}

#[tauri::command]
fn export_method_markdown(
    method_id: String,
    input: serde_json::Value,
    result: serde_json::Value,
) -> Result<String, AppError> {
    let input_json = serde_json::to_string(&input).map_err(|err| AppError {
        code: "internal".into(),
        message: err.to_string(),
    })?;
    let result_json = serde_json::to_string(&result).map_err(|err| AppError {
        code: "internal".into(),
        message: err.to_string(),
    })?;

    clinsize_core::dispatch::report_markdown_json(&method_id, &input_json, &result_json)
        .map_err(AppError::from)
}

#[tauri::command]
fn export_calculation_rationale(
    method_id: String,
    input_json: String,
    result_json: String,
) -> Result<String, AppError> {
    clinsize_core::dispatch::rationale_json(&method_id, &input_json, &result_json)
        .map_err(AppError::from)
}

#[tauri::command]
fn export_protocol_text(
    method_id: String,
    input_json: String,
    result_json: String,
) -> Result<String, AppError> {
    clinsize_core::dispatch::protocol_text_json(&method_id, &input_json, &result_json)
        .map_err(AppError::from)
}

#[tauri::command]
fn create_project(name: String, state: tauri::State<'_, ProjectFileState>) -> ProjectFile {
    state.clear_active_path();
    ProjectFile::new(name)
}

#[tauri::command]
async fn open_project_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, ProjectFileState>,
) -> Result<Option<OpenedProjectFile>, AppError> {
    let selected_path = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("ClinSize project", &["clinsize.json"])
            .blocking_pick_file()
    })
    .await
    .map_err(|err| AppError {
        code: "internal".into(),
        message: err.to_string(),
    })?;

    let Some(selected_path) = selected_path else {
        return Ok(None);
    };
    let path = selected_path.into_path().map_err(|err| AppError {
        code: "project_file".into(),
        message: err.to_string(),
    })?;

    open_project_at_path(&state, path)
        .map(Some)
        .map_err(AppError::from)
}

#[tauri::command]
async fn save_project_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, ProjectFileState>,
    project: ProjectFile,
) -> Result<Option<String>, AppError> {
    if state.active_path().is_some() {
        return save_project_to_active_path(&state, &project)
            .map(Some)
            .map_err(AppError::from);
    }

    let default_file_name = format!("{}.clinsize.json", project.name);
    let selected_path = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("ClinSize project", &["clinsize.json"])
            .set_file_name(default_file_name)
            .blocking_save_file()
    })
    .await
    .map_err(|err| AppError {
        code: "internal".into(),
        message: err.to_string(),
    })?;

    let Some(selected_path) = selected_path else {
        return Ok(None);
    };
    let path = selected_path.into_path().map_err(|err| AppError {
        code: "project_file".into(),
        message: err.to_string(),
    })?;

    save_project_at_path(&state, path, &project)
        .map(Some)
        .map_err(AppError::from)
}

#[tauri::command]
async fn save_export_file(
    app: tauri::AppHandle,
    export_type: ExportFileType,
    file_stem: String,
    contents: Vec<u8>,
) -> Result<Option<String>, AppError> {
    let default_file_name = export_type.default_file_name(&file_stem);
    let selected_path = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter(export_type.filter_name(), &[export_type.extension()])
            .set_file_name(default_file_name)
            .blocking_save_file()
    })
    .await
    .map_err(|err| AppError {
        code: "internal".into(),
        message: err.to_string(),
    })?;

    let Some(selected_path) = selected_path else {
        return Ok(None);
    };
    let path = selected_path.into_path().map_err(|err| AppError {
        code: "export".into(),
        message: err.to_string(),
    })?;

    write_export_file(&path, &contents)
        .map(Some)
        .map_err(AppError::from)
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
    // Evidence is embedded at compile time; a packaged install has no
    // repository checkout to read from.
    clinsize_core::validation_report::generate_markdown_embedded(&method_id).map_err(AppError::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ProjectFileState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            engine_info,
            list_methods,
            calculate_method,
            export_method_markdown,
            export_calculation_rationale,
            export_protocol_text,
            create_project,
            open_project_file,
            save_project_file,
            save_export_file,
            export_markdown_as_html,
            export_markdown_as_docx,
            export_markdown_as_pdf,
            generate_validation_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod export_file_tests {
    use super::*;

    fn two_sample_ttest_input() -> serde_json::Value {
        serde_json::json!({
            "solveMode": "sample_size",
            "alpha": 0.05,
            "power": 0.8,
            "meanDifference": 1,
            "standardDeviation": 1,
            "allocationRatio": 1,
            "alternative": "two_sided"
        })
    }

    fn poisson_input() -> serde_json::Value {
        serde_json::json!({
            "solveMode": "sample_size",
            "alpha": 0.05,
            "power": 0.8,
            "controlRate": 2,
            "treatmentRate": 1,
            "exposureTime": 1,
            "allocationRatio": 1,
            "alternative": "two_sided"
        })
    }

    #[test]
    fn generic_method_calculates_from_json_value() {
        let result = calculate_method(
            "continuous.two_sample_ttest".into(),
            two_sample_ttest_input(),
        )
        .expect("calculate method");

        assert_eq!(result["nControl"], 17);
        assert_eq!(result["nTreatment"], 17);
    }

    #[test]
    fn generic_method_exports_markdown_from_json_values() {
        let input = two_sample_ttest_input();
        let result = calculate_method("continuous.two_sample_ttest".into(), input.clone())
            .expect("calculate method");

        let markdown = export_method_markdown("continuous.two_sample_ttest".into(), input, result)
            .expect("export markdown");

        assert!(markdown.contains("# ClinSize calculation summary"));
        assert!(markdown.contains("Two-sample t-test"));
    }

    #[test]
    fn generic_method_handles_registered_poisson_without_legacy_commands() {
        let input = poisson_input();
        let result = calculate_method("count.poisson".into(), input.clone())
            .expect("calculate Poisson method");

        assert!(result["totalN"].as_u64().is_some());

        let markdown = export_method_markdown("count.poisson".into(), input, result)
            .expect("export Poisson markdown");

        assert!(markdown.contains("# ClinSize calculation summary"));
        assert!(markdown.contains("Two-sample Poisson (event counts)"));

        let source = include_str!("lib.rs");
        let legacy_calculate = ["calculate", "poisson"].join("_");
        let legacy_export = ["export", "poisson", "markdown"].join("_");
        assert!(!source.contains(&format!("fn {legacy_calculate}(")));
        assert!(!source.contains(&format!("fn {legacy_export}(")));
    }

    #[test]
    fn generic_method_rejects_unsupported_method_id() {
        let error = calculate_method("unsupported.method".into(), serde_json::json!({}))
            .expect_err("unsupported method should fail");

        assert_eq!(error.code, "unsupported_method");
    }

    #[test]
    fn generic_method_commands_are_registered_with_tauri() {
        let source = include_str!("lib.rs");

        assert!(source.contains("            calculate_method,\n"));
        assert!(source.contains("            export_method_markdown,\n"));
        assert!(source.contains("            export_calculation_rationale,\n"));
        assert!(source.contains("            export_protocol_text,\n"));
    }

    #[test]
    fn writes_binary_export_content_and_returns_the_file_name() {
        let path = std::env::temp_dir().join(format!(
            "clinsize-export-{}-{}.pdf",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("current time")
                .as_nanos()
        ));
        let contents = [0, 1, 2, 255];

        let file_name = write_export_file(&path, &contents).expect("write export file");
        let saved = std::fs::read(&path).expect("read export file");
        std::fs::remove_file(&path).expect("remove temporary export file");

        assert_eq!(
            file_name,
            path.file_name().expect("file name").to_string_lossy()
        );
        assert_eq!(saved, contents);
    }

    #[test]
    fn export_file_types_create_safe_default_file_names() {
        assert_eq!(
            ExportFileType::Pdf.default_file_name("summary/../../trial"),
            "summary-trial.pdf"
        );
    }

    #[test]
    fn renderer_has_no_filesystem_plugin_authority() {
        assert!(!include_str!("../capabilities/default.json").contains("\"fs:"));
        assert!(!include_str!("../Cargo.toml").contains("tauri-plugin-fs"));
        assert!(!include_str!("../../package.json").contains("@tauri-apps/plugin-fs"));
        assert!(!include_str!("../../package.json").contains("@tauri-apps/plugin-dialog"));
    }

    #[test]
    fn tauri_config_uses_restrictive_production_and_development_csps() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).expect("valid Tauri config");
        let security = &config["app"]["security"];
        let production_csp = security["csp"].as_str().expect("configured production CSP");
        let development_csp = security["devCsp"]
            .as_str()
            .expect("configured development CSP");

        assert!(production_csp.contains("default-src 'self'"));
        assert!(production_csp.contains("connect-src ipc: http://ipc.localhost"));
        assert!(production_csp.contains("object-src 'none'"));
        assert!(production_csp.contains("form-action 'none'"));
        assert!(production_csp.contains("frame-ancestors 'none'"));
        assert!(!production_csp.contains("http://localhost"));
        assert!(development_csp.contains("http://localhost:1420"));
        assert!(development_csp.contains("ws://localhost:1420"));
    }
}
