use serde::{Deserialize, Serialize};
use serde_json::Result;
use stone_prover_sdk::fri::{FriComputer, L1VerifierFriComputer};
use stone_prover_sdk::models::{ProverConfig, StarkParameters};

const DEFAULT_CPU_AIR_PARAMS: &[u8] = include_bytes!("../../configs/default_cpu_air_params.json");
const DEFAULT_CPU_PROVER_CONFIG: &[u8] =
    include_bytes!("../../configs/default_cpu_air_prover_config.json");

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

impl ProverParametersWithHash {
    pub fn from_json(json_value: serde_json::Value) -> Result<Self> {
        serde_json::from_value(json_value)
    }
}

pub fn get_default_prover_parameters(nb_steps: u32) -> Result<ProverParametersWithHash> {
    let default_prover_params: serde_json::Value = serde_json::from_slice(DEFAULT_CPU_AIR_PARAMS)?;
    let mut prover_parameters = ProverParametersWithHash::from_json(default_prover_params)?;

    let fri_parameters = L1VerifierFriComputer.compute_fri_parameters(nb_steps);
    prover_parameters.stark.fri = fri_parameters;
    Ok(prover_parameters)
}

pub fn get_default_prover_config() -> Result<ProverConfig> {
    let default_prover_config: serde_json::Value =
        serde_json::from_slice(DEFAULT_CPU_PROVER_CONFIG)?;
    let config: ProverConfig = serde_json::from_value(default_prover_config)?;
    Ok(config)
}
