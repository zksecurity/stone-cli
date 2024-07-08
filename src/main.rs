use starknet_adapter::cairo1::run_cairo1;
use starknet_adapter::prover::run_stone_prover;
use starknet_adapter::starknet::run_starknet_verifier;
use starknet_adapter::utils::{cleanup_tmp_files, handle_error};
use tempfile::Builder;

fn main() -> anyhow::Result<()> {
    let tmp_dir = Builder::new()
        .prefix("starknet-adapter-")
        .tempdir()
        .map_err(|e| anyhow::anyhow!("Failed to create temp dir: {}", e))?;

    let result = run_cairo1(std::env::args(), &tmp_dir)
        .map_err(|e| anyhow::anyhow!("Failed to run cairo1: {}", e))
        .and_then(|run_cairo1_result| {
            run_stone_prover(
                &run_cairo1_result.air_public_input,
                &run_cairo1_result.air_private_input,
                &tmp_dir,
            )
            .map_err(|e| anyhow::anyhow!("Failed to run stone prover: {}", e))
        })
        .and_then(|run_stone_prover_result| {
            run_starknet_verifier(&run_stone_prover_result.proof)
                .map_err(|e| anyhow::anyhow!("Failed to run starknet verifier: {}", e))
        });

    match result {
        Ok(_) => {
            cleanup_tmp_files(&tmp_dir);
            Ok(())
        }
        Err(err) => {
            cleanup_tmp_files(&tmp_dir);
            handle_error(&err);
            Err(err)
        }
    }
}
