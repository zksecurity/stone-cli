mod args;

use args::Cairo1Args;
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct Cairo1RunResult {
    pub air_public_input: PathBuf,
    pub air_private_input: PathBuf,
    pub memory_file: PathBuf,
    pub trace_file: PathBuf,
}

/// Runs a Cairo 1 program and generates proof-related files.
///
/// # Arguments
///
/// * `args` - An iterator over command-line arguments
/// * `tmp_dir` - A temporary directory to store output files
///
/// # Returns
///
/// A `Result` containing a `Cairo1RunResult` on success, or an `anyhow::Error` on failure.
pub fn run_cairo1(
    args: impl Iterator<Item = String>,
    tmp_dir: &tempfile::TempDir,
) -> Result<Cairo1RunResult, anyhow::Error> {
    let mut parsed_args = Cairo1Args::try_parse_from(args)?;
    let filename = parsed_args.filename.file_stem().unwrap().to_str().unwrap();

    // Set default file paths
    let mut default_paths = [
        (&mut parsed_args.trace_file, "_trace.json"),
        (&mut parsed_args.memory_file, "_memory.json"),
        (&mut parsed_args.air_public_input, "_air_public_input.json"),
        (
            &mut parsed_args.air_private_input,
            "_air_private_input.json",
        ),
    ];

    for (arg, suffix) in default_paths.iter_mut() {
        if arg.is_none() {
            **arg = Some(tmp_dir.path().join(format!("{}{}", filename, suffix)));
        }
    }

    let cairo1_run_path = std::env::var("CAIRO1_RUN")
        .map_err(|e| anyhow::anyhow!("Failed to get CAIRO1_RUN environment variable: {}", e))?;
    let mut command = Command::new(cairo1_run_path);
    command
        .arg(&parsed_args.filename)
        .arg("--layout")
        .arg(parsed_args.layout.to_str());

    // Add optional arguments
    let optional_args = [
        ("--trace_file", &parsed_args.trace_file),
        ("--memory_file", &parsed_args.memory_file),
        ("--air_public_input", &parsed_args.air_public_input),
        ("--air_private_input", &parsed_args.air_private_input),
        ("--cairo_pie_output", &parsed_args.cairo_pie_output),
        ("--args_file", &parsed_args.args_file),
    ];

    for (arg, value) in optional_args.iter() {
        if let Some(v) = value {
            command.arg(arg).arg(v);
        }
    }

    // `args` is of different type than the others, so we need to handle it separately
    if let Some(args) = &parsed_args.args {
        command.arg("--args").arg(args);
    }

    // Add flag arguments
    if parsed_args.proof_mode {
        command.arg("--proof_mode");
    }
    if parsed_args.print_output {
        command.arg("--print_output");
    }
    if parsed_args.append_return_values {
        command.arg("--append_return_values");
    }

    let output = command.output().expect("Failed to execute cairo1-run");

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "cairo1-run failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    } else {
        println!("cairo1-run executed successfully.");
    }

    Ok(Cairo1RunResult {
        memory_file: parsed_args.memory_file.unwrap(),
        trace_file: parsed_args.trace_file.unwrap(),
        air_public_input: parsed_args.air_public_input.unwrap(),
        air_private_input: parsed_args.air_private_input.unwrap(),
    })
}
