use crate::args::ProveBootloaderArgs;
use crate::utils::{get_formatted_air_public_input, FileWriter};
use cairo_bootloader::hints::{
    BootloaderConfig, BootloaderHintProcessor, BootloaderInput, PackedOutput,
    SimpleBootloaderInput, TaskSpec,
};
use cairo_bootloader::insert_bootloader_input;
use cairo_bootloader::tasks::{make_bootloader_tasks, BootloaderTaskError};
use cairo_vm::air_public_input::PublicInputError;
use cairo_vm::cairo_run::{
    cairo_run_program_with_initial_scope, write_encoded_memory, write_encoded_trace,
    CairoRunConfig, EncodeTraceError,
};
use cairo_vm::types::errors::program_errors::ProgramError;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::types::program::Program;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::trace_errors::TraceError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use cairo_vm::Felt252;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

const BOOTLOADER_V0_13_1: &[u8] = include_bytes!("../resources/bootloader-0.13.1.json");
// Hash of the simple bootloader program
// Value is taken from data stored on-chain:
//     https://etherscan.io/address/0xd51A3D50d4D2f99a345a66971E650EEA064DD8dF#readContract#F6
const SIMPLE_BOOTLOADER_PROGRAM_HASH: &str =
    "382450030162484995497251732956824096484321811411123989415157331925872358847";
// Hashes of Cairo verifier programs that can be used to recursively verify a simple bootloader program proof
// Pre-image data for the hash stored on-chain:
//     https://etherscan.io/address/0xd51A3D50d4D2f99a345a66971E650EEA064DD8dF#readContract#F6
const CAIRO_VERIFIER_PROGRAM_HASHES: &[&str] = &[
    "0x97e831fcc22602fa025e89c9c6b7e7272686398de428136cf52f3f006a845e",
    "0x7b2acdc57670aff4eac1f72b41ef759f003c122ed6cece634b76581966eade2",
    "0x24a3890c0d0ee8f7dfed5d1f89e3991bbc1b20d506c0700b24977f16f4487",
    "0x49904b6ecb9e083a42a1f50eb336ecc7e7a7c3ce06aabea405847cf0e2c1b2",
    "0x64b111ddda7af6661f2d1e6255ad7576ce8281ec701b166f07debca3bd7a0eb",
    "0x29a7e7366aa18c837867443aed5975f55107a8fdb6f33c418b81463a4156abf",
    "0x1fbfa8a63b6197519c5fbbf3b9090b6fadea637c8afba051c7419fd1d3d7fb3",
    "0x7d9c440b45a189c29e5d544d5b3ed167d089e3dd21e154abede91f90afb35ca",
    "0x41e6fdf682dca5b1e1194a93da5312fe66c06f08550a62c9e27645ca3874483",
    "0x3b58154a414a8e66fb65b1f6f612cd4ca21d68815fb0c6252930d4ddb04c72c",
    "0x2c47af88d90c4acd90fa663713e02a1f0a8b1239882d2f6b58dc964529540c9",
    "0x571ed7fb8805802da530fcac931794462cb7909479ec0ffd24766913a88636c",
    "0x6ad5606d7e4e7bc01b38a36d3fdca7afcf24d5db25ed4d050a4e77c31c5527b",
    "0x323c8251dbd935f45105bc241765a5082d9a985cbc3bffece3382708fb88dc5",
];

pub struct CairoBootloaderRunResult {
    pub air_public_input: PathBuf,
    pub air_private_input: PathBuf,
    pub memory_file: PathBuf,
    pub trace_file: PathBuf,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
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
    BootloaderTask(#[from] BootloaderTaskError),
    #[error(transparent)]
    VirtualMachine(#[from] VirtualMachineError),
    #[error("Topology file should be specified as it will be required for serializing bootloader proofs")]
    TopologyFileNotSpecified,
}

pub fn run_bootloader(
    prove_bootloader_args: &ProveBootloaderArgs,
    tmp_dir: &tempfile::TempDir,
) -> Result<CairoBootloaderRunResult, Error> {
    let bootloader_program = Program::from_bytes(BOOTLOADER_V0_13_1, Some("main"))?;
    let program_paths = prove_bootloader_args
        .cairo_programs
        .as_ref()
        .map(|paths| paths.iter().map(|p| p.as_path()).collect::<Vec<_>>());
    let program_inputs = Some(vec![
        HashMap::new();
        program_paths.as_ref().map_or(0, |p| p.len())
    ]);

    let pie_paths = prove_bootloader_args
        .cairo_pies
        .as_ref()
        .map(|paths| paths.iter().map(|p| p.as_path()).collect::<Vec<_>>());

    let tasks = make_bootloader_tasks(
        program_paths.as_deref(),
        program_inputs.as_deref(),
        pie_paths.as_deref(),
    )?;

    let mut runner = cairo_run_bootloader_in_proof_mode(
        &bootloader_program,
        tasks,
        prove_bootloader_args.layout.to_cairo_vm_layout(),
        prove_bootloader_args.fact_topologies_output.clone(),
        prove_bootloader_args.ignore_fact_topologies,
    )?;

    let relocated_trace = runner
        .relocated_trace
        .as_ref()
        .ok_or(Error::Trace(TraceError::TraceNotRelocated))?;
    let trace_path = tmp_dir.path().join("bootloader_trace.json");
    let trace_file = std::fs::File::create(trace_path.clone())?;
    let mut trace_writer =
        FileWriter::new(io::BufWriter::with_capacity(3 * 1024 * 1024, trace_file));
    write_encoded_trace(relocated_trace, &mut trace_writer)?;
    trace_writer.flush()?;

    let memory_path = tmp_dir.path().join("bootloader_memory.json");
    let memory_file = std::fs::File::create(memory_path.clone())?;
    let mut memory_writer =
        FileWriter::new(io::BufWriter::with_capacity(5 * 1024 * 1024, memory_file));
    write_encoded_memory(&runner.relocated_memory, &mut memory_writer)?;
    memory_writer.flush()?;

    let air_public_input_path = tmp_dir.path().join("bootloader_air_public_input.json");
    let air_public_input_str = get_formatted_air_public_input(&runner.get_air_public_input()?)?;
    std::fs::write(air_public_input_path.clone(), air_public_input_str)?;

    let air_private_input_path = tmp_dir.path().join("bootloader_air_private_input.json");
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

    let mut output_buffer = "Bootloader program output:\n".to_string();
    runner.vm.write_output(&mut output_buffer)?;
    print!("{output_buffer}");

    Ok(CairoBootloaderRunResult {
        air_public_input: air_public_input_path,
        air_private_input: air_private_input_path,
        memory_file: memory_path,
        trace_file: trace_path,
    })
}

fn cairo_run_bootloader_in_proof_mode(
    bootloader_program: &Program,
    tasks: Vec<TaskSpec>,
    layout: LayoutName,
    fact_topologies_path: PathBuf,
    ignore_fact_topologies: bool,
) -> Result<CairoRunner, CairoRunError> {
    let mut hint_processor = BootloaderHintProcessor::new();

    let cairo_run_config = CairoRunConfig {
        entrypoint: "main",
        trace_enabled: true,
        relocate_mem: true,
        layout,
        proof_mode: true,
        secure_run: None,
        disable_trace_padding: false,
        allow_missing_builtins: None,
        dynamic_layout_params: None,
    };

    let program_hash = Felt252::from_dec_str(SIMPLE_BOOTLOADER_PROGRAM_HASH).unwrap();
    let verifier_hashes = CAIRO_VERIFIER_PROGRAM_HASHES
        .iter()
        .map(|h| Felt252::from_hex(h).unwrap())
        .collect();

    let n_tasks = tasks.len();

    let bootloader_input = BootloaderInput {
        simple_bootloader_input: SimpleBootloaderInput {
            fact_topologies_path: Some(fact_topologies_path),
            single_page: false,
            tasks,
        },
        bootloader_config: BootloaderConfig {
            simple_bootloader_program_hash: program_hash,
            supported_cairo_verifier_program_hashes: verifier_hashes,
        },
        packed_outputs: vec![PackedOutput::Plain(vec![]); n_tasks],
        ignore_fact_topologies: false,
    };

    let mut exec_scopes = ExecutionScopes::new();
    insert_bootloader_input(&mut exec_scopes, bootloader_input);

    cairo_run_program_with_initial_scope(
        bootloader_program,
        &cairo_run_config,
        &mut hint_processor,
        exec_scopes,
    )
}
