use clap::Parser;
use core::fmt::{self, Display};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// TODO: remove unnessary args
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct StarknetArgs {
    #[clap(value_parser)]
    pub filename: PathBuf,

    #[clap(long = "trace_file", value_parser)]
    pub trace_file: Option<PathBuf>,

    #[clap(long = "memory_file")]
    pub memory_file: Option<PathBuf>,

    #[clap(long = "layout", default_value = "plain", value_enum)]
    pub layout: LayoutName,

    #[clap(long = "proof_mode", value_parser)]
    pub proof_mode: bool,

    #[clap(long = "air_public_input", requires = "proof_mode")]
    pub air_public_input: Option<PathBuf>,

    #[clap(
        long = "air_private_input",
        requires_all = ["proof_mode", "trace_file", "memory_file"]
    )]
    pub air_private_input: Option<PathBuf>,

    #[clap(
        long = "cairo_pie_output",
        conflicts_with_all = ["proof_mode", "air_private_input", "air_public_input"]
    )]
    pub cairo_pie_output: Option<PathBuf>,

    #[clap(long = "args")]
    pub args: Option<String>,

    #[clap(long = "args_file", value_parser, conflicts_with = "args")]
    pub args_file: Option<PathBuf>,

    #[clap(long = "print_output", value_parser)]
    pub print_output: bool,

    #[clap(
        long = "append_return_values",
        conflicts_with_all = ["proof_mode", "air_private_input", "air_public_input"]
    )]
    pub append_return_values: bool,

    #[clap(long = "prover_config_file", value_parser)]
    pub prover_config_file: Option<PathBuf>,

    #[clap(long = "verify_on_starknet", value_parser)]
    pub verify_on_starknet: bool,
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
}

impl Display for LayoutName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_str().fmt(f)
    }
}
