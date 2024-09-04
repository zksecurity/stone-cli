use crate::define_enum;
pub use crate::prover;

use clap::{Args, Parser, ValueHint};
use prover::config::{ProverConfig, ProverParametersConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "stone-cli",
    version = "0.1.0-alpha",
    about = "CLI for proving Cairo 1 programs on Starknet"
)]
#[command(bin_name = "stone-cli")]
#[allow(clippy::large_enum_variant)]
pub enum Cli {
    Prove(ProveArgs),
    ProveBootloader(ProveBootloaderArgs),
    Verify(VerifyArgs),
    SerializeProof(SerializeArgs),
}

#[derive(Args, Debug)]
#[command(version)]
pub struct ProveArgs {
    #[clap(long = "cairo_version", value_enum, default_value = "cairo1")]
    pub cairo_version: CairoVersion,

    #[clap(long = "cairo_program", value_hint=ValueHint::FilePath)]
    pub cairo_program: PathBuf,

    #[clap(
        long = "program_input",
        help = "Arguments should be spaced, with array elements placed between brackets, e.g. '1 2 [1 2 3]'"
    )]
    pub program_input: Option<String>,

    #[clap(long = "program_input_file", value_hint=ValueHint::FilePath, conflicts_with="program_input")]
    pub program_input_file: Option<PathBuf>,

    #[clap(long = "layout", default_value = "recursive", value_enum)]
    pub layout: LayoutName,

    #[clap(long = "prover_config_file", conflicts_with_all = ["store_full_lde", "use_fft_for_eval", "constraint_polynomial_task_size", "n_out_of_memory_merkle_layers", "table_prover_n_tasks_per_segment"])]
    pub prover_config_file: Option<PathBuf>,

    #[clap(long = "parameter_file", conflicts_with_all = ["field", "channel_hash", "commitment_hash", "n_verifier_friendly_commitment_layers", "pow_hash", "page_hash", "fri_step_list", "last_layer_degree_bound", "n_queries", "proof_of_work_bits", "log_n_cosets", "use_extension_field", "verifier_friendly_channel_updates", "verifier_friendly_commitment_hash"])]
    pub parameter_file: Option<PathBuf>,

    #[clap(long = "output", default_value = "./proof.json")]
    pub output: PathBuf,

    #[clap(flatten)]
    pub parameter_config: ProverParametersConfig,

    #[clap(flatten)]
    pub prover_config: ProverConfig,
}

#[derive(Args, Debug)]
pub struct ProveBootloaderArgs {
    #[clap(long = "cairo_programs", value_hint=ValueHint::FilePath, value_delimiter = ' ', num_args = 1..)]
    pub cairo_programs: Option<Vec<PathBuf>>,

    #[clap(long = "cairo_pies", value_hint=ValueHint::FilePath, value_delimiter = ' ', num_args = 1..)]
    pub cairo_pies: Option<Vec<PathBuf>>,

    #[clap(long = "layout", default_value = "starknet", value_enum)]
    pub layout: LayoutName,

    #[clap(long = "prover_config_file", conflicts_with_all = ["store_full_lde", "use_fft_for_eval", "constraint_polynomial_task_size", "n_out_of_memory_merkle_layers", "table_prover_n_tasks_per_segment"])]
    pub prover_config_file: Option<PathBuf>,

    #[clap(long = "parameter_file", conflicts_with_all = ["field", "channel_hash", "commitment_hash", "n_verifier_friendly_commitment_layers", "pow_hash", "page_hash", "fri_step_list", "last_layer_degree_bound", "n_queries", "proof_of_work_bits", "log_n_cosets", "use_extension_field", "verifier_friendly_channel_updates", "verifier_friendly_commitment_hash"])]
    pub parameter_file: Option<PathBuf>,

    #[clap(long = "output", default_value = "./bootloader_proof.json")]
    pub output: PathBuf,

    #[clap(long = "fact_topologies_output", default_value = "./fact_topologies.json", value_hint=ValueHint::FilePath, help = "Output of bootloader required along with bootloader_proof.json to split proofs for Ethereum")]
    pub fact_topologies_output: PathBuf,

    #[clap(flatten)]
    pub parameter_config: ProverParametersConfig,

    #[clap(flatten)]
    pub prover_config: ProverConfig,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    #[clap(long = "proof", value_parser)]
    pub proof: PathBuf,

    #[clap(long = "annotation_file", value_hint=ValueHint::FilePath)]
    pub annotation_file: Option<PathBuf>,

    #[clap(long = "extra_output_file", value_hint=ValueHint::FilePath)]
    pub extra_output_file: Option<PathBuf>,
}

define_enum! {
    CairoVersion,
    cairo0 => "cairo0",
    cairo1 => "cairo1",
}

define_enum! {
    LayoutName,
    plain => "plain",
    small => "small",
    dex => "dex",
    recursive => "recursive",
    starknet => "starknet",
    starknet_with_keccak => "starknet_with_keccak",
    recursive_large_output => "recursive_large_output",
    recursive_with_poseidon => "recursive_with_poseidon",
    all_solidity => "all_solidity",
    all_cairo => "all_cairo",
    dynamic => "all_cairo",
}

impl std::str::FromStr for LayoutName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(LayoutName::plain),
            "small" => Ok(LayoutName::small),
            "dex" => Ok(LayoutName::dex),
            "recursive" => Ok(LayoutName::recursive),
            "starknet" => Ok(LayoutName::starknet),
            "starknet_with_keccak" => Ok(LayoutName::starknet_with_keccak),
            "recursive_large_output" => Ok(LayoutName::recursive_large_output),
            "recursive_with_poseidon" => Ok(LayoutName::recursive_with_poseidon),
            "all_solidity" => Ok(LayoutName::all_solidity),
            "all_cairo" => Ok(LayoutName::all_cairo),
            "dynamic" => Ok(LayoutName::dynamic),
            _ => Err(()),
        }
    }
}

impl LayoutName {
    pub fn to_cairo_vm_layout(&self) -> cairo_vm::types::layout_name::LayoutName {
        match self {
            LayoutName::plain => cairo_vm::types::layout_name::LayoutName::plain,
            LayoutName::small => cairo_vm::types::layout_name::LayoutName::small,
            LayoutName::dex => cairo_vm::types::layout_name::LayoutName::dex,
            LayoutName::recursive => cairo_vm::types::layout_name::LayoutName::recursive,
            LayoutName::starknet => cairo_vm::types::layout_name::LayoutName::starknet,
            LayoutName::starknet_with_keccak => {
                cairo_vm::types::layout_name::LayoutName::starknet_with_keccak
            }
            LayoutName::recursive_large_output => {
                cairo_vm::types::layout_name::LayoutName::recursive_large_output
            }
            LayoutName::recursive_with_poseidon => {
                cairo_vm::types::layout_name::LayoutName::recursive_with_poseidon
            }
            LayoutName::all_solidity => cairo_vm::types::layout_name::LayoutName::all_solidity,
            LayoutName::all_cairo => cairo_vm::types::layout_name::LayoutName::all_cairo,
            LayoutName::dynamic => cairo_vm::types::layout_name::LayoutName::dynamic,
        }
    }
}

#[derive(Args, Debug, Clone)]
pub struct SerializeArgs {
    #[clap(long = "proof", value_hint=ValueHint::FilePath)]
    pub proof: PathBuf,

    #[clap(long = "network", value_enum)]
    pub network: Network,

    #[clap(long = "output")]
    pub output: PathBuf,

    #[clap(
        long = "annotation_file",
        help = "Required for serializing proofs for Ethereum",
        value_hint=ValueHint::FilePath
    )]
    pub annotation_file: Option<PathBuf>,

    #[clap(
        long = "extra_output_file",
        help = "Required for serializing proofs for Ethereum",
        value_hint=ValueHint::FilePath
    )]
    pub extra_output_file: Option<PathBuf>,
}

define_enum! {
    Network,
    starknet => "starknet",
    ethereum => "ethereum",
}
