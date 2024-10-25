pub mod config;

use crate::args::{ProveArgs, ProveBootloaderArgs};
use crate::utils::write_json_to_file;
use config::{ProverConfig, ProverParametersConfig};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use stone_prover_sdk::error::ProverError;
use stone_prover_sdk::models::PublicInput;

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
/// An empty `Result` on success, or an `Error` on failure
pub fn run_stone_prover(
    prove_args: &ProveArgs,
    air_public_input: &PathBuf,
    air_private_input: &PathBuf,
    tmp_dir: &tempfile::TempDir,
) -> Result<(), ProverError> {
    println!("Running prover...");

    run_stone_prover_internal(
        &prove_args.parameter_config,
        prove_args.parameter_file.as_ref(),
        &prove_args.prover_config,
        prove_args.prover_config_file.as_ref(),
        &prove_args.output,
        air_public_input,
        air_private_input,
        tmp_dir,
    )?;

    println!("Prover finished successfully");
    Ok(())
}

/// Runs the Stone prover for bootloader with the given inputs
///
/// # Arguments
///
/// * `prove_bootloader_args` - Arguments for proving bootloader
/// * `air_public_input` - Path to the public input file
/// * `air_private_input` - Path to the private input file
/// * `tmp_dir` - Temporary directory for intermediate files
///
/// # Returns
///
/// An empty `Result` on success, or an `Error` on failure
pub fn run_stone_prover_bootloader(
    prove_bootloader_args: &ProveBootloaderArgs,
    air_public_input: &PathBuf,
    air_private_input: &PathBuf,
    tmp_dir: &tempfile::TempDir,
) -> Result<(), ProverError> {
    println!("Running prover for bootloader...");

    run_stone_prover_internal(
        &prove_bootloader_args.parameter_config,
        prove_bootloader_args.parameter_file.as_ref(),
        &prove_bootloader_args.prover_config,
        prove_bootloader_args.prover_config_file.as_ref(),
        &prove_bootloader_args.output,
        air_public_input,
        air_private_input,
        tmp_dir,
    )?;

    println!("Prover finished successfully");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_stone_prover_internal(
    parameter_config: &ProverParametersConfig,
    parameter_file: Option<&PathBuf>,
    prover_config: &ProverConfig,
    prover_config_file: Option<&PathBuf>,
    output_file: &PathBuf,
    air_public_input: &PathBuf,
    air_private_input: &PathBuf,
    tmp_dir: &tempfile::TempDir,
) -> Result<(), ProverError> {
    let tmp_prover_parameters_path = tmp_dir.path().join("prover_parameters.json");

    let prover_parameters_path = if let Some(parameter_file) = &parameter_file {
        parameter_file
    } else {
        let air_public_input_json: PublicInput =
            serde_json::from_str(&fs::read_to_string(air_public_input)?).unwrap();
        let prover_parameters =
            ProverParametersConfig::new(air_public_input_json.n_steps, parameter_config).unwrap();
        write_json_to_file(prover_parameters, &tmp_prover_parameters_path)?;
        &tmp_prover_parameters_path
    };

    let tmp_prover_config_path = tmp_dir.path().join("prover_config.json");

    let prover_config_path = if let Some(prover_config_file) = &prover_config_file {
        prover_config_file
    } else {
        let prover_config = ProverConfig::new(prover_config).unwrap();
        write_json_to_file(prover_config, &tmp_prover_config_path)?;
        &tmp_prover_config_path
    };

    run_prover_from_command_line_with_annotations(
        air_public_input,
        air_private_input,
        prover_config_path,
        prover_parameters_path,
        output_file,
        true,
    )?;

    Ok(())
}

fn run_prover_from_command_line_with_annotations(
    public_input_file: &PathBuf,
    private_input_file: &PathBuf,
    prover_config_file: &PathBuf,
    prover_parameter_file: &PathBuf,
    output_file: &PathBuf,
    generate_annotations: bool,
) -> Result<(), ProverError> {
    // TODO: Add better error handling
    let prover_run_path = std::env::var("CPU_AIR_PROVER_V6").unwrap();

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
