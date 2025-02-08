pub mod args;
pub mod bootloader;
pub mod cairo;
pub mod fri;
pub mod prover;
pub mod serialize;
pub mod sharp;
pub mod utils;
pub mod verifier;

pub use prover::config;
pub use setup::{
    path_corelib,           // path to the corelib directory
    path_stone_v5_prover,   // path to the stone v5 prover binary
    path_stone_v5_verifier, // path to the stone v5 verifier binary
    path_stone_v6_prover,   // path to the stone v6 prover binary
    path_stone_v6_verifier, // path to the stone v6 verifier binary
};

mod setup;
