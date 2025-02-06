use crate::args::{CairoVersion, LayoutName, ProveArgs};
use crate::utils::{get_formatted_air_public_input, process_args, Config, FileWriter, FuncArgs};
use cairo1_run::FuncArg;
use cairo1_run::{cairo_run_program as cairo_run_program_cairo1, Cairo1RunConfig, CairoRunner};
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::setup_project;
use cairo_lang_compiler::{compile_prepared_db, CompilerConfig};
use cairo_lang_filesystem::db::init_dev_corelib;
use cairo_vm::air_public_input::PublicInputError;
use cairo_vm::cairo_run::{
    cairo_run_program, write_encoded_memory, write_encoded_trace, CairoRunConfig, EncodeTraceError,
};
use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{
    BuiltinHintProcessor, HintFunc,
};
use cairo_vm::hint_processor::builtin_hint_processor::hint_utils::insert_value_from_var_name;
use cairo_vm::types::errors::program_errors::ProgramError;
use cairo_vm::types::layout::CairoLayoutParams;
use cairo_vm::types::program::Program;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::trace_errors::TraceError;
use cairo_vm::Felt252;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use thiserror::Error;

// TODO: get the correct one
const DYNAMIC_LAYOUT: &'static str = r#"{
    "rc_units": 16,
    "memory_units_per_step": 8,
    "public_memory_fraction": 4,
    "log_diluted_units_per_step": 4,
    "cpu_component_step": 8,
    "uses_pedersen_builtin": true,
    "pedersen_ratio": 256,
    "uses_range_check_builtin": true,
    "range_check_ratio": 8,
    "uses_ecdsa_builtin": true,
    "ecdsa_ratio": 2048,
    "uses_bitwise_builtin": true,
    "bitwise_ratio": 16,
    "uses_ec_op_builtin": true,
    "ec_op_ratio": 1024,
    "uses_keccak_builtin": true,
    "keccak_ratio": 2048,
    "uses_poseidon_builtin": true,
    "poseidon_ratio": 256,
    "uses_range_check96_builtin": true,
    "range_check96_ratio": 8,
    "range_check96_ratio_den": 1,
    "uses_add_mod_builtin": true,
    "add_mod_ratio": 128,
    "add_mod_ratio_den": 1,
    "uses_mul_mod_builtin": true,
    "mul_mod_ratio": 256,
    "mul_mod_ratio_den": 1
}"#;

const CONFIG: &str = include_str!("../configs/env.json");

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
    #[error(transparent)]
    ProgramInput(#[from] serde_json::Error),
}

pub fn run_cairo(
    args: &ProveArgs,
    tmp_dir: &tempfile::TempDir,
) -> Result<CairoRunResult, anyhow::Error> {
    match args.cairo_version {
        CairoVersion::cairo0 => run_cairo0(args, tmp_dir).map_err(Into::into),
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
) -> Result<CairoRunResult, anyhow::Error> {
    let filename = prove_args
        .cairo_program
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    let program = Program::from_file(&prove_args.cairo_program, Some("main"))?;
    let program_input = if let Some(program_input_file) = prove_args.program_input_file.clone() {
        let program_input_file_str = std::fs::read_to_string(program_input_file)?;
        serde_json::from_str::<HashMap<String, serde_json::Value>>(&program_input_file_str)?
    } else {
        HashMap::new()
    };

    let mut hint_processor = BuiltinHintProcessor::new_empty();
    let program_input_clone = program_input.clone();
    hint_processor.add_hint(
        "ids.fibonacci_claim_index = program_input['fibonacci_claim_index']".to_string(),
        Rc::new(HintFunc(Box::new(
            move |vm, _exec_scopes, ids_data, ap_tracking, _constants| {
                let fibonacci_claim_index = program_input
                    .get("fibonacci_claim_index")
                    .unwrap()
                    .as_u64()
                    .unwrap();
                insert_value_from_var_name(
                    "fibonacci_claim_index",
                    Felt252::from(fibonacci_claim_index),
                    vm,
                    ids_data,
                    ap_tracking,
                )?;
                Ok(())
            },
        ))),
    );
    hint_processor.add_hint(
        "ids.iterations = program_input['iterations']".to_string(),
        Rc::new(HintFunc(Box::new(
            move |vm, _exec_scopes, ids_data, ap_tracking, _constants| {
                let iterations = program_input_clone
                    .get("iterations")
                    .unwrap()
                    .as_u64()
                    .unwrap();
                insert_value_from_var_name(
                    "iterations",
                    Felt252::from(iterations),
                    vm,
                    ids_data,
                    ap_tracking,
                )?;
                Ok(())
            },
        ))),
    );

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
    let file_paths = write_to_files(&runner, tmp_dir, filename)?;
    Ok(file_paths)
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

    let args = if let Some(program_input_file) = &prove_args.program_input_file {
        let file_content = std::fs::read_to_string(program_input_file)?;
        process_args(&file_content).unwrap()
    } else {
        prove_args.program_input.clone()
    };

    let cairo_run_config = match &prove_args.layout {
        LayoutName::dynamic | LayoutName::automatic => {
            let cairo_layout_params_file = tmp_dir.path().join("cairo_layout_params_file.json");
            std::fs::write(cairo_layout_params_file.clone(), DYNAMIC_LAYOUT)?;
            Cairo1RunConfig {
                proof_mode: true,
                serialize_output: true,
                relocate_mem: true,
                layout: cairo_vm::types::layout_name::LayoutName::dynamic,
                trace_enabled: true,
                args: &args.0,
                finalize_builtins: true,
                append_return_values: true,
                dynamic_layout_params: Some(CairoLayoutParams::from_file(
                    cairo_layout_params_file.as_path(),
                )?),
            }
        }
        layout => Cairo1RunConfig {
            proof_mode: true,
            serialize_output: true,
            relocate_mem: true,
            layout: layout.to_cairo_vm_layout(),
            trace_enabled: true,
            args: &args.0,
            finalize_builtins: true,
            append_return_values: true,
            dynamic_layout_params: None,
        },
    };

    // Try to parse the file as a sierra program
    let file = std::fs::read(&prove_args.cairo_program)?;
    let sierra_program = match serde_json::from_slice(&file) {
        Ok(program) => program,
        Err(_) => {
            // If it fails, try to compile it as a cairo program
            let compiler_config = CompilerConfig {
                replace_ids: true,
                ..CompilerConfig::default()
            };
            let mut db = RootDatabase::builder()
                .skip_auto_withdraw_gas()
                .build()
                .unwrap();

            let config: Config = serde_json::from_str(CONFIG).expect("Failed to parse config file");
            let download_dir = Path::new(env!("HOME")).join(&config.download_dir);
            let corelib_dir = download_dir.join("corelib");
            if !corelib_dir.exists() {
                anyhow::bail!("Corelib directory does not exist: {:?}", corelib_dir);
            }
            init_dev_corelib(&mut db, corelib_dir.join("src"));
            let main_crate_ids = setup_project(&mut db, &prove_args.cairo_program).unwrap();
            let sierra_program_with_dbg =
                compile_prepared_db(&db, main_crate_ids, compiler_config).unwrap();
            sierra_program_with_dbg.program
        }
    };

    let (runner, _, serialized_output) =
        cairo_run_program_cairo1(&sierra_program, cairo_run_config)?;
    println!("Cairo1 program output: {:?}", serialized_output);

    let file_paths = write_to_files(&runner, tmp_dir, filename)?;
    Ok(file_paths)
}

pub fn get_cairo_runner(
    prove_args: &ProveArgs,
    tmp_dir: &tempfile::TempDir,
) -> Result<CairoRunner, anyhow::Error> {
    assert_eq!(
        prove_args.layout,
        LayoutName::automatic,
        "layout must be automatic"
    );

    let cairo_layout_params_file = tmp_dir.path().join("cairo_layout_params_file.json");

    // write to "cairo_layout_params_file.json"
    std::fs::write(cairo_layout_params_file.clone(), DYNAMIC_LAYOUT)?;

    let args = if let Some(program_input_file) = &prove_args.program_input_file {
        let file_content = std::fs::read_to_string(program_input_file)?;
        process_args(&file_content).unwrap()
    } else {
        prove_args.program_input.clone()
    };
    let cairo_run_config = Cairo1RunConfig {
        proof_mode: true,
        serialize_output: true,
        relocate_mem: true,
        layout: cairo_vm::types::layout_name::LayoutName::dynamic,
        trace_enabled: true,
        args: &args.0,
        finalize_builtins: true,
        append_return_values: true,
        dynamic_layout_params: Some(CairoLayoutParams::from_file(
            cairo_layout_params_file.as_path(),
        )?),
    };

    // Try to parse the file as a sierra program
    let file = std::fs::read(&prove_args.cairo_program)?;
    let sierra_program = match serde_json::from_slice(&file) {
        Ok(program) => program,
        Err(_) => {
            // If it fails, try to compile it as a cairo program
            let compiler_config = CompilerConfig {
                replace_ids: true,
                ..CompilerConfig::default()
            };
            let mut db = RootDatabase::builder()
                .skip_auto_withdraw_gas()
                .build()
                .unwrap();

            let config: Config = serde_json::from_str(CONFIG).expect("Failed to parse config file");
            let download_dir = Path::new(env!("HOME")).join(&config.download_dir);
            let corelib_dir = download_dir.join("corelib");
            if !corelib_dir.exists() {
                anyhow::bail!("Corelib directory does not exist: {:?}", corelib_dir);
            }
            init_dev_corelib(&mut db, corelib_dir.join("src"));
            let main_crate_ids = setup_project(&mut db, &prove_args.cairo_program).unwrap();
            let sierra_program_with_dbg =
                compile_prepared_db(&db, main_crate_ids, compiler_config).unwrap();

            sierra_program_with_dbg.program
        }
    };

    let (runner, _, serialized_output) =
        cairo_run_program_cairo1(&sierra_program, cairo_run_config)?;
    println!("Cairo1 program output: {:?}", serialized_output);

    Ok(runner)
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

fn write_to_files(
    runner: &CairoRunner,
    tmp_dir: &tempfile::TempDir,
    filename: &str,
) -> Result<CairoRunResult, anyhow::Error> {
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
