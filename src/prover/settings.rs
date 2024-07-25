use serde::{Deserialize, Serialize};
use serde_json::Result;
use stone_prover_sdk::fri::{FriComputer, L1VerifierFriComputer};
use stone_prover_sdk::models::{ProverConfig, StarkParameters};

const DEFAULT_CPU_AIR_PARAMS: &str = include_str!("../../configs/default_cpu_air_params.json");
const DEFAULT_CPU_PROVER_CONFIG: &str =
    include_str!("../../configs/default_cpu_air_prover_config.json");

#[derive(Debug, Serialize, Deserialize)]
struct StatementParameters {
    page_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProverParametersWithHash {
    field: String,
    channel_hash: String,
    commitment_hash: String,
    n_verifier_friendly_commitment_layers: u32,
    pow_hash: String,
    statement: StatementParameters,
    stark: StarkParameters,
    use_extension_field: bool,
    verifier_friendly_channel_updates: bool,
    verifier_friendly_commitment_hash: String,
}

pub fn get_default_prover_parameters(nb_steps: u32) -> Result<ProverParametersWithHash> {
    let mut prover_parameters: ProverParametersWithHash =
        serde_json::from_str(DEFAULT_CPU_AIR_PARAMS)?;

    let fri_parameters = L1VerifierFriComputer.compute_fri_parameters(nb_steps);
    prover_parameters.stark.fri = fri_parameters;
    Ok(prover_parameters)
}

pub fn get_default_prover_config() -> Result<ProverConfig> {
    let default_prover_config: ProverConfig = serde_json::from_str(DEFAULT_CPU_PROVER_CONFIG)?;
    Ok(default_prover_config)
}
