//! ClinSize project file model for calculation history and scenario comparison.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFile {
    pub version: u32,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub calculations: Vec<CalculationRecord>,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationRecord {
    pub id: String,
    pub method_id: String,
    pub method_name: String,
    pub label: Option<String>,
    pub created_at: String,
    pub input: serde_json::Value,
    pub result: serde_json::Value,
    pub summary: CalculationSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationSummary {
    pub primary_label: String,
    pub primary_value: String,
    pub secondary_label: Option<String>,
    pub secondary_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub calculation_ids: Vec<String>,
}

impl ProjectFile {
    pub fn new(name: impl Into<String>) -> Self {
        let timestamp = iso_timestamp();
        Self {
            version: 1,
            name: name.into(),
            created_at: timestamp.clone(),
            updated_at: timestamp,
            calculations: Vec::new(),
            scenarios: Vec::new(),
        }
    }
}

pub fn iso_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{seconds}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_project_file() {
        let project = ProjectFile::new("Example trial");
        let json = serde_json::to_string(&project).expect("serialize");
        assert!(json.contains("\"name\":\"Example trial\""));
    }
}
