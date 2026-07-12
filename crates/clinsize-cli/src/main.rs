use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use clinsize_core::{dispatch, engine_version, registry, validation_report, Error};

#[derive(Parser)]
#[command(
    name = "clinsize",
    version,
    about = "Clinical trial sample size and power calculations"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List registered calculation methods.
    List,
    /// Print the core engine version.
    Version,
    /// Run a calculation from a JSON input file.
    Calculate {
        /// Registered method identifier (see `clinsize list`).
        #[arg(long)]
        method: String,
        /// JSON file containing method inputs.
        #[arg(long)]
        input: PathBuf,
        /// Optional file to write JSON results. Defaults to stdout.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Render a Markdown calculation summary from JSON input and result files.
    Report {
        #[arg(long)]
        method: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        result: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Generate a validation report for a supported method.
    ValidationReport {
        #[arg(long)]
        method: String,
        /// Root of the `validation/` directory.
        #[arg(long, default_value = "validation")]
        validation_root: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Commands::List => cmd_list(),
        Commands::Version => {
            println!("{}", engine_version());
            Ok(())
        }
        Commands::Calculate {
            method,
            input,
            output,
        } => cmd_calculate(&method, &input, output.as_deref()),
        Commands::Report {
            method,
            input,
            result,
            output,
        } => cmd_report(&method, &input, &result, output.as_deref()),
        Commands::ValidationReport {
            method,
            validation_root,
            output,
        } => cmd_validation_report(&method, &validation_root, output.as_deref()),
    }
}

fn cmd_list() -> Result<(), String> {
    for method in registry::list_methods() {
        let modes_str = if method.supported_solve_modes.is_empty() {
            "n/a".to_string()
        } else {
            method
                .supported_solve_modes
                .iter()
                .map(|mode| format!("{mode:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        println!(
            "{}\t{}\t{}\t{}",
            method.id, method.display_name, method.endpoint_category, modes_str
        );
    }
    Ok(())
}

fn cmd_calculate(
    method: &str,
    input_path: &Path,
    output_path: Option<&Path>,
) -> Result<(), String> {
    let input_json = read_file(input_path)?;
    let result_json = dispatch::calculate_json(method, &input_json).map_err(format_error)?;
    write_output(output_path, &result_json)
}

fn cmd_report(
    method: &str,
    input_path: &Path,
    result_path: &Path,
    output_path: Option<&Path>,
) -> Result<(), String> {
    let input_json = read_file(input_path)?;
    let result_json = read_file(result_path)?;
    let markdown =
        dispatch::report_markdown_json(method, &input_json, &result_json).map_err(format_error)?;
    write_output(output_path, &markdown)
}

fn cmd_validation_report(
    method: &str,
    validation_root: &Path,
    output_path: Option<&Path>,
) -> Result<(), String> {
    let markdown =
        validation_report::generate_markdown(method, validation_root).map_err(format_error)?;
    write_output(output_path, &markdown)
}

fn read_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("failed to read {}: {err}", path.display()))
}

fn write_output(path: Option<&Path>, content: &str) -> Result<(), String> {
    match path {
        Some(path) => fs::write(path, content)
            .map_err(|err| format!("failed to write {}: {err}", path.display())),
        None => {
            io::stdout()
                .write_all(content.as_bytes())
                .map_err(|err| format!("failed to write stdout: {err}"))?;
            if !content.ends_with('\n') {
                io::stdout()
                    .write_all(b"\n")
                    .map_err(|err| format!("failed to write stdout: {err}"))?;
            }
            Ok(())
        }
    }
}

fn format_error(err: Error) -> String {
    err.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn list_includes_all_methods() {
        // The exact method count is pinned by the registry's own tests;
        // duplicating it here is what let this test go stale.
        let lines: Vec<String> = registry::list_methods()
            .iter()
            .map(|method| method.id.to_string())
            .collect();
        assert!(!lines.is_empty());
        assert!(lines.contains(&"continuous.two_sample_ttest".to_string()));
        assert!(lines.contains(&"survival.log_rank".to_string()));
        assert!(lines.contains(&"design.blinded_ssre".to_string()));
    }

    #[test]
    fn calculate_command_round_trip() {
        let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/continuous/two-sample-ttest/sample-size.json");
        cmd_calculate("continuous.two_sample_ttest", &input, None).expect("calculate");
    }
}
