use assert_cmd::prelude::*;
use predicates::prelude::*;
use rstest::{fixture, rstest};
use std::path::Path;
use std::process::Command;
use stone_cli::utils::{parse, set_env_vars};

#[fixture]
fn setup() {
    const CONFIG: &str = include_str!("../configs/env.json");
    let config = parse(CONFIG);
    set_env_vars(&config);
}

#[rstest]
#[case("--field", "PrimeField0")]
#[case("--channel_hash", "poseidon3")]
#[case("--commitment_hash", "keccak256-masked160-lsb")]
#[case("--n_verifier_friendly_commitment_layers", "0")]
#[case("--pow_hash", "keccak256")]
#[case("--page_hash", "pedersen")]
#[case("--fri_step_list", "0 4 4 3")]
#[case("--last_layer_degree_bound", "64")]
#[case("--n_queries", "16")]
#[case("--proof_of_work_bits", "32")]
#[case("--log_n_cosets", "4")]
#[case("--use_extension_field", "false")]
#[case("--verifier_friendly_channel_updates", "true")]
#[case("--verifier_friendly_commitment_hash", "poseidon3")]
#[case("--constraint_polynomial_task_size", "256")]
#[case("--n_out_of_memory_merkle_layers", "0")]
#[case("--table_prover_n_tasks_per_segment", "32")]
#[case("--store_full_lde", "false")]
#[case("--use_fft_for_eval", "true")]
fn test_fail_on_arg_conflicts_with_config_file(
    #[from(setup)] _path: (),
    #[case(arg)] arg: &str,
    #[case(value)] value: &str,
) {
    let program_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("fibonacci.cairo");

    let (config_file, config_arg) = match arg {
        "--constraint_polynomial_task_size"
        | "--n_out_of_memory_merkle_layers"
        | "--table_prover_n_tasks_per_segment"
        | "--store_full_lde"
        | "--use_fft_for_eval" => (
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("configs")
                .join("cpu_air_prover_config.json"),
            "--prover_config_file",
        ),
        _ => (
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("configs")
                .join("cpu_air_params.json"),
            "--parameter_file",
        ),
    };

    let mut cmd = Command::cargo_bin("stone-cli").unwrap();
    cmd.arg("prove")
        .arg("--cairo_program")
        .arg(&program_file)
        .arg(config_arg)
        .arg(&config_file)
        .arg(arg)
        .arg(value);

    let expected_error_message = if arg == "--fri_step_list" {
        format!(
            "the argument '{} <{}>' cannot be used with '{} <{}>...'",
            config_arg,
            config_arg.trim_start_matches("--").to_uppercase(),
            arg,
            arg.trim_start_matches("--").to_uppercase()
        )
    } else {
        format!(
            "the argument '{} <{}>' cannot be used with '{} <{}>'",
            config_arg,
            config_arg.trim_start_matches("--").to_uppercase(),
            arg,
            arg.trim_start_matches("--").to_uppercase()
        )
    };
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(expected_error_message));
}
