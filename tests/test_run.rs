use rstest::{fixture, rstest};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use stone_cli::{
    args::{
        CairoVersion, LayoutName, Network, ProveArgs, ProveBootloaderArgs, SerializeArgs,
        VerifyArgs,
    },
    bootloader::run_bootloader,
    cairo::{run_cairo0, run_cairo1},
    config::{ProverConfig, ProverParametersConfig},
    prover::{run_stone_prover, run_stone_prover_bootloader},
    serialize::serialize_proof,
    utils::{parse, set_env_vars},
    verifier::run_stone_verifier,
};
use tempfile::TempDir;

#[fixture]
fn setup() {
    const CONFIG: &str = include_str!("../configs/env.json");
    let config = parse(CONFIG);
    set_env_vars(&config);
}

#[rstest]
#[case("recursive", "abs_value_array.json")]
#[case("recursive", "array_sum.json")]
#[case("recursive", "assert_250_bit_element_array.json")]
#[case("recursive", "assert_le_felt_hint.json")]
#[case("recursive", "assert_le_felt_old.json")]
#[case("recursive", "assert_lt_felt.json")]
#[case("recursive", "assert_nn.json")]
#[case("recursive", "assert_not_zero.json")]
#[case("recursive", "big_struct.json")]
#[case("recursive", "bigint.json")]
#[case("recursive", "bitand_hint.json")]
#[case("recursive", "bitwise_builtin_test.json")]
#[case("recursive", "bitwise_output.json")]
#[case("recursive", "bitwise_recursion.json")]
#[case("recursive", "blake2s_felts.json")]
#[case("recursive", "blake2s_hello_world_hash.json")]
#[case("recursive", "blake2s_integration_tests.json")]
#[case("recursive", "cairo_finalize_keccak.json")]
#[case("recursive", "cairo_finalize_keccak_block_size_1000.json")]
#[case("recursive", "call_function_assign_param_by_name.json")]
#[case("recursive", "chained_ec_op.json")]
#[case("recursive", "common_signature.json")]
#[case("recursive", "compare_arrays.json")]
#[case("recursive", "compare_different_arrays.json")]
#[case("recursive", "compare_greater_array.json")]
#[case("recursive", "compare_lesser_array.json")]
#[case("recursive", "compute_doubling_slope_v2.json")]
#[case("recursive", "compute_slope_v2.json")]
#[case("recursive", "dict.json")]
#[case("recursive", "dict_integration_tests.json")]
#[case("recursive", "dict_squash.json")]
#[case("recursive", "dict_store_cast_ptr.json")]
#[case("recursive", "dict_update.json")]
#[case("recursive", "div_mod_n.json")]
#[case("recursive", "ec_double_assign_new_x_v3.json")]
#[case("recursive", "ec_double_slope.json")]
#[case("recursive", "ec_double_v4.json")]
#[case("recursive", "ec_negate.json")]
#[case("recursive", "ec_op.json")]
#[case("recursive", "ec_recover.json")]
#[case("recursive", "ed25519_ec.json")]
#[case("recursive", "ed25519_field.json")]
#[case("recursive", "efficient_secp256r1_ec.json")]
#[case("recursive", "example_blake2s.json")]
#[case("recursive", "example_program.json")]
#[case("recursive", "factorial.json")]
#[case("recursive", "fast_ec_add_v2.json")]
#[case("recursive", "fast_ec_add_v3.json")]
#[case("recursive", "fibonacci.json")]
#[case("recursive", "field_arithmetic.json")]
#[case("recursive", "finalize_blake2s.json")]
#[case("recursive", "finalize_blake2s_v2_hint.json")]
#[case("recursive", "find_element.json")]
#[case("recursive", "fq.json")]
#[case("recursive", "fq_test.json")]
#[case("recursive", "function_return.json")]
#[case("recursive", "function_return_if_print.json")]
#[case("recursive", "function_return_to_variable.json")]
#[case("recursive", "garaga.json")]
#[case("recursive", "highest_bitlen.json")]
#[case("recursive", "if_and_prime.json")]
#[case("recursive", "if_in_function.json")]
#[case("recursive", "if_list.json")]
#[case("recursive", "if_reloc_equal.json")]
#[case("recursive", "integration.json")]
#[case("recursive", "integration_with_alloc_locals.json")]
#[case("recursive", "inv_mod_p_uint512.json")]
#[case("recursive", "is_quad_residue_test.json")]
#[case("recursive", "is_zero.json")]
#[case("recursive", "is_zero_pack.json")]
#[case("recursive", "jmp.json")]
#[case("recursive", "jmp_if_condition.json")]
#[case("recursive", "keccak.json")]
#[case("recursive", "keccak_add_uint256.json")]
#[case("recursive", "keccak_alternative_hint.json")]
#[case("recursive", "keccak_builtin.json")]
#[case("recursive", "keccak_copy_inputs.json")]
#[case("recursive", "keccak_integration_tests.json")]
#[case("recursive", "keccak_uint256.json")]
#[case("recursive", "math_cmp.json")]
#[case("recursive", "math_cmp_and_pow_integration_tests.json")]
#[case("recursive", "math_integration_tests.json")]
#[case("recursive", "memcpy_test.json")]
#[case("recursive", "memory_holes.json")]
#[case("recursive", "memory_integration_tests.json")]
#[case("recursive", "memset.json")]
#[case("recursive", "mul_s_inv.json")]
#[case("recursive", "multiplicative_inverse.json")]
#[case("recursive", "n_bit.json")]
#[case("recursive", "nondet_bigint3_v2.json")]
#[case("recursive", "normalize_address.json")]
#[case("recursive", "not_main.json")]
#[case("recursive", "operations_with_data_structures.json")]
#[case("recursive", "packed_sha256.json")]
#[case("recursive", "packed_sha256_test.json")]
#[case("recursive", "pedersen_extra_builtins.json")]
#[case("recursive", "pedersen_test.json")]
#[case("recursive", "pointers.json")]
#[case("recursive", "poseidon_builtin.json")]
#[case("recursive", "poseidon_hash.json")]
#[case("recursive", "poseidon_multirun.json")]
#[case("recursive", "pow.json")]
#[case("recursive", "print.json")]
#[case("recursive", "recover_y.json")]
#[case("recursive", "reduce.json")]
#[case("recursive", "relocate_segments.json")]
#[case("recursive", "relocate_segments_with_offset.json")]
#[case("recursive", "relocate_temporary_segment_append.json")]
#[case("recursive", "relocate_temporary_segment_into_new.json")]
#[case("recursive", "return.json")]
#[case("recursive", "reversed_register_instructions.json")]
#[case("recursive", "search_sorted_lower.json")]
#[case("recursive", "secp.json")]
#[case("recursive", "secp256r1_div_mod_n.json")]
#[case("recursive", "secp256r1_fast_ec_add.json")]
#[case("recursive", "secp256r1_slope.json")]
#[case("recursive", "secp_ec.json")]
#[case("recursive", "secp_integration_tests.json")]
#[case("recursive", "set_add.json")]
#[case("recursive", "set_integration_tests.json")]
#[case("recursive", "sha256.json")]
#[case("recursive", "sha256_test.json")]
#[case("recursive", "signature.json")]
#[case("recursive", "signed_div_rem.json")]
#[case("recursive", "simple_print.json")]
#[case("recursive", "split_felt.json")]
#[case("recursive", "split_int.json")]
#[case("recursive", "split_int_big.json")]
#[case("recursive", "split_xx_hint.json")]
#[case("recursive", "sqrt.json")]
#[case("recursive", "squash_dict.json")]
#[case("recursive", "struct.json")]
#[case("recursive", "test_addition_if.json")]
#[case("recursive", "test_reverse_if.json")]
#[case("recursive", "test_subtraction_if.json")]
#[case("recursive", "uint256.json")]
#[case("recursive", "uint256_improvements.json")]
#[case("recursive", "uint256_integration_tests.json")]
#[case("recursive", "uint384.json")]
#[case("recursive", "uint384_extension.json")]
#[case("recursive", "uint384_extension_test.json")]
#[case("recursive", "uint384_test.json")]
#[case("recursive", "unsafe_keccak.json")]
#[case("recursive", "unsafe_keccak_finalize.json")]
#[case("recursive", "unsigned_div_rem.json")]
#[case("recursive", "use_imported_module.json")]
#[case("recursive", "usort.json")]
#[case("recursive", "value_beyond_segment.json")]
fn test_run_cairo0_success(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("stone-cli-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("cairo0")
        .join(program);
    let prove_args = ProveArgs {
        cairo_version: CairoVersion::cairo0,
        cairo_program: program_file.clone(),
        program_input: None,
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
        parameter_config: ProverParametersConfig::default(),
        prover_config: ProverConfig::default(),
    };

    match run_cairo0(&prove_args, &tmp_dir) {
        Ok(_) => {
            println!("Successfully ran cairo0");
        }
        Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
    }

    check_tmp_files(&tmp_dir, &program_file);
}

#[rstest]
#[case("plain", "fibonacci.cairo")]
fn test_run_cairo1_fail(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("stone-cli-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(program);
    let prove_args = ProveArgs {
        cairo_version: CairoVersion::cairo1,
        cairo_program: program_file.clone(),
        program_input: None,
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
        parameter_config: ProverParametersConfig::default(),
        prover_config: ProverConfig::default(),
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
        .prefix("stone-cli-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(program);
    let prove_args = ProveArgs {
        cairo_version: CairoVersion::cairo1,
        cairo_program: program_file.clone(),
        program_input: None,
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
        parameter_config: ProverParametersConfig::default(),
        prover_config: ProverConfig::default(),
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
        .prefix("stone-cli-test-")
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
        cairo_version: CairoVersion::cairo1,
        cairo_program: program_file.clone(),
        program_input: None,
        program_input_file: Some(input_file),
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
        parameter_config: ProverParametersConfig::default(),
        prover_config: ProverConfig::default(),
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
        .prefix("stone-cli-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("with_input")
        .join(program);
    let prove_args = ProveArgs {
        cairo_version: CairoVersion::cairo1,
        cairo_program: program_file.clone(),
        program_input: Some(input.to_string()),
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
        parameter_config: ProverParametersConfig::default(),
        prover_config: ProverConfig::default(),
    };

    match run_cairo1(&prove_args, &tmp_dir) {
        Ok(_) => {
            println!("Successfully ran cairo1 with input file");
        }
        Err(e) => panic!("Expected a successful result but got an error: {:?}", e),
    }
}

#[rstest]
#[case("small", "fibonacci.json", CairoVersion::cairo0)]
#[case("small", "fibonacci.cairo", CairoVersion::cairo1)]
fn test_run_cairo_e2e(
    #[from(setup)] _path: (),
    #[case(layout)] layout: &str,
    #[case(program)] program: &str,
    #[case(cairo_version)] cairo_version: CairoVersion,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("stone-cli-test-")
        .tempdir()
        .expect("Failed to create temp dir");
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(if cairo_version == CairoVersion::cairo0 {
            "cairo0"
        } else {
            ""
        })
        .join(program);
    let prove_args = ProveArgs {
        cairo_version: cairo_version.clone(),
        cairo_program: program_file.clone(),
        program_input: None,
        program_input_file: None,
        layout: LayoutName::from_str(layout).unwrap(),
        prover_config_file: None,
        parameter_file: None,
        output: tmp_dir.path().join("proof.json"),
        parameter_config: ProverParametersConfig::default(),
        prover_config: ProverConfig::default(),
    };
    let verify_args = VerifyArgs {
        proof: tmp_dir.path().join("proof.json"),
        annotation_file: None,
        extra_output_file: None,
    };

    let run_result = match cairo_version {
        CairoVersion::cairo0 => run_cairo0(&prove_args, &tmp_dir).map_err(|e| anyhow::anyhow!(e)),
        CairoVersion::cairo1 => run_cairo1(&prove_args, &tmp_dir),
    };

    match run_result {
        Ok(_) => {
            let filename = program_file.file_stem().unwrap().to_str().unwrap();
            let air_public_input = tmp_dir
                .path()
                .join(format!("{}_air_public_input.json", filename));
            let air_private_input = tmp_dir
                .path()
                .join(format!("{}_air_private_input.json", filename));

            match run_stone_prover(&prove_args, &air_public_input, &air_private_input, &tmp_dir) {
                Ok(_) => match run_stone_verifier(verify_args) {
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

    check_tmp_files(&tmp_dir, &program_file);
}

#[rstest]
#[case("bitwise_output.json", "", [1,0], [1])]
#[case("", "fibonacci_with_output.zip", [1,0], [2])]
fn test_run_bootloader(
    #[from(setup)] _path: (),
    #[case(cairo_program)] cairo_program: &str,
    #[case(cairo_pie)] cairo_pie: &str,
    #[case(tree_structure)] tree_structure: [u32; 2],
    #[case(page_sizes)] page_sizes: [u32; 1],
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("stone-cli-test-")
        .tempdir()
        .expect("Failed to create temp dir");

    let program_file = if cairo_program == "" {
        None
    } else {
        Some(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples")
                .join("cairo0")
                .join(cairo_program),
        )
    };
    let cairo_pie_file = if cairo_pie == "" {
        None
    } else {
        Some(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples")
                .join("cairo_pie")
                .join(cairo_pie),
        )
    };
    let bootloader_params_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("configs")
        .join("bootloader_cpu_air_params.json");
    let prove_bootloader_args = ProveBootloaderArgs {
        cairo_program: program_file.clone(),
        cairo_pie: cairo_pie_file.clone(),
        layout: LayoutName::starknet,
        prover_config_file: None,
        parameter_file: Some(bootloader_params_file.clone()),
        output: tmp_dir.path().join("bootloader_proof.json"),
        parameter_config: ProverParametersConfig::default(),
        prover_config: ProverConfig::default(),
        fact_topologies_output: tmp_dir.path().join("fact_topologies.json"),
    };

    match run_bootloader(&prove_bootloader_args, &tmp_dir) {
        Ok(_) => {
            let fact_topologies_content =
                std::fs::read_to_string(&prove_bootloader_args.fact_topologies_output)
                    .expect("Failed to read fact_topologies file");
            let fact_topologies: serde_json::Value = serde_json::from_str(&fact_topologies_content)
                .expect("Failed to parse fact_topologies JSON");

            // Assert the content of fact_topologies
            assert_eq!(
                fact_topologies["fact_topologies"][0]["tree_structure"],
                json!(tree_structure)
            );
            assert_eq!(
                fact_topologies["fact_topologies"][0]["page_sizes"],
                json!(page_sizes)
            );
        }
        Err(e) => panic!(
            "Expected a successful result but got an error while running bootloader: {:?}",
            e
        ),
    }
}

#[rstest]
fn test_run_serialize(#[from(setup)] _path: ()) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("stone-cli-test-")
        .tempdir()
        .expect("Failed to create temp dir");

    let proof_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resources")
        .join("bootloader_proof.json");
    let annotation_file = tmp_dir.path().join("bootloader_annotation.json");
    let extra_output_file = tmp_dir.path().join("bootloader_extra_output.json");

    let serialized_proof_file = tmp_dir.path().join("bootloader_proof_serialized.json");
    let expected_serialized_proof_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resources")
        .join("bootloader_proof_serialized.json");

    let verify_args = VerifyArgs {
        proof: proof_file.clone(),
        annotation_file: Some(annotation_file.clone()),
        extra_output_file: Some(extra_output_file.clone()),
    };

    let serialize_args = SerializeArgs {
        proof: proof_file,
        network: Network::ethereum,
        annotation_file: Some(annotation_file),
        extra_output_file: Some(extra_output_file),
        output: serialized_proof_file.clone(),
    };

    match run_stone_verifier(verify_args) {
        Ok(_) => match serialize_proof(serialize_args) {
            Ok(_) => {
                let expected_serialized_proof_content =
                    std::fs::read_to_string(expected_serialized_proof_file)
                        .expect("Failed to read expected serialized proof file");
                let serialized_proof_content = std::fs::read_to_string(serialized_proof_file)
                    .expect("Failed to read serialized proof file");
                assert_eq!(serialized_proof_content, expected_serialized_proof_content);
            }
            Err(e) => panic!(
                "Expected a successful result but got an error while serializing proof: {:?}",
                e
            ),
        },
        Err(e) => panic!(
            "Expected a successful result but got an error while running verifier: {:?}",
            e
        ),
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
