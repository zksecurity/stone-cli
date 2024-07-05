mod settings;

use crate::utils::write_json_to_file;
use settings::{get_prover_config, get_prover_parameters};
use std::path::PathBuf;
use stone_prover_sdk::error::ProverError;
use stone_prover_sdk::prover::run_prover_from_command_line_with_annotations;

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
    let prover_parameters = get_prover_parameters();
    let prover_parameters_path = tmp_dir.path().join("prover_parameters.json");
    write_json_to_file(&prover_parameters, &prover_parameters_path)?;

    let prover_config = get_prover_config();
    let prover_config_path = tmp_dir.path().join("prover_config.json");
    write_json_to_file(&prover_config, &prover_config_path)?;

    let proof_path = tmp_dir.path().join("proof.json");

    run_prover_from_command_line_with_annotations(
        air_public_input,
        air_private_input,
        prover_config_path.as_path(),
        prover_parameters_path.as_path(),
        proof_path.as_path(),
        true,
    )?;

    Ok(StoneProverResult { proof: proof_path })
}
