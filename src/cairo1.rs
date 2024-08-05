use crate::args::ProveArgs;
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
/// A `Result` containing a `Cairo1RunResult` on success, or an `anyhow::Error` on failure
///
/// # Note
///
/// This function ignores the following arguments to cairo1-run: `append_return_values`, `cairo_pie_output`, `print_output`.
pub fn run_cairo1(
    prove_args: &ProveArgs,
    tmp_dir: &tempfile::TempDir,
) -> Result<Cairo1RunResult, anyhow::Error> {
    let filename = prove_args
        .cairo_program
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let cairo1_run_path = std::env::var("CAIRO1_RUN")
        .map_err(|e| anyhow::anyhow!("Failed to get CAIRO1_RUN environment variable: {}", e))?;
    let mut command = Command::new(cairo1_run_path);
    command
        .arg(&prove_args.cairo_program)
        .arg("--layout")
        .arg(prove_args.layout.clone().to_str());

    // Set default file paths using tmp_dir
    let trace_file = tmp_dir.path().join(format!("{}_trace.json", filename));
    let memory_file = tmp_dir.path().join(format!("{}_memory.json", filename));
    let air_public_input = tmp_dir
        .path()
        .join(format!("{}_air_public_input.json", filename));
    let air_private_input = tmp_dir
        .path()
        .join(format!("{}_air_private_input.json", filename));
    let tmp_file_args = [
        ("--trace_file", &trace_file),
        ("--memory_file", &memory_file),
        ("--air_public_input", &air_public_input),
        ("--air_private_input", &air_private_input),
    ];
    for (arg, value) in tmp_file_args.iter() {
        command.arg(arg).arg(value);
    }

    if let Some(args_file) = &prove_args.program_input_file {
        command.arg("--args_file").arg(args_file);
    }
    if let Some(args) = &prove_args.program_input {
        command.arg("--args").arg(args);
    }
    command.arg("--proof_mode");

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
        memory_file,
        trace_file,
        air_public_input,
        air_private_input,
    })
}
