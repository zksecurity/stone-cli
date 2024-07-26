use rstest::{fixture, rstest};
use starknet_adapter::{
    args::{LayoutName, ProveArgs, VerifyArgs},
    cairo1::run_cairo1,
    prover::run_stone_prover,
    utils::{parse, set_env_vars},
    verifier::run_stone_verifier,
};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use tempfile::TempDir;

#[fixture]
fn setup() {
    const CONFIG: &str = include_str!("../configs/env.json");
    let config = parse(CONFIG);
    set_env_vars(&config);
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
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(program);
    let prove_args = ProveArgs {
        cairo_program: program_file.clone(),
        program_input: None,
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
    };
    match run_cairo1(&prove_args, &tmp_dir) {
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
#[case("starknet_with_keccak", "fibonacci.cairo")]
#[case("recursive_large_output", "fibonacci.cairo")]
#[case("recursive_with_poseidon", "fibonacci.cairo")]
#[case("all_solidity", "fibonacci.cairo")]
#[case("all_cairo", "fibonacci.cairo")]
#[case("dynamic", "fibonacci.cairo")]
#[case("recursive", "array_append.cairo")]
#[case("recursive", "array_get.cairo")]
#[case("recursive", "array_integer_tuple.cairo")]
#[case("recursive", "bitwise.cairo")]
#[case("recursive", "bytes31_ret.cairo")]
#[case("recursive", "dict_with_struct.cairo")]
#[case("recursive", "dictionaries.cairo")]
#[case("recursive", "enum_flow.cairo")]
#[case("recursive", "enum_match.cairo")]
#[case("recursive", "factorial.cairo")]
#[case("recursive", "felt_dict_squash.cairo")]
#[case("recursive", "felt_dict.cairo")]
#[case("recursive", "felt_span.cairo")]
#[case("recursive", "hello.cairo")]
#[case("recursive", "null_ret.cairo")]
#[case("recursive", "nullable_box_vec.cairo")]
#[case("recursive", "nullable_dict.cairo")]
#[case("recursive", "ops.cairo")]
#[case("recursive", "pedersen_example.cairo")]
#[case("recursive", "print.cairo")]
#[case("recursive", "recursion.cairo")]
#[case("recursive", "sample.cairo")]
#[case("recursive", "serialize_felt.cairo")]
#[case("recursive", "simple_struct.cairo")]
#[case("recursive", "simple.cairo")]
#[case("recursive", "struct_span_return.cairo")]
#[case("recursive", "tensor_new.cairo")]
#[case("starknet", "ecdsa_recover.cairo")]
#[case("starknet", "poseidon_pedersen.cairo")]
#[case("starknet", "poseidon.cairo")]
fn test_run_cairo1_success(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("starknet-adapter-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(program);
    let prove_args = ProveArgs {
        cairo_program: program_file.clone(),
        program_input: None,
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
    };
    match run_cairo1(&prove_args, &tmp_dir) {
        Ok(result) => println!("Successfully ran cairo1: {:?}", result),
        Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
    }

    check_tmp_files(&tmp_dir, &program_file);
}

#[rstest]
#[case("recursive", "array_input_sum.cairo", "array_input_sum_input.txt")]
#[case("recursive", "array_length.cairo", "array_length_input.txt")]
#[case("recursive", "branching.cairo", "branching_input.txt")]
#[case("recursive", "dict_with_input.cairo", "dict_with_input_input.txt")]
#[case("recursive", "tensor.cairo", "tensor_input.txt")]
fn test_run_cairo1_with_input_file(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
    #[case(input)] input: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("starknet-adapter-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("with_input")
        .join(program);
    let input_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("with_input")
        .join(input);

    let prove_args = ProveArgs {
        cairo_program: program_file.clone(),
        program_input: None,
        program_input_file: Some(input_file),
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
    };

    match run_cairo1(&prove_args, &tmp_dir) {
        Ok(_) => {
            println!("Successfully ran cairo1 with input file");
        }
        Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
    }
}

#[rstest]
#[case("recursive", "array_input_sum.cairo", "[2 4 1 2 3 4 0 2 9 8]")]
#[case("recursive", "array_length.cairo", "[4 1 2 3 4 0]")]
#[case("recursive", "branching.cairo", "[17]")]
#[case("recursive", "dict_with_input.cairo", "[17 18]")]
#[case("recursive", "tensor.cairo", "[2 2 2 4 1 2 3 4]")]
fn test_run_cairo1_with_inputs(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
    #[case(input)] input: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("starknet-adapter-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("with_input")
        .join(program);
    let prove_args = ProveArgs {
        cairo_program: program_file.clone(),
        program_input: Some(input.to_string()),
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
    };

    match run_cairo1(&prove_args, &tmp_dir) {
        Ok(_) => {
            println!("Successfully ran cairo1 with input file");
        }
        Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
    }
}

#[rstest]
#[case("recursive", "fibonacci.cairo", "")]
#[case("recursive", "array_input_sum.cairo", "[2 4 1 2 3 4 0 2 9 8]")]
fn test_run_e2e(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
    #[case(input)] input: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("starknet-adapter-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = if !input.is_empty() {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("with_input")
            .join(program)
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join(program)
    };
    let program_input = if !input.is_empty() {
        Some(input.to_string())
    } else {
        None
    };
    let prove_args = ProveArgs {
        cairo_program: program_file.clone(),
        program_input,
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
    };
    let verify_args = VerifyArgs {
        proof: tmp_dir.path().join("proof.json"),
    };

    match run_cairo1(&prove_args, &tmp_dir) {
        Ok(_) => {
            let filename = program_file.file_stem().unwrap().to_str().unwrap();
            let air_public_input = tmp_dir
                .path()
                .join(format!("{}_air_public_input.json", filename));
            let air_private_input = tmp_dir
                .path()
                .join(format!("{}_air_private_input.json", filename));

            match run_stone_prover(&prove_args, &air_public_input, &air_private_input, &tmp_dir) {
                Ok(_) => match run_stone_verifier(&verify_args) {
                    Ok(_) => {
                        println!("Successfully ran stone verifier");
                    }
                    Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
                },
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
