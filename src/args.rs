use clap::{Args, Parser, ValueHint};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "starknet-adapter",
    version = "0.1.0-alpha",
    about = "CLI for proving Cairo 1 programs on Starknet"
)]
#[command(bin_name = "starknet-adapter")]
pub enum Cli {
    Prove(ProveArgs),
    Verify(VerifyArgs),
}

#[derive(Args, Debug)]
#[command(version)]
pub struct ProveArgs {
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

    #[clap(long = "prover_config_file")]
    pub prover_config_file: Option<PathBuf>,

    #[clap(long = "parameter_file")]
    pub parameter_file: Option<PathBuf>,

    #[clap(long = "output", default_value = "./proof.json")]
    pub output: PathBuf,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    #[clap(long = "proof", value_parser)]
    pub proof: PathBuf,
}
/// Enum representing the name of a Cairo Layout
#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq, Hash, clap::ValueEnum)]
#[allow(non_camel_case_types)]
pub enum LayoutName {
    plain,
    small,
    dex,
    recursive,
    starknet,
    starknet_with_keccak,
    recursive_large_output,
    recursive_with_poseidon,
    all_solidity,
    all_cairo,
    dynamic,
}

impl LayoutName {
    pub fn to_str(self) -> &'static str {
        match self {
            LayoutName::plain => "plain",
            LayoutName::small => "small",
            LayoutName::dex => "dex",
            LayoutName::recursive => "recursive",
            LayoutName::starknet => "starknet",
            LayoutName::starknet_with_keccak => "starknet_with_keccak",
            LayoutName::recursive_large_output => "recursive_large_output",
            LayoutName::recursive_with_poseidon => "recursive_with_poseidon",
            LayoutName::all_solidity => "all_solidity",
            LayoutName::all_cairo => "all_cairo",
            LayoutName::dynamic => "all_cairo",
        }
    }

    pub fn from_str(layout: &str) -> Option<Self> {
        match layout {
            "plain" => Some(LayoutName::plain),
            "small" => Some(LayoutName::small),
            "dex" => Some(LayoutName::dex),
            "recursive" => Some(LayoutName::recursive),
            "starknet" => Some(LayoutName::starknet),
            "starknet_with_keccak" => Some(LayoutName::starknet_with_keccak),
            "recursive_large_output" => Some(LayoutName::recursive_large_output),
            "recursive_with_poseidon" => Some(LayoutName::recursive_with_poseidon),
            "all_solidity" => Some(LayoutName::all_solidity),
            "all_cairo" => Some(LayoutName::all_cairo),
            "dynamic" => Some(LayoutName::dynamic),
            _ => None,
        }
    }
}
