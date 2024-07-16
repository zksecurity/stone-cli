use clap::Parser;
use starknet_adapter::args::Cli;
use starknet_adapter::cairo1::run_cairo1;
use starknet_adapter::prover::run_stone_prover;
use starknet_adapter::utils::{cleanup_tmp_files, set_env_vars};
use starknet_adapter::verifier::run_stone_verifier;
use tempfile::Builder;

const CONFIG: &[u8] = include_bytes!("../configs/env.json");

fn main() -> anyhow::Result<()> {
    set_env_vars(CONFIG);

    let cli = Cli::parse();
    match cli {
        Cli::Prove(args) => {
            let tmp_dir = Builder::new()
                .prefix("starknet-adapter-")
                .tempdir()
                .map_err(|e| anyhow::anyhow!("Failed to create temp dir: {}", e))?;
            let result = run_cairo1(&args, &tmp_dir)
                .map_err(|e| anyhow::anyhow!("Failed to run cairo1: {}", e))
                .and_then(|run_cairo1_result| {
                    run_stone_prover(
                        &args,
                        &run_cairo1_result.air_public_input,
                        &run_cairo1_result.air_private_input,
                        &tmp_dir,
                    )
                    .map_err(|e| anyhow::anyhow!("Failed to run stone prover: {}", e))
                });
            match result {
                Ok(_) => {
                    cleanup_tmp_files(&tmp_dir);
                    Ok(())
                }
                Err(err) => {
                    cleanup_tmp_files(&tmp_dir);
                    Err(err)
                }
            }
        }
        Cli::Verify(args) => {
            run_stone_verifier(&args).map_err(|e| anyhow::anyhow!("Verification failed: {}", e))
        }
    }
}
