mod vec252;

use anyhow::{Context, Result};
use cairo_args_runner::Felt252;
use cairo_proof_parser::parse;
use itertools::chain;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use vec252::VecFelt252;

/// Runs the Starknet verifier on a given proof file.
///
/// This function reads a proof file, parses its content, prepares the calldata,
/// and executes a Starknet command to verify the proof.
///
/// # Arguments
///
/// * `proof_file` - A reference to the Path of the proof file to be verified.
///
/// # Returns
///
/// A Result indicating success or containing an error if any step fails.
pub fn run_starknet_verifier(proof_file: &Path) -> Result<()> {
    let proof_file_content =
        std::fs::read_to_string(proof_file).context("Failed to read proof file")?;

    let parsed = parse(proof_file_content).context("Failed to parse proof file")?;

    let config: VecFelt252 =
        serde_json::from_str(&parsed.config.to_string()).context("Failed to parse config")?;
    let public_input: VecFelt252 = serde_json::from_str(&parsed.public_input.to_string())
        .context("Failed to parse public input")?;
    let unsent_commitment: VecFelt252 = serde_json::from_str(&parsed.unsent_commitment.to_string())
        .context("Failed to parse unsent commitment")?;
    let witness: VecFelt252 =
        serde_json::from_str(&parsed.witness.to_string()).context("Failed to parse witness")?;

    let proof = chain!(
        config.into_iter(),
        public_input.into_iter(),
        unsent_commitment.into_iter(),
        witness.into_iter()
    );

    // TODO: replace Felt252::from(1) with the correct cairo version
    let calldata = chain!(proof, std::iter::once(Felt252::from(1)));

    let calldata_string = calldata
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    execute_sncast_command(&calldata_string)
}

fn execute_sncast_command(calldata: &str) -> Result<()> {
    println!("Broadcasting tx...");

    const ACCOUNT: &str = "testnet-sepolia";
    const CONTRACT_ADDRESS: &str =
        "0x274d8165a19590bdeaa94d1dd427e2034462d7611754ab3e15714a908c60df7";
    const RPC_URL: &str = "https://free-rpc.nethermind.io/sepolia-juno/v0_7";
    const FUNCTION: &str = "verify_and_register_fact";

    let mut child = Command::new("sncast")
        .args(&[
            "--account",
            ACCOUNT,
            "--url",
            RPC_URL,
            "--wait",
            "invoke",
            "--contract-address",
            CONTRACT_ADDRESS,
            "--function",
            FUNCTION,
            "--calldata",
            calldata,
        ])
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to spawn sncast command")?;

    let stdout = child.stdout.take().expect("Failed to open stdout");
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        println!("{}", line.context("Failed to read line from stdout")?);
    }

    let status = child.wait().context("Failed to wait on sncast command")?;
    if !status.success() {
        anyhow::bail!("sncast command failed with exit code: {:?}", status.code());
    }

    Ok(())
}
