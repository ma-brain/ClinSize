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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![engine_info, list_methods])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
