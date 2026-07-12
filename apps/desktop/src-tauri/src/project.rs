//! ClinSize project file model for calculation history and scenario comparison.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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

/// Current UTC time as an ISO 8601 string (`YYYY-MM-DDTHH:MM:SSZ`).
///
/// Built from `std::time::SystemTime` via the Howard Hinnant civil-from-days
/// algorithm (http://howardhinnant.github.io/date_algorithms.html) to avoid a
/// `chrono` direct dependency. Only second resolution, always UTC.
pub fn iso_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let secs_in_day = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = secs_in_day / 3_600;
    let minute = (secs_in_day % 3_600) / 60;
    let second = secs_in_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Howard Hinnant's civil-from-days algorithm: converts a count of days since
/// the Unix epoch (1970-01-01) to a proleptic Gregorian (year, month, day).
/// Valid for any date in the algorithm's supported range.
fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146_096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if month <= 2 { year + 1 } else { year }, month, day)
}

const PROJECT_FILE_SUFFIX: &str = ".clinsize.json";

pub fn is_project_file_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(PROJECT_FILE_SUFFIX))
}

#[derive(Debug)]
pub enum ProjectFileError {
    InvalidPath,
    NoActivePath,
    Io(std::io::Error),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for ProjectFileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => write!(
                formatter,
                "Project files must use the .clinsize.json extension"
            ),
            Self::NoActivePath => write!(formatter, "No project file is currently open"),
            Self::Io(error) => error.fmt(formatter),
            Self::Serialization(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ProjectFileError {}

pub fn read_project_file(path: &Path) -> Result<ProjectFile, ProjectFileError> {
    validate_project_file_path(path)?;
    let content = std::fs::read_to_string(path).map_err(ProjectFileError::Io)?;
    serde_json::from_str(&content).map_err(ProjectFileError::Serialization)
}

pub fn write_project_file(path: &Path, project: &ProjectFile) -> Result<(), ProjectFileError> {
    validate_project_file_path(path)?;
    let content = serde_json::to_string_pretty(project).map_err(ProjectFileError::Serialization)?;
    std::fs::write(path, content).map_err(ProjectFileError::Io)
}

fn validate_project_file_path(path: &Path) -> Result<(), ProjectFileError> {
    if is_project_file_path(path) {
        Ok(())
    } else {
        Err(ProjectFileError::InvalidPath)
    }
}

#[derive(Default)]
pub struct ProjectFileState {
    active_path: Mutex<Option<PathBuf>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedProjectFile {
    pub project: ProjectFile,
    pub file_name: String,
}

pub fn open_project_at_path(
    state: &ProjectFileState,
    path: PathBuf,
) -> Result<OpenedProjectFile, ProjectFileError> {
    let project = read_project_file(&path)?;
    let file_name = project_file_name(&path)?;
    state.set_active_path(path);

    Ok(OpenedProjectFile { project, file_name })
}

pub fn save_project_to_active_path(
    state: &ProjectFileState,
    project: &ProjectFile,
) -> Result<String, ProjectFileError> {
    let path = state.active_path().ok_or(ProjectFileError::NoActivePath)?;
    save_project_at_path(state, path, project)
}

pub fn save_project_at_path(
    state: &ProjectFileState,
    path: PathBuf,
    project: &ProjectFile,
) -> Result<String, ProjectFileError> {
    write_project_file(&path, project)?;
    let file_name = project_file_name(&path)?;
    state.set_active_path(path);
    Ok(file_name)
}

fn project_file_name(path: &Path) -> Result<String, ProjectFileError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .ok_or(ProjectFileError::InvalidPath)
}

impl ProjectFileState {
    pub fn active_path(&self) -> Option<PathBuf> {
        self.active_path
            .lock()
            .expect("project file state mutex poisoned")
            .clone()
    }

    pub fn set_active_path(&self, path: PathBuf) {
        *self
            .active_path
            .lock()
            .expect("project file state mutex poisoned") = Some(path);
    }

    pub fn clear_active_path(&self) {
        *self
            .active_path
            .lock()
            .expect("project file state mutex poisoned") = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn serializes_project_file() {
        let project = ProjectFile::new("Example trial");
        let json = serde_json::to_string(&project).expect("serialize");
        assert!(json.contains("\"name\":\"Example trial\""));
    }

    #[test]
    fn civil_from_days_unix_epoch_is_1970_01_01() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn civil_from_days_handles_leap_day() {
        // 2000-02-29 is a leap day (2000 is divisible by 400).
        // Days from 1970-01-01 to 2000-02-29 = 11_016.
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
    }

    #[test]
    fn civil_from_days_handles_non_leap_century() {
        // 2100 is NOT a leap year (divisible by 100 but not 400), so Feb 2100
        // has 28 days and March 1st follows directly. Days from 1970-01-01 to
        // 2100-03-01 = 47_541.
        assert_eq!(civil_from_days(47_541), (2100, 3, 1));
    }

    #[test]
    fn iso_timestamp_produces_rfc3339_format() {
        let ts = iso_timestamp();
        // YYYY-MM-DDTHH:MM:SSZ — 20 chars, ends with Z.
        assert_eq!(ts.len(), 20);
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.chars().nth(4), Some('-'));
        assert_eq!(ts.chars().nth(10), Some('T'));
        assert_eq!(ts.chars().nth(13), Some(':'));
    }

    #[test]
    fn accepts_clinsize_project_paths() {
        assert!(is_project_file_path(Path::new("example.clinsize.json")));
    }

    #[test]
    fn rejects_non_project_file_paths() {
        assert!(!is_project_file_path(Path::new("example.json")));
        assert!(!is_project_file_path(Path::new("example.txt")));
    }

    #[test]
    fn project_file_round_trips_through_persistence_helpers() {
        let path = std::env::temp_dir().join(format!(
            "clinsize-project-{}-{}.clinsize.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("current time")
                .as_nanos()
        ));
        let expected = ProjectFile::new("Persistence test");

        write_project_file(&path, &expected).expect("write project file");
        let actual = read_project_file(&path).expect("read project file");
        std::fs::remove_file(&path).expect("remove temporary project file");

        assert_eq!(actual, expected);
    }

    #[test]
    fn saves_to_the_state_owned_project_path() {
        let path = std::env::temp_dir().join(format!(
            "clinsize-active-project-{}-{}.clinsize.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("current time")
                .as_nanos()
        ));
        let state = ProjectFileState::default();
        let project = ProjectFile::new("Active project");
        state.set_active_path(path.clone());

        let file_name = save_project_to_active_path(&state, &project).expect("save active project");
        let saved = read_project_file(&path).expect("read saved project");
        std::fs::remove_file(&path).expect("remove temporary project file");

        assert_eq!(
            file_name,
            path.file_name().expect("file name").to_string_lossy()
        );
        assert_eq!(saved, project);
    }

    #[test]
    fn opening_a_project_sets_its_state_owned_path() {
        let path = std::env::temp_dir().join(format!(
            "clinsize-open-project-{}-{}.clinsize.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("current time")
                .as_nanos()
        ));
        let state = ProjectFileState::default();
        let expected = ProjectFile::new("Opened project");
        write_project_file(&path, &expected).expect("write project file");

        let opened = open_project_at_path(&state, path.clone()).expect("open project");
        std::fs::remove_file(&path).expect("remove temporary project file");

        assert_eq!(opened.project, expected);
        assert_eq!(
            opened.file_name,
            path.file_name().expect("file name").to_string_lossy()
        );
        assert_eq!(state.active_path(), Some(path));
    }

    #[test]
    fn clearing_a_project_removes_its_state_owned_path() {
        let state = ProjectFileState::default();
        state.set_active_path(PathBuf::from("existing.clinsize.json"));

        state.clear_active_path();

        assert_eq!(state.active_path(), None);
    }
}
