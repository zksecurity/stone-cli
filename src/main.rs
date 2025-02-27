use clap::Parser;
use stone_cli::args::Cli;
use stone_cli::bootloader::run_bootloader;
use stone_cli::cairo::run_cairo;
use stone_cli::prover::{run_stone_prover, run_stone_prover_bootloader};
use stone_cli::serialize::serialize_proof;
use stone_cli::utils::cleanup_tmp_files;
use stone_cli::verifier::run_stone_verifier;
use tempfile::Builder;

fn main() -> anyhow::Result<()> {
    // create a temporary directory for intermediate files
    let tmp_dir = Builder::new()
        .prefix("stone-cli-")
        .tempdir()
        .map_err(|e| anyhow::anyhow!("Failed to create temp dir: {}", e))?;

    let cli = Cli::parse();
    match cli {
        Cli::Prove(args) => {
            let result = run_cairo(&args, &tmp_dir)
                .map_err(|e| anyhow::anyhow!("Failed to run cairo: {}", e))
                .and_then(|run_cairo_result| {
                    run_stone_prover(
                        &args,
                        &run_cairo_result.air_public_input,
                        &run_cairo_result.air_private_input,
                        &tmp_dir,
                    )
                    .map_err(|e| anyhow::anyhow!("Failed to run stone prover: {}", e))
                });
            match result {
                Ok(_) => {
                    println!("Created proof at {:?}", args.output);
                    cleanup_tmp_files(&tmp_dir);
                    Ok(())
                }
                Err(err) => {
                    cleanup_tmp_files(&tmp_dir);
                    Err(err)
                }
            }
        }
        Cli::ProveBootloader(args) => {
            let tmp_dir = Builder::new()
                .prefix("stone-cli-")
                .tempdir()
                .map_err(|e| anyhow::anyhow!("Failed to create temp dir: {}", e))?;
            let result = run_bootloader(&args, &tmp_dir)
                .map_err(|e| anyhow::anyhow!("Bootloader failed: {}", e))
                .and_then(|run_bootloader_result| {
                    run_stone_prover_bootloader(
                        &args,
                        &run_bootloader_result.air_public_input,
                        &run_bootloader_result.air_private_input,
                        &tmp_dir,
                    )
                    .map_err(|e| anyhow::anyhow!("Failed to run stone prover: {}", e))
                });
            match result {
                Ok(_) => {
                    println!("Created proof at {:?}", args.output);
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
            run_stone_verifier(args).map_err(|e| anyhow::anyhow!("Verification failed: {}", e))
        }
        Cli::SerializeProof(args) => {
            serialize_proof(args).map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))
        }
    }
}
