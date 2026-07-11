use thiserror::Error;

/// Structured errors for the calculation engine.
///
/// See handbook `02-architecture.md` ("Error Handling") for the category
/// definitions. Do not return raw strings from method calculation logic;
/// construct one of these variants instead so the boundary layer (Tauri
/// commands, CLI) can map it to a user-facing message.
#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("invalid input for `{field}`: {message}")]
    InvalidInput { field: String, message: String },

    #[error("unsupported method: {0}")]
    UnsupportedMethod(String),

    #[error("numerical convergence failure: {0}")]
    ConvergenceFailure(String),

    #[error("internal calculation error: {0}")]
    Internal(String),

    #[error("export error: {0}")]
    Export(String),
}

pub type Result<T> = std::result::Result<T, Error>;
