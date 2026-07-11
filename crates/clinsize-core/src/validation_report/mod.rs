//! Validation report generation from `validation/` evidence files.

use std::fs;
use std::path::Path;

use crate::methods::continuous::two_sample_ttest::{self, TwoSampleTTestInput};
use crate::methods::survival::log_rank::{self, LogRankInput};
use crate::types::{Alternative, SolveMode};
use crate::{engine_version, Error, Result};

/// Generate a Markdown validation report for a registered method.
pub fn generate_markdown(method_id: &str, validation_root: &Path) -> Result<String> {
    match method_id {
        "continuous.two_sample_ttest" => {
            generate_two_sample_ttest_report(validation_root.join("continuous/two-sample-ttest"))
        }
        "survival.log_rank" => generate_log_rank_report(validation_root.join("survival/log-rank")),
        other => Err(Error::UnsupportedMethod(format!(
            "validation report generation is not implemented for {other}"
        ))),
    }
}

fn generate_two_sample_ttest_report(method_dir: std::path::PathBuf) -> Result<String> {
    let cases_path = method_dir.join("cases.csv");
    let rows = read_csv_rows(&cases_path)?;
    let mut lines = report_header("Two-sample t-test", "continuous.two_sample_ttest");

    lines.push(
        "| Case | Expected control N | Actual control N | Expected power | Actual power | Status |"
            .into(),
    );
    lines.push("| --- | --- | --- | --- | --- | --- |".into());

    for row in &rows {
        let input = TwoSampleTTestInput {
            solve_mode: SolveMode::SampleSize,
            alpha: parse_f64(row, "alpha")?,
            power: Some(parse_f64(row, "power")?),
            control_n: None,
            mean_difference: parse_f64(row, "mean_difference")?,
            standard_deviation: parse_f64(row, "standard_deviation")?,
            allocation_ratio: parse_f64(row, "allocation_ratio")?,
            alternative: parse_alternative(row.get("alternative"))?,
            dropout_rate: parse_optional_f64(row, "dropout_rate"),
        };
        let result = two_sample_ttest::calculate(input)?;
        let expected_n = parse_u32(row, "expected_n_control")?;
        let expected_power = parse_optional_f64(row, "expected_achieved_power");
        let tolerance = row
            .get("tolerance")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(1e-4);
        let power_ok = expected_power
            .map(|expected| (result.achieved_power - expected).abs() <= tolerance)
            .unwrap_or(true);
        let status = if result.n_control == expected_n && power_ok {
            "pass"
        } else {
            "fail"
        };
        lines.push(format!(
            "| {} | {} | {} | {} | {:.5} | {} |",
            row.get("case_id").cloned().unwrap_or_default(),
            expected_n,
            result.n_control,
            expected_power
                .map(|value| format!("{value:.5}"))
                .unwrap_or_else(|| "—".into()),
            result.achieved_power,
            status
        ));
    }

    lines.push(String::new());
    lines.push("Automated coverage: `cargo test -p clinsize-core two_sample_ttest`".into());
    Ok(lines.join("\n"))
}

fn generate_log_rank_report(method_dir: std::path::PathBuf) -> Result<String> {
    let cases_path = method_dir.join("cases.csv");
    let rows = read_csv_rows(&cases_path)?;
    let mut lines = report_header("Log-rank test", "survival.log_rank");

    lines.push("| Case | Expected events | Actual events | Expected total N | Actual total N | Expected power | Actual power | Status |".into());
    lines.push("| --- | --- | --- | --- | --- | --- | --- | --- |".into());

    for row in &rows {
        let solve_mode = parse_solve_mode(row.get("solve_mode"))?;
        let input = LogRankInput {
            solve_mode,
            alpha: parse_f64(row, "alpha")?,
            power: parse_optional_f64(row, "power"),
            total_events: parse_optional_u32(row, "total_events"),
            hazard_ratio: parse_f64(row, "hazard_ratio")?,
            allocation_ratio: parse_f64(row, "allocation_ratio")?,
            alternative: parse_alternative(row.get("alternative"))?,
            control_hazard_rate: parse_optional_f64(row, "control_hazard_rate"),
            accrual_duration: parse_optional_f64(row, "accrual_duration"),
            minimum_follow_up: parse_optional_f64(row, "minimum_follow_up"),
            dropout_hazard_rate: parse_optional_f64(row, "dropout_hazard_rate"),
        };
        let result = log_rank::calculate(input)?;
        let expected_events = parse_optional_u32(row, "expected_required_events");
        let expected_total_n = parse_optional_u32(row, "expected_total_n");
        let expected_power = parse_optional_f64(row, "expected_achieved_power");
        let events_ok = expected_events
            .map(|expected| result.required_events == expected)
            .unwrap_or(true);
        let total_n_ok = expected_total_n
            .map(|expected| result.total_n == Some(expected))
            .unwrap_or(true);
        let power_ok = expected_power
            .map(|expected| (result.achieved_power - expected).abs() <= 1e-4)
            .unwrap_or(true);
        let status = if events_ok && total_n_ok && power_ok {
            "pass"
        } else {
            "fail"
        };
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {:.5} | {} |",
            row.get("case_id").cloned().unwrap_or_default(),
            expected_events
                .map(|value| value.to_string())
                .unwrap_or_else(|| "—".into()),
            result.required_events,
            expected_total_n
                .map(|value| value.to_string())
                .unwrap_or_else(|| "—".into()),
            result
                .total_n
                .map(|value| value.to_string())
                .unwrap_or_else(|| "—".into()),
            expected_power
                .map(|value| format!("{value:.5}"))
                .unwrap_or_else(|| "—".into()),
            result.achieved_power,
            status
        ));
    }

    lines.push(String::new());
    lines.push("Automated coverage: `cargo test -p clinsize-core survival`".into());
    Ok(lines.join("\n"))
}

fn report_header(method_name: &str, method_id: &str) -> Vec<String> {
    vec![
        format!("# {method_name} validation report"),
        String::new(),
        "## Method version".into(),
        format!("- ClinSize engine: {}", engine_version()),
        format!("- Method identifier: `{method_id}`"),
        String::new(),
        "## Results".into(),
    ]
}

fn read_csv_rows(path: &Path) -> Result<Vec<std::collections::HashMap<String, String>>> {
    let content = fs::read_to_string(path).map_err(|err| Error::Export(err.to_string()))?;
    let mut lines = content.lines();
    let header = lines
        .next()
        .ok_or_else(|| Error::Export("cases.csv is empty".into()))?;
    let columns: Vec<&str> = header.split(',').collect();

    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values: Vec<&str> = line.split(',').collect();
        let mut row = std::collections::HashMap::new();
        for (index, column) in columns.iter().enumerate() {
            if let Some(value) = values.get(index) {
                row.insert((*column).to_string(), (*value).to_string());
            }
        }
        rows.push(row);
    }
    Ok(rows)
}

fn parse_f64(row: &std::collections::HashMap<String, String>, key: &str) -> Result<f64> {
    row.get(key)
        .ok_or_else(|| Error::Export(format!("missing column {key}")))?
        .parse::<f64>()
        .map_err(|_| Error::Export(format!("invalid numeric value for {key}")))
}

fn parse_u32(row: &std::collections::HashMap<String, String>, key: &str) -> Result<u32> {
    row.get(key)
        .ok_or_else(|| Error::Export(format!("missing column {key}")))?
        .parse::<u32>()
        .map_err(|_| Error::Export(format!("invalid integer value for {key}")))
}

fn parse_optional_f64(row: &std::collections::HashMap<String, String>, key: &str) -> Option<f64> {
    row.get(key)
        .filter(|value| !value.trim().is_empty())
        .and_then(|value| value.parse::<f64>().ok())
}

fn parse_optional_u32(row: &std::collections::HashMap<String, String>, key: &str) -> Option<u32> {
    row.get(key)
        .filter(|value| !value.trim().is_empty())
        .and_then(|value| value.parse::<u32>().ok())
}

fn parse_alternative(value: Option<&String>) -> Result<Alternative> {
    match value.map(String::as_str) {
        Some("two_sided") => Ok(Alternative::TwoSided),
        Some("greater") => Ok(Alternative::Greater),
        Some("less") => Ok(Alternative::Less),
        other => Err(Error::Export(format!(
            "unsupported alternative: {}",
            other.unwrap_or("missing")
        ))),
    }
}

fn parse_solve_mode(value: Option<&String>) -> Result<SolveMode> {
    match value.map(String::as_str) {
        Some("sample_size") => Ok(SolveMode::SampleSize),
        Some("power") => Ok(SolveMode::Power),
        other => Err(Error::Export(format!(
            "unsupported solve mode: {}",
            other.unwrap_or("missing")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn validation_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation")
    }

    #[test]
    fn generates_two_sample_ttest_report() {
        let report = generate_markdown("continuous.two_sample_ttest", &validation_root())
            .expect("generate report");
        assert!(report.contains("equal_two_sided_d1"));
        assert!(report.contains("| pass |"));
    }

    #[test]
    fn generates_log_rank_report() {
        let report =
            generate_markdown("survival.log_rank", &validation_root()).expect("generate report");
        assert!(report.contains("logrank_two_sided_80"));
        assert!(report.contains("| pass |"));
    }
}
