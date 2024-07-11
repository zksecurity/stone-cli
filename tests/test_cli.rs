use rstest::{fixture, rstest};
use starknet_adapter::{
    cairo1::run_cairo1, prover::run_stone_prover, starknet::run_starknet_verifier,
    utils::set_env_vars,
};
use std::{env::current_exe, path::Path, path::PathBuf};
use tempfile::TempDir;

#[fixture]
fn setup() {
    const CONFIG: &[u8] = include_bytes!("../config.json");
    set_env_vars(CONFIG);
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

#[rstest]
#[case("recursive", "array_append.cairo")]
#[case("recursive", "array_integer_tuple.cairo")]
#[case("recursive", "bytes31_ret.cairo")]
#[case("recursive", "enum_flow.cairo")]
#[case("recursive", "enum_match.cairo")]
#[case("recursive", "factorial.cairo")]
#[case("recursive", "felt_span.cairo")]
#[case("recursive", "fibonacci.cairo")]
#[case("recursive", "hello.cairo")]
#[case("recursive", "null_ret.cairo")]
#[case("recursive", "pedersen_example.cairo")]
#[case("recursive", "print.cairo")]
#[case("recursive", "recursion.cairo")]
#[case("recursive", "sample.cairo")]
#[case("recursive", "simple_struct.cairo")]
#[case("recursive", "simple.cairo")]
#[case("recursive", "struct_span_return.cairo")]
fn test_run_e2e(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("starknet-adapter-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let current_exe = current_exe().expect("Failed to get current executable");
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

    match run_cairo1(args.clone(), &tmp_dir) {
        Ok(result) => {
            let filename = program_file.file_stem().unwrap().to_str().unwrap();
            let air_public_input = tmp_dir
                .path()
                .join(format!("{}_air_public_input.json", filename));
            let air_private_input = tmp_dir
                .path()
                .join(format!("{}_air_private_input.json", filename));

            match run_stone_prover(&air_public_input, &air_private_input, &tmp_dir) {
                Ok(result) => {
                    let proof = result.proof;

                    match run_starknet_verifier(args, &proof) {
                        Ok(result) => {
                            println!("Successfully ran starknet verifier: {:?}", result);
                        }
                        Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
                    }
                }
                Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
            }
        }
        Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
    }
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
