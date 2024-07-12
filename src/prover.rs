mod settings;

use crate::utils::write_json_to_file;
use settings::{get_prover_config, get_prover_parameters};
use std::path::PathBuf;
use std::process::Command;
use stone_prover_sdk::error::ProverError;
use stone_prover_sdk::json::read_json_from_file;

/// Represents the result of running the Stone prover
pub struct StoneProverResult {
    /// Path to the generated proof
    pub proof: PathBuf,
}

/// Runs the Stone prover with the given inputs
///
/// # Arguments
///
/// * `air_public_input` - Path to the public input file
/// * `air_private_input` - Path to the private input file
/// * `tmp_dir` - Temporary directory for intermediate files
///
/// # Returns
///
/// A `Result` containing `StoneProverResult` on success, or an `Error` on failure
pub fn run_stone_prover(
    air_public_input: &PathBuf,
    air_private_input: &PathBuf,
    tmp_dir: &tempfile::TempDir,
) -> Result<StoneProverResult, ProverError> {
    let air_public_input_json: serde_json::Value = read_json_from_file(air_public_input)?;
    // TODO: handle error properly
    let n_steps = air_public_input_json["n_steps"].as_u64().unwrap() as u32;
    let prover_parameters = get_prover_parameters(n_steps);
    let prover_parameters_path = tmp_dir.path().join("prover_parameters.json");
    write_json_to_file(&prover_parameters, &prover_parameters_path)?;

    let prover_config = get_prover_config();
    let prover_config_path = tmp_dir.path().join("prover_config.json");
    write_json_to_file(&prover_config, &prover_config_path)?;

    let proof_path = tmp_dir.path().join("proof.json");

    run_prover_from_command_line_with_annotations(
        air_public_input,
        air_private_input,
        &prover_config_path,
        &prover_parameters_path,
        &proof_path,
        true,
    )?;

    Ok(StoneProverResult { proof: proof_path })
}

pub fn run_prover_from_command_line_with_annotations(
    public_input_file: &PathBuf,
    private_input_file: &PathBuf,
    prover_config_file: &PathBuf,
    prover_parameter_file: &PathBuf,
    output_file: &PathBuf,
    generate_annotations: bool,
) -> Result<(), ProverError> {
    println!("Running prover...");
    // TODO: Add better error handling
    let prover_run_path = std::env::var("CPU_AIR_PROVER").unwrap();

    let mut command = Command::new(prover_run_path);
    command
        .arg("--out-file")
        .arg(output_file)
        .arg("--public-input-file")
        .arg(public_input_file)
        .arg("--private-input-file")
        .arg(private_input_file)
        .arg("--prover-config-file")
        .arg(prover_config_file)
        .arg("--parameter-file")
        .arg(prover_parameter_file);
    if generate_annotations {
        command.arg("--generate-annotations");
    }

    let output = command.output()?;
    if !output.status.success() {
        return Err(ProverError::CommandError(output));
    }

    Ok(())
}
