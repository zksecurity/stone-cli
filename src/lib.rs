pub mod args;
pub mod bootloader;
pub mod cairo;
pub mod fri;
pub mod prover;
pub mod serialize;
pub mod utils;
pub mod verifier;

pub use prover::config;

mod resources;
mod setup;

pub use setup::setup;

pub fn resource_id() -> u64 {
    resources::RESOURCE_ID
}

// per-user
pub fn resource_root(uid: u32) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("stone-cli-{}", uid))
}

// per-resource snapshot
pub fn resource_dir(uid: u32) -> std::path::PathBuf {
    resource_root(uid).join(format!("{:x}", resource_id()))
}

pub fn resource_tar() -> &'static [u8] {
    resources::RESOURCE_TAR
}
