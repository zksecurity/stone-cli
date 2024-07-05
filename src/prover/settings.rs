use serde::Serialize;
use stone_prover_sdk::models::{FriParameters, ProverConfig, StarkParameters};

#[derive(Debug, Serialize)]
struct StatementParameters {
    page_hash: String,
}

#[derive(Debug, Serialize)]
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

// TODO: make these configurable
pub fn get_prover_parameters() -> ProverParametersWithHash {
    ProverParametersWithHash {
        field: "PrimeField0".to_string(),
        channel_hash: "poseidon3".to_string(),
        commitment_hash: "keccak256_masked160_lsb".to_string(),
        n_verifier_friendly_commitment_layers: 9999,
        pow_hash: "keccak256".to_string(),
        statement: StatementParameters {
            page_hash: "pedersen".to_string(),
        },
        stark: StarkParameters {
            fri: FriParameters {
                fri_step_list: vec![0, 4, 4, 3],
                last_layer_degree_bound: 128,
                n_queries: 10,
                proof_of_work_bits: 30,
            },
            log_n_cosets: 2,
        },
        use_extension_field: false,
        verifier_friendly_channel_updates: true,
        verifier_friendly_commitment_hash: "poseidon3".to_string(),
    }
}

pub fn get_prover_config() -> ProverConfig {
    let mut config = ProverConfig::default();
    config.n_out_of_memory_merkle_layers = 0;
    config
}
