use clap::Parser;
use stone_cli::args::Cli;
use stone_cli::cairo1::run_cairo1;
use stone_cli::prover::run_stone_prover;
use stone_cli::serialize::serialize_proof;
use stone_cli::utils::{cleanup_tmp_files, parse, set_env_vars};
use stone_cli::verifier::run_stone_verifier;
use tempfile::Builder;

const CONFIG: &str = include_str!("../configs/env.json");

fn main() -> anyhow::Result<()> {
    let config = parse(CONFIG);
    set_env_vars(&config);

    let cli = Cli::parse();
    match cli {
        Cli::Prove(args) => {
            let tmp_dir = Builder::new()
                .prefix("stone-cli-")
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
        Cli::SerializeProof(args) => {
            serialize_proof(&args).map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))
        }
    }
}
