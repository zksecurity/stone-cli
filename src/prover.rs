pub mod config;

use crate::args::{LayoutName, ProveArgs, ProveBootloaderArgs, StoneVersion};
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

// path to the private key
const ENV_SHARP_KEY_PATH: &str = "SHARP_KEY_PATH";

// decryption key for the private key
const ENV_SHARP_PASSWORD: &str = "SHARP_KEY_PASSWD";

// SHARP API URL
const SHARP_API_URL: &str = "https://sharp-bi.provingservice.io/sharp_bi";

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

use reqwest;

const SHARP_CERT: &str = include_str!("./sharp-server.crt");

/// Get
fn get_sharp_cert() -> reqwest::Certificate {
    reqwest::Certificate::from_pem(SHARP_CERT.as_bytes()).unwrap()
}

/// Get an identity for the SHARP API
///
/// For easy of use, this method will attempt to
/// obtain a key using the following methods:
///
/// - PEM without a password
/// - PEM with a password
/// - DER with a password
fn get_identity() -> Result<reqwest::Identity, anyhow::Error> {
    // read the key
    let key_path = match std::env::var(ENV_SHARP_KEY_PATH) {
        Ok(path) => path,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "Using a method which requires a SHARP certificate. Please set the \"{}\" enviroment variable",
                ENV_SHARP_KEY_PATH
            ))
        }
    };
    let key_bytes = fs::read(key_path).map_err(|e| anyhow::anyhow!("Failed to read key: {}", e))?;

    // try without a password
    if let Ok(id) = reqwest::Identity::from_pem(&key_bytes) {
        return Ok(id);
    }

    // try with a password
    let cert_pass = match std::env::var(ENV_SHARP_PASSWORD) {
        Ok(pass) => pass,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "PEM key requires password, please set the \"{}\" enviroment variable",
                ENV_SHARP_PASSWORD
            ))
        }
    };
    if let Ok(id) = reqwest::Identity::from_pkcs8_pem(&key_bytes, cert_pass.as_bytes()) {
        return Ok(id);
    }
    if let Ok(id) = reqwest::Identity::from_pkcs12_der(&key_bytes, &cert_pass) {
        return Ok(id);
    }
    Err(anyhow::anyhow!("Failed to load identity"))
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
) -> Result<(), ProverError> {
    // resolve dynamic layout using the SHARP API
    if LayoutName::dynamic == prove_args.layout {
        println!("Using the SHARP API to resolve dynamic layout...");

        let identity = get_identity().unwrap(); // TODO
        let certificate = get_sharp_cert();

        let client = reqwest::blocking::ClientBuilder::new()
            .identity(identity)
            .add_root_certificate(certificate)
            .build()
            .unwrap();

        client
            .get(format!("{}/is_alive", SHARP_API_URL))
            .send()
            .unwrap();

        return Ok(());
    }

    println!("Running prover...");

    run_stone_prover_internal(
        &prove_args.parameter_config,
        prove_args.parameter_file.as_ref(),
        &prove_args.prover_config,
        prove_args.prover_config_file.as_ref(),
        &prove_args.output,
        &prove_args.stone_version,
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
        &StoneVersion::V5,
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
