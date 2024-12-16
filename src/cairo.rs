use crate::args::{CairoVersion, LayoutName, ProveArgs};
use crate::utils::{get_formatted_air_public_input, FileWriter};
use cairo_vm::air_public_input::PublicInputError;
use cairo_vm::cairo_run::{
    cairo_run_program, write_encoded_memory, write_encoded_trace, CairoRunConfig, EncodeTraceError,
};
use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor;
use cairo_vm::types::errors::program_errors::ProgramError;
use cairo_vm::types::program::Program;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::trace_errors::TraceError;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Debug)]
pub struct CairoRunResult {
    pub air_public_input: PathBuf,
    pub air_private_input: PathBuf,
    pub memory_file: PathBuf,
    pub trace_file: PathBuf,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to interact with the file system")]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Runner(#[from] CairoRunError),
    #[error(transparent)]
    EncodeTrace(#[from] EncodeTraceError),
    #[error(transparent)]
    Trace(#[from] TraceError),
    #[error(transparent)]
    PublicInput(#[from] PublicInputError),
    #[error(transparent)]
    Program(#[from] ProgramError),
}

pub fn run_cairo(
    args: &ProveArgs,
    tmp_dir: &tempfile::TempDir,
) -> Result<CairoRunResult, anyhow::Error> {
    match args.cairo_version {
        CairoVersion::cairo0 => run_cairo0(args, tmp_dir).map_err(|e| anyhow::anyhow!(e)),
        CairoVersion::cairo1 => run_cairo1(args, tmp_dir),
    }
}

/// Runs a Cairo 0 program and generates the necessary outputs for proving
///
/// # Arguments
///
/// * `prove_args` - The arguments for the prove command
/// * `tmp_dir` - A temporary directory to store intermediate files
///
/// # Returns
///
/// A `Result` containing `CairoRunResult` on success, or an `Error` on failure
///
/// # Errors
///
/// This function can return various errors related to file I/O, program execution,
/// trace encoding, and public input generation.
pub fn run_cairo0(
    prove_args: &ProveArgs,
    tmp_dir: &tempfile::TempDir,
) -> Result<CairoRunResult, Error> {
    let filename = prove_args
        .cairo_program
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let program = Program::from_file(&prove_args.cairo_program, Some("main"))?;

    let mut hint_processor = BuiltinHintProcessor::new_empty();

    let cairo_run_config = CairoRunConfig {
        entrypoint: "main",
        trace_enabled: true,
        relocate_mem: true,
        layout: get_layout(&prove_args.layout),
        proof_mode: true,
        secure_run: None,
        disable_trace_padding: false,
        allow_missing_builtins: None,
        dynamic_layout_params: None, // TODO
    };

    let runner = cairo_run_program(&program, &cairo_run_config, &mut hint_processor)?;
    let relocated_trace = runner
        .relocated_trace
        .as_ref()
        .ok_or(Error::Trace(TraceError::TraceNotRelocated))?;

    let trace_path = tmp_dir.path().join(format!("{}_trace.json", filename));
    let trace_file = std::fs::File::create(trace_path.clone())?;
    let mut trace_writer =
        FileWriter::new(io::BufWriter::with_capacity(3 * 1024 * 1024, trace_file));
    write_encoded_trace(relocated_trace, &mut trace_writer)?;
    trace_writer.flush()?;

    let memory_path = tmp_dir.path().join(format!("{}_memory.json", filename));
    let memory_file = std::fs::File::create(memory_path.clone())?;
    let mut memory_writer =
        FileWriter::new(io::BufWriter::with_capacity(5 * 1024 * 1024, memory_file));
    write_encoded_memory(&runner.relocated_memory, &mut memory_writer)?;
    memory_writer.flush()?;

    let air_public_input_path = tmp_dir
        .path()
        .join(format!("{}_air_public_input.json", filename));
    let air_public_input_str = get_formatted_air_public_input(&runner.get_air_public_input()?)?;
    std::fs::write(air_public_input_path.clone(), air_public_input_str)?;

    let air_private_input_path = tmp_dir
        .path()
        .join(format!("{}_air_private_input.json", filename));
    let trace_absolute_path = trace_path
        .as_path()
        .canonicalize()
        .unwrap_or(trace_path.clone())
        .to_str()
        .unwrap()
        .to_string();
    let memory_absolute_path = memory_path
        .as_path()
        .canonicalize()
        .unwrap_or(memory_path.clone())
        .to_str()
        .unwrap()
        .to_string();
    let air_private_input = runner
        .get_air_private_input()
        .to_serializable(trace_absolute_path, memory_absolute_path)
        .serialize_json()
        .map_err(PublicInputError::Serde)?;
    std::fs::write(air_private_input_path.clone(), air_private_input)?;

    Ok(CairoRunResult {
        air_public_input: air_public_input_path,
        air_private_input: air_private_input_path,
        memory_file: memory_path,
        trace_file: trace_path,
    })
}

/// Runs a Cairo 1 program and generates the necessary outputs for proving
///
/// # Arguments
///
/// * `prove_args` - The arguments for the prove command
/// * `tmp_dir` - A temporary directory to store intermediate files
///
/// # Returns
///
/// A `Result` containing `CairoRunResult` on success, or an `anyhow::Error` on failure
///
/// # Note
///
/// This function ignores the following arguments to cairo1-run: `append_return_values`, `cairo_pie_output`, `print_output`.
pub fn run_cairo1(
    prove_args: &ProveArgs,
    tmp_dir: &tempfile::TempDir,
) -> Result<CairoRunResult, anyhow::Error> {
    let filename = prove_args
        .cairo_program
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let cairo1_run_path = std::env::var("CAIRO1_RUN")
        .map_err(|e| anyhow::anyhow!("Failed to get CAIRO1_RUN environment variable: {}", e))?;

    let mut cmd = Command::new(cairo1_run_path);
    cmd.arg(&prove_args.cairo_program);

    match &prove_args.layout {
        LayoutName::automatic => {
            cmd.arg("--layout").arg("dynamic");
            cmd.arg("--cairo_layout_params_file")
                .arg("cairo_layout_params_file.json");
        }
        layout => {
            cmd.arg("--layout").arg(layout.clone().to_str());
        }
    }

    // Set default file paths using tmp_dir
    let trace_file = tmp_dir.path().join(format!("{}_trace.json", filename));
    let memory_file = tmp_dir.path().join(format!("{}_memory.json", filename));
    let air_public_input = tmp_dir
        .path()
        .join(format!("{}_air_public_input.json", filename));
    let air_private_input = tmp_dir
        .path()
        .join(format!("{}_air_private_input.json", filename));

    cmd.arg("--trace_file") //
        .arg(trace_file.clone());
    cmd.arg("--memory_file") //
        .arg(memory_file.clone());
    cmd.arg("--air_public_input") //
        .arg(air_public_input.clone());
    cmd.arg("--air_private_input")
        .arg(air_private_input.clone());

    if let Some(args_file) = &prove_args.program_input_file {
        cmd.arg("--args_file").arg(args_file.to_str().unwrap());
    }

    if let Some(args) = &prove_args.program_input {
        cmd.arg("--args").arg(args);
    }

    cmd.arg("--proof_mode");

    println!("Running cairo1-run... {:?}", cmd);

    let output = cmd.output().expect("Failed to execute cairo1-run");

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "cairo1-run failed with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    } else {
        println!("cairo1-run executed successfully.");
    }

    Ok(CairoRunResult {
        air_public_input,
        air_private_input,
        memory_file,
        trace_file,
    })
}

fn get_layout(layout: &LayoutName) -> cairo_vm::types::layout_name::LayoutName {
    match layout {
        LayoutName::dynamic => cairo_vm::types::layout_name::LayoutName::dynamic,
        LayoutName::automatic => cairo_vm::types::layout_name::LayoutName::dynamic,
        LayoutName::all_cairo => cairo_vm::types::layout_name::LayoutName::all_cairo,
        LayoutName::all_solidity => cairo_vm::types::layout_name::LayoutName::all_solidity,
        LayoutName::recursive_with_poseidon => {
            cairo_vm::types::layout_name::LayoutName::recursive_with_poseidon
        }
        LayoutName::recursive_large_output => {
            cairo_vm::types::layout_name::LayoutName::recursive_large_output
        }
        LayoutName::starknet_with_keccak => {
            cairo_vm::types::layout_name::LayoutName::starknet_with_keccak
        }
        LayoutName::starknet => cairo_vm::types::layout_name::LayoutName::starknet,
        LayoutName::recursive => cairo_vm::types::layout_name::LayoutName::recursive,
        LayoutName::dex => cairo_vm::types::layout_name::LayoutName::dex,
        LayoutName::small => cairo_vm::types::layout_name::LayoutName::small,
        LayoutName::plain => cairo_vm::types::layout_name::LayoutName::plain,
    }
}
