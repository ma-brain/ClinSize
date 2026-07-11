use serde::{Deserialize, Serialize};

/// Direction of the alternative hypothesis, shared across method inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alternative {
    TwoSided,
    Less,
    Greater,
}

/// Clinical study objective for binary endpoint methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StudyObjective {
    Superiority,
    NonInferiority,
}

/// Which quantity a method should solve for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolveMode {
    SampleSize,
    Power,
    DetectableEffect,
}

/// A non-fatal warning surfaced alongside a calculation result, e.g. a
/// rounding note or an assumption the user should be aware of. Every
/// method result should carry a `Vec<CalculationWarning>` per
/// `06-ui-ux-guidelines.md` ("Result Display").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalculationWarning {
    pub code: String,
    pub message: String,
}

impl CalculationWarning {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
