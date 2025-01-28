use tempfile::Builder;

use fs2::FileExt;
use std::{fs::OpenOptions, os::unix::fs::MetadataExt};

use crate::{resource_dir, resource_id, resource_root, resource_tar};

// binary paths relative to the resource directory
const BIN_STONE_V5_PROVER: &str = "executables/cpu_air_prover_v5";
const BIN_STONE_V5_VERIFIER: &str = "executables/cpu_air_verifier_v5";
const BIN_STONE_V6_PROVER: &str = "executables/cpu_air_prover_v6";
const BIN_STONE_V6_VERIFIER: &str = "executables/cpu_air_verifier_v6";
const BIN_CAIRO1_RUN: &str = "executables/cairo1-run";

// environment variables to set
const ENV_CONFIGURE: [(&str, &str); 5] = [
    ("CPU_AIR_PROVER_V5", BIN_STONE_V5_PROVER),
    ("CPU_AIR_VERIFIER_V5", BIN_STONE_V5_VERIFIER),
    ("CPU_AIR_PROVER_V6", BIN_STONE_V6_PROVER),
    ("CPU_AIR_VERIFIER_V6", BIN_STONE_V6_VERIFIER),
    ("CAIRO1_RUN", BIN_CAIRO1_RUN),
];

fn copy_resources(uid: u32, mode: u32) -> anyhow::Result<()> {
    // if the flag file exists, return: setup is already done
    let root_dir = resource_root(uid);
    let flag_file = root_dir.join(format!("flag-{:x}.marker", resource_id()));
    if flag_file.exists() {
        return Ok(());
    }

    // create the resource root directory if it doesn't exist
    match std::fs::create_dir(&root_dir) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => anyhow::bail!("Failed to create resource root directory: {}", e),
    }

    // ensure that the mode on the root directory is correct, ensuring that:
    //
    // 1. we own the directory
    // 2. the directory is not world-writable
    //
    // this is a security measure to prevent other users from tampering with the resources:
    // otherwise, they could race us and replace the resources with malicious ones
    // leading to a privilege escalation attack
    let root_meta = root_dir.metadata()?;
    if root_meta.uid() != uid {
        anyhow::bail!("Resource root directory is not owned by the current user");
    }
    if root_meta.mode() & 0o777 != mode & 0o777 {
        anyhow::bail!("Resource root directory has incorrect permissions");
    }

    // otherwise: take an exclusive lock.
    // this ensures that at-most one instance is in the critical region
    // where we copy the resources and write to the flag file
    let lock = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(root_dir.join(format!("lock-{:x}.marker", resource_id())))
        .map_err(|e| anyhow::anyhow!("Failed to open lock file: {}", e))?;
    lock.lock_exclusive()
        .map_err(|e| anyhow::anyhow!("Failed to lock file: {}", e))?;

    //// Critical Region ////

    // check if the flag file exists now
    if flag_file.exists() {
        return Ok(());
    }

    // remove any remnants of a failed copy
    let res_dir = resource_dir(uid);
    match std::fs::remove_dir_all(&res_dir) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => anyhow::bail!("Failed to remove resources dir: {}", e),
    }
    std::fs::create_dir(&res_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create resources dir: {}", e))?;

    // unpack the resource tar into the stone-cli directory
    let tar = std::io::Cursor::new(resource_tar());
    let decoder = flate2::read::GzDecoder::new(tar);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(&res_dir)
        .map_err(|e| anyhow::anyhow!("Failed to unpack resources: {}", e))?;

    // write to the flag:
    // which is only created if the copy is successful
    std::fs::File::create(&flag_file)?;
    Ok(())
}

pub fn setup() {
    // copy the binaries and corelibs to a directory
    let tmp_dir = Builder::new().prefix("tester").tempdir().unwrap();
    let meta = tmp_dir.path().metadata().unwrap();
    copy_resources(meta.uid(), meta.mode())
        .map_err(|e| anyhow::anyhow!("Failed to copy resources: {}", e))
        .unwrap();

    // set the environment variables (not already set)
    let dir = resource_dir(meta.uid());
    for (env_name, filename) in ENV_CONFIGURE.iter() {
        let full_path = dir.join(filename);
        debug_assert!(
            full_path.exists(), //
            "File not found: {:?}",
            full_path
        );
        if std::env::var(env_name).is_err() {
            std::env::set_var(env_name, full_path);
        }
    }
}
