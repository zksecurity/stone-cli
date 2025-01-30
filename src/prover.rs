pub mod config;

use crate::args::{LayoutName, ProveArgs, ProveBootloaderArgs, StoneVersion};
use crate::sharp::{resolve_automatic_layout, DynamicParamsResponse};
use crate::utils::write_json_to_file;
use cairo_vm::air_public_input::{MemorySegmentAddresses, PublicMemoryEntry};
use config::{ProverConfig, ProverParametersConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PublicInput {
    pub layout: LayoutName,
    pub rc_min: u32,
    pub rc_max: u32,
    pub n_steps: u32,
    pub memory_segments: HashMap<String, MemorySegmentAddresses>,
    pub public_memory: Vec<PublicMemoryEntry>,
    pub dynamic_params: Option<HashMap<String, u32>>,
}

#[derive(Error, Debug)]
pub enum ProverError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    CommandError(ProverCommandError),
}

#[derive(Debug)]
pub struct ProverCommandError {
    output: std::process::Output,
    stone_version: StoneVersion,
}

impl std::fmt::Display for ProverCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to run stone prover with version: {:?}, status: {}, stderr: {}",
            self.stone_version,
            self.output.status,
            String::from_utf8(self.output.stderr.clone()).unwrap()
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AirPublicInput {
    pub layout: String,
    pub rc_min: isize,
    pub rc_max: isize,
    pub n_steps: usize,
    pub memory_segments: HashMap<String, MemorySegmentAddresses>,
    pub public_memory: Vec<PublicMemoryEntry>,
    pub dynamic_params: Option<DynamicParamsResponse>,
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
/// An empty `Result` on success, or an `Error` on failure
pub fn run_stone_prover(
    prove_args: &ProveArgs,
    air_public_input: &PathBuf,
    air_private_input: &PathBuf,
    tmp_dir: &tempfile::TempDir,
) -> Result<(), anyhow::Error> {
    // optionally resolve the dynamic layout
    let air_public_input = if let Some((dynamic_params, cairo_runner)) =
        resolve_automatic_layout(prove_args, tmp_dir)?
    {
        // load the public input and update the layout
        let input = cairo_runner.get_air_public_input()?;

        let mut pi: AirPublicInput = serde_json::from_str(&input.serialize_json().unwrap())?;
        pi.dynamic_params = Some(dynamic_params);

        // save the updated params to the temporary directory
        let air_public_input = tmp_dir.path().join("air_public_input_dyn_params.json");
        write_json_to_file(pi, &air_public_input)?;
        air_public_input
    } else {
        // no dynamic layout, continue as usual
        air_public_input.clone()
    };

    log::debug!("running prover...");
    run_stone_prover_internal(
        &prove_args.parameter_config,
        prove_args.parameter_file.as_ref(),
        &prove_args.prover_config,
        prove_args.prover_config_file.as_ref(),
        &prove_args.output,
        &prove_args.stone_version,
        &air_public_input,
        air_private_input,
        tmp_dir,
    )?;
    log::debug!("prover finished successfully");
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
    log::debug!("Running prover for bootloader...");
    run_stone_prover_internal(
        &prove_bootloader_args.parameter_config,
        prove_bootloader_args.parameter_file.as_ref(),
        &prove_bootloader_args.prover_config,
        prove_bootloader_args.prover_config_file.as_ref(),
        &prove_bootloader_args.output,
        &StoneVersion::V5,
        air_public_input,
        air_private_input,
        tmp_dir,
    )?;
    log::debug!("prover finished successfully");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_stone_prover_internal(
    parameter_config: &ProverParametersConfig,
    parameter_file: Option<&PathBuf>,
    prover_config: &ProverConfig,
    prover_config_file: Option<&PathBuf>,
    output_file: &PathBuf,
    stone_version: &StoneVersion,
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
        stone_version,
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
    stone_version: &StoneVersion,
) -> Result<(), ProverError> {
    // TODO: Add better error handling
    let prover_run_path = match stone_version {
        StoneVersion::V5 => std::env::var("CPU_AIR_PROVER_V5").unwrap(),
        StoneVersion::V6 => std::env::var("CPU_AIR_PROVER_V6").unwrap(),
    };

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
        return Err(ProverError::CommandError(ProverCommandError {
            output,
            stone_version: stone_version.clone(),
        }));
    }

    Ok(())
}
