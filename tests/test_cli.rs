use rstest::{fixture, rstest};
use starknet_adapter::{build::setup as build_setup, cairo1::run_cairo1};
use std::{env::current_exe, path::Path, path::PathBuf};
use tempfile::TempDir;

#[fixture]
fn setup() {
    build_setup();
}

#[rstest]
#[case("plain", "fibonacci.cairo")]
fn test_run_cairo1_fail(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("starknet-adapter-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    println!(
        "cargo manifest dir: {:?}",
        Path::new(env!("CARGO_MANIFEST_DIR"))
    );
    let current_exe = current_exe().expect("Failed to get current executable");
    println!("Current executable: {:?}", current_exe);
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(program);
    let args = vec![
        current_exe.to_str().unwrap(),
        program_file.to_str().unwrap(),
        "--layout",
        layout,
        "--proof_mode",
    ]
    .into_iter()
    .map(|s| s.to_string());
    match run_cairo1(args, &tmp_dir) {
        Ok(result) => panic!(
            "Expected an error but got a successful result: {:?}",
            result
        ),
        Err(e) => assert_error_msg_eq(
            &e,
            "cairo1-run failed with error: Error: VirtualMachine(Memory(AddressNotRelocatable))\n",
        ),
    }
}

#[rstest]
#[case("small", "fibonacci.cairo")]
#[case("dex", "fibonacci.cairo")]
#[case("recursive", "fibonacci.cairo")]
#[case("starknet", "fibonacci.cairo")]
#[case("starknet-with-keccak", "fibonacci.cairo")]
#[case("recursive-large-output", "fibonacci.cairo")]
#[case("recursive-with-poseidon", "fibonacci.cairo")]
#[case("all-solidity", "fibonacci.cairo")]
#[case("all-cairo", "fibonacci.cairo")]
#[case("dynamic", "fibonacci.cairo")]
fn test_run_cairo1_success(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("starknet-adapter-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    println!(
        "cargo manifest dir: {:?}",
        Path::new(env!("CARGO_MANIFEST_DIR"))
    );
    let current_exe = current_exe().expect("Failed to get current executable");
    println!("Current executable: {:?}", current_exe);
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(program);
    let args = vec![
        current_exe.to_str().unwrap(),
        program_file.to_str().unwrap(),
        "--layout",
        layout,
        "--proof_mode",
    ]
    .into_iter()
    .map(|s| s.to_string());
    match run_cairo1(args, &tmp_dir) {
        Ok(result) => println!("Successfully ran cairo1: {:?}", result),
        Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
    }

    check_tmp_files(&tmp_dir, &program_file);
}

fn assert_error_msg_eq(e: &anyhow::Error, expected: &str) {
    assert_eq!(e.to_string(), expected);
}

fn check_tmp_files(tmp_dir: &TempDir, program_file: &PathBuf) {
    let filename = program_file.file_stem().unwrap().to_str().unwrap();
    let trace_file = tmp_dir.path().join(format!("{}_trace.json", filename));
    assert!(trace_file.exists(), "Trace file does not exist");
    let memory_file = tmp_dir.path().join(format!("{}_memory.json", filename));
    assert!(memory_file.exists(), "Memory file does not exist");
    let air_public_input_file = tmp_dir
        .path()
        .join(format!("{}_air_public_input.json", filename));
    assert!(
        air_public_input_file.exists(),
        "AIR public input file does not exist"
    );
    let air_private_input_file = tmp_dir
        .path()
        .join(format!("{}_air_private_input.json", filename));
    assert!(
        air_private_input_file.exists(),
        "AIR private input file does not exist"
    );
}
