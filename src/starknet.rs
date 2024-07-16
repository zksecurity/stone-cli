mod args;
mod vec252;

use anyhow::Ok;
use anyhow::{Context, Result};
use cairo_felt::Felt252;
use cairo_lang_runner::{Arg, ProfilingInfoCollectionConfig, RunResultValue, SierraCasmRunner};
use cairo_lang_sierra::program::VersionedProgram;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_proof_parser::parse;
use itertools::chain;
use itertools::Itertools;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use vec252::VecFelt252;

use crate::args::ProveArgs;

pub fn run_starknet_verifier(args: &ProveArgs) -> Result<()> {
    run_locally(&args.output)?;

    Ok(())
}

fn run_on_starknet(proof_file: &Path) -> Result<()> {
    let (config, public_input, unsent_commitment, witness) = parse_proof_file(proof_file)?;

    let proof = chain!(
        config.into_iter(),
        public_input.into_iter(),
        unsent_commitment.into_iter(),
        witness.into_iter()
    );

    let calldata = chain!(proof, std::iter::once(Felt252::from(1)));

    let calldata_string = calldata
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    execute_sncast_command(&calldata_string)?;

    Ok(())
}

fn run_locally(proof_file: &Path) -> Result<()> {
    let (config, public_input, unsent_commitment, witness) = parse_proof_file(proof_file)?;

    let proof = chain!(
        config.into_iter(),
        public_input.into_iter(),
        unsent_commitment.into_iter(),
        witness.into_iter()
    )
    .collect_vec();

    println!("proof size: {} felts", proof.len());

    let program =
        std::env::var("CAIRO_VERIFIER").expect("Failed to get CAIRO_VERIFIER environment variable");
    let sierra_program =
        serde_json::from_str::<VersionedProgram>(&fs::read_to_string(program)?)?.into_v1()?;
    println!(
        "program size: {} bytes",
        sierra_program.program.statements.len()
    );

    let runner = SierraCasmRunner::new(
        sierra_program.program.clone(),
        Some(Default::default()),
        OrderedHashMap::default(),
        Some(ProfilingInfoCollectionConfig::default()),
    )
    .unwrap();

    let function = "main";
    let func = runner.find_function(function).unwrap();
    let proof_arg = Arg::Array(proof.into_iter().map(Arg::Value).collect_vec());
    let cairo_version_arg = Arg::Value(Felt252::from(1));
    let args = &[proof_arg, cairo_version_arg];
    let result = runner
        .run_function_with_starknet_context(func, args, Some(u32::MAX as usize), Default::default())
        .unwrap();

    println!("gas_counter: {}", result.gas_counter.unwrap());

    match result.value {
        RunResultValue::Success(msg) => {
            println!("{:?}", msg);
        }
        RunResultValue::Panic(msg) => {
            panic!("{:?}", msg);
        }
    }

    Ok(())
}

fn parse_proof_file(proof_file: &Path) -> Result<(VecFelt252, VecFelt252, VecFelt252, VecFelt252)> {
    let proof_file_content = std::fs::read_to_string(proof_file)?;
    let parsed = parse(proof_file_content)?;

    Ok((
        serde_json::from_str(&parsed.config.to_string())?,
        serde_json::from_str(&parsed.public_input.to_string())?,
        serde_json::from_str(&parsed.unsent_commitment.to_string())?,
        serde_json::from_str(&parsed.witness.to_string())?,
    ))
}

fn execute_sncast_command(calldata: &str) -> Result<()> {
    println!("Broadcasting tx...");

    const ACCOUNT: &str = "testnet-sepolia";
    // Integrity verifier contract address (https://github.com/HerodotusDev/integrity)
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
