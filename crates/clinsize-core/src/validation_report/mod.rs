//! Validation report generation from `validation/` evidence files.
//!
//! Each method directory holds a `cases.json`: externally referenced inputs
//! with expected outputs. Cases run through [`dispatch::calculate_json`], so
//! any registered method gains report support the moment its evidence file
//! lands — there is no per-method wiring to forget. The same files are
//! embedded at compile time (see `build.rs`) so packaged builds can generate
//! reports without the repository checkout.

use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

use crate::{dispatch, engine_version, registry, Error, Result};

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_cases.rs"));
}

/// Absolute tolerance applied when an expectation is a bare number.
/// Integer expectations (sample sizes, event counts) compare exactly at
/// this tolerance; pinned floats should use `{ "value": …, "tol": … }`.
const DEFAULT_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CaseFile {
    method_id: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    case_id: String,
    source: String,
    input: Value,
    /// Field path (`/`-separated for nested values, numeric segments index
    /// arrays) mapped to an expected value: a bare JSON literal or
    /// `{ "value": number, "tol": number }`.
    expect: serde_json::Map<String, Value>,
}

/// Evidence directory for a method identifier, relative to the validation
/// root: `binary.one_sample_binomial` → `binary/one-sample-binomial`.
pub fn method_dir(method_id: &str) -> String {
    method_id.replace('.', "/").replace('_', "-")
}

/// Method identifiers with embedded validation evidence.
pub fn embedded_method_ids() -> impl Iterator<Item = &'static str> {
    embedded::EMBEDDED_CASES.iter().map(|(id, _)| *id)
}

/// Generate a Markdown validation report from evidence files on disk.
pub fn generate_markdown(method_id: &str, validation_root: &Path) -> Result<String> {
    let path = validation_root
        .join(method_dir(method_id))
        .join("cases.json");
    let content = fs::read_to_string(&path).map_err(|err| {
        Error::Export(format!(
            "no validation evidence for {method_id}: failed to read {}: {err}",
            path.display()
        ))
    })?;
    generate_markdown_from_cases(method_id, &content)
}

/// Generate a Markdown validation report from evidence embedded at compile
/// time. Packaged builds must use this variant: the repository's
/// `validation/` directory does not ship with the application.
pub fn generate_markdown_embedded(method_id: &str) -> Result<String> {
    let content = embedded::EMBEDDED_CASES
        .iter()
        .find(|(id, _)| *id == method_id)
        .map(|(_, content)| *content)
        .ok_or_else(|| {
            Error::Export(format!(
                "no validation evidence recorded yet for {method_id}"
            ))
        })?;
    generate_markdown_from_cases(method_id, content)
}

/// Run every case in a `cases.json` document and render the results.
pub fn generate_markdown_from_cases(method_id: &str, cases_json: &str) -> Result<String> {
    let file: CaseFile = serde_json::from_str(cases_json)
        .map_err(|err| Error::Export(format!("invalid cases.json for {method_id}: {err}")))?;
    if file.method_id != method_id {
        return Err(Error::Export(format!(
            "cases.json declares method {} but {method_id} was requested",
            file.method_id
        )));
    }
    if file.cases.is_empty() {
        return Err(Error::Export(format!(
            "cases.json for {method_id} contains no cases"
        )));
    }

    let mut rows: Vec<String> = Vec::new();
    let mut sources: Vec<String> = Vec::new();
    let mut passed = 0usize;
    let mut failed = 0usize;

    for case in &file.cases {
        sources.push(format!("- `{}` — {}", case.case_id, case.source));
        let input_json =
            serde_json::to_string(&case.input).map_err(|err| Error::Export(err.to_string()))?;
        let result: Value = match dispatch::calculate_json(method_id, &input_json) {
            Ok(result_json) => {
                serde_json::from_str(&result_json).map_err(|err| Error::Export(err.to_string()))?
            }
            Err(err) => {
                failed += 1;
                rows.push(format!(
                    "| {} | — | calculation succeeds | error: {err} | fail |",
                    case.case_id
                ));
                continue;
            }
        };
        let mut case_ok = true;
        for (field, expectation) in &case.expect {
            let check = check_field(&result, field, expectation);
            case_ok &= check.pass;
            rows.push(format!(
                "| {} | {} | {} | {} | {} |",
                case.case_id,
                field,
                check.expected,
                check.actual,
                if check.pass { "pass" } else { "fail" }
            ));
        }
        if case_ok {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    let display_name = registry::list_methods()
        .iter()
        .find(|method| method.id == method_id)
        .map(|method| method.display_name)
        .unwrap_or(method_id);

    let mut lines = vec![
        format!("# Validation report: {display_name}"),
        String::new(),
        format!("- Engine version: {}", engine_version()),
        format!("- Method: `{method_id}`"),
        format!(
            "- Cases: {} ({passed} passed, {failed} failed)",
            file.cases.len()
        ),
        String::new(),
        "## Results".into(),
        String::new(),
        "| Case | Field | Expected | Actual | Status |".into(),
        "| --- | --- | --- | --- | --- |".into(),
    ];
    lines.extend(rows);
    lines.push(String::new());
    lines.push("## Reference sources".into());
    lines.push(String::new());
    lines.extend(sources);
    Ok(lines.join("\n"))
}

struct FieldCheck {
    expected: String,
    actual: String,
    pass: bool,
}

fn check_field(result: &Value, field: &str, expectation: &Value) -> FieldCheck {
    let (expected_value, tolerance) = match expectation {
        Value::Object(spec) => {
            let value = spec.get("value").cloned().unwrap_or(Value::Null);
            let tolerance = spec
                .get("tol")
                .and_then(Value::as_f64)
                .unwrap_or(DEFAULT_TOLERANCE);
            (value, tolerance)
        }
        other => (other.clone(), DEFAULT_TOLERANCE),
    };

    let expected_display = match expectation {
        Value::Object(_) => format!("{expected_value} ± {tolerance:e}"),
        other => other.to_string(),
    };

    let Some(actual) = lookup(result, field) else {
        return FieldCheck {
            expected: expected_display,
            actual: "missing".into(),
            pass: false,
        };
    };

    let pass = match (expected_value.as_f64(), actual.as_f64()) {
        (Some(expected), Some(actual)) => (actual - expected).abs() <= tolerance,
        _ => &expected_value == actual,
    };

    FieldCheck {
        expected: expected_display,
        actual: actual.to_string(),
        pass,
    }
}

fn lookup<'a>(result: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = result;
    for segment in path.split('/') {
        current = match current {
            Value::Object(map) => map.get(segment)?,
            Value::Array(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn validation_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../validation")
    }

    #[test]
    fn every_embedded_case_passes() {
        let mut checked = 0;
        for method_id in embedded_method_ids() {
            let report = generate_markdown_embedded(method_id)
                .unwrap_or_else(|err| panic!("report for {method_id}: {err}"));
            assert!(
                !report.contains("| fail |"),
                "{method_id} has failing validation cases:\n{report}"
            );
            assert!(report.contains("| pass |"), "{method_id} checked nothing");
            checked += 1;
        }
        assert!(
            checked >= 19,
            "expected >= 19 evidence files, got {checked}"
        );
    }

    #[test]
    fn embedded_evidence_matches_registry() {
        for method_id in embedded_method_ids() {
            assert!(
                registry::list_methods()
                    .iter()
                    .any(|method| method.id == method_id),
                "embedded evidence for unregistered method {method_id}"
            );
        }
    }

    #[test]
    fn embedded_evidence_matches_disk() {
        // Catches a stale build cache: build.rs re-runs on evidence changes,
        // but nothing else guarantees the embedded copies are current.
        for (method_id, embedded_content) in embedded::EMBEDDED_CASES {
            let path = validation_root()
                .join(method_dir(method_id))
                .join("cases.json");
            let disk_content = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            assert_eq!(
                *embedded_content, disk_content,
                "embedded evidence for {method_id} is stale; run cargo clean -p clinsize-core"
            );
        }
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

    #[test]
    fn unknown_method_reports_missing_evidence() {
        let err = generate_markdown_embedded("continuous.does_not_exist").unwrap_err();
        assert!(err.to_string().contains("no validation evidence"));
    }
}
