use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::ffi::OsStr;
use std::fs::{metadata, remove_file, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use thiserror::Error;

const CONFIG: &str = include_str!("configs/env.json");

static DISTS: Lazy<HashMap<(Os, Arch), Artifacts>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert((Os::Linux, Arch::Amd64), Artifacts {
        url: "https://github.com/zksecurity/stone-cli/releases/download/v0.1.0-alpha/stone-cli-linux-x86_64.tar.gz".to_string(),
        sha256_sums: vec![
            "2a100342be0660fc8363e7ac6230ffd9ea0937e7afc35265b7af1595d64dcff4".to_string(),
            "039d81f62004613f34bfb39b10c4b6b234e22a2b26c8b68c07701e5edaa98a33".to_string(),
            "a13a1ae5a5f4109489bbe93f78a12778ec99a896e9f4fbe3c88f38d1f61612b2".to_string(),
        ],
    });
    m.insert((Os::MacOS, Arch::Aarch64), Artifacts {
        url: "https://github.com/zksecurity/stone-cli/releases/download/v0.1.0-alpha/stone-cli-macos-aarch64.tar.gz".to_string(),
        sha256_sums: vec![
            "22b3d5a9d9c9bbaab6196a3ff4d372e765fa75c50272d20fc562917849974a2b".to_string(),
            "9d56eaa56eda5caa6853761f93d363dc3e9e9af27cf142cd0178dbcd4f61d405".to_string(),
            "bfd92c9f8c6be41a0486c936b0f12df153ee2743edbf782e21f15fa56e3bdb70".to_string(),
        ],
    });
    m
});

#[derive(Debug, Error)]
enum ConversionError {
    #[error("Unsupported architecture: {0}")]
    UnsupportedArchitecture(String),
    #[error("Unsupported operating system: {0}")]
    UnsupportedOperatingSystem(String),
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum Os {
    Linux,
    MacOS,
}

impl TryInto<Os> for &str {
    type Error = ConversionError;

    fn try_into(self) -> Result<Os, Self::Error> {
        match self {
            "linux" => Ok(Os::Linux),
            "macos" => Ok(Os::MacOS),
            _ => Err(ConversionError::UnsupportedOperatingSystem(
                self.to_string(),
            )),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
enum Arch {
    Aarch64,
    Amd64,
}

impl TryInto<Arch> for &str {
    type Error = ConversionError;

    fn try_into(self) -> Result<Arch, Self::Error> {
        match self {
            "aarch64" => Ok(Arch::Aarch64),
            "x86_64" => Ok(Arch::Amd64),
            _ => Err(ConversionError::UnsupportedArchitecture(self.to_string())),
        }
    }
}

#[derive(Deserialize)]
struct Artifacts {
    url: String,
    sha256_sums: Vec<String>,
}

#[derive(Deserialize)]
struct Config {
    download_dir: String,
    file_names: Vec<String>,
    #[allow(dead_code)]
    env_names: Vec<String>,
}

fn main() {
    let config: Config = serde_json::from_str(CONFIG).expect("Failed to parse config file");
    download_executables(&config);
    download_corelib_repo();
}

fn download_executables(config: &Config) {
    let download_dir = Path::new(env!("HOME")).join(&config.download_dir);
    if !download_dir.exists() {
        std::fs::create_dir_all(&download_dir).expect("Failed to create download directory");
    }

    if config
        .file_names
        .iter()
        .all(|filename| download_dir.join(filename).exists())
    {
        return;
    }

    // look up the stone-prover distribution for the current OS and architecture
    let os = OS.try_into().unwrap();
    let arch = ARCH.try_into().unwrap();
    let dist = match DISTS.get(&(os, arch)) {
        Some(dist) => dist,
        None => panic!("Unsupported OS or architecture {}/{}", OS, ARCH),
    };

    let url = &dist.url;
    let download_file_name = Path::new(url)
        .file_name()
        .expect("Failed to get the last path of the URL");
    let download_file_path = download_dir.join(download_file_name);
    download_from_url(url, &download_file_path);
    unzip_file(&download_file_path, &download_dir);
    move_files(&download_dir, download_file_name, &config.file_names);
    remove_file(&download_file_path).expect("Failed to remove tar file");

    let sha256_sums = &dist.sha256_sums;
    validate_unpacked_files(&download_dir, &config.file_names, sha256_sums);
    set_execute_permissions(config);
}

fn set_execute_permissions(config: &Config) {
    let download_dir = Path::new(env!("HOME")).join(&config.download_dir);
    for filename in config.file_names.iter() {
        let file_path = download_dir.join(filename);
        if !file_path.exists() {
            panic!("File {} does not exist", file_path.display());
        }
        let mut permissions = metadata(&file_path)
            .expect("Failed to get file metadata")
            .permissions();
        permissions.set_mode(0o755);
        set_permissions(&file_path, permissions).expect("Failed to set file permissions");
    }
}

fn download_corelib_repo() {
    let download_dir = Path::new(env!("HOME")).join(".stone-cli");
    let corelib_dir = Path::new(env!("HOME")).join(download_dir.join("corelib"));
    let url = "https://github.com/starkware-libs/cairo/releases/download/v2.8.4/release-x86_64-unknown-linux-musl.tar.gz";
    let download_file_path = download_dir.join("release-x86_64-unknown-linux-musl.tar.gz");
    if !corelib_dir.exists() {
        download_from_url(url, &download_file_path);
        unzip_file(&download_file_path, &download_dir);
        remove_file(&download_file_path).expect("Failed to remove tar file");

        if !std::process::Command::new("cp")
            .args([
                "-r",
                &download_dir.join("cairo").join("corelib").to_string_lossy(),
                &download_dir.to_string_lossy(),
            ])
            .status()
            .expect("Failed to copy corelib directory")
            .success()
        {
            panic!("Failed to copy corelib directory. Please check if the directory exists in the current directory.");
        }

        if !std::process::Command::new("rm")
            .args(["-rf", &download_dir.join("cairo").to_string_lossy()])
            .status()
            .expect("Failed to remove the repository")
            .success()
        {
            panic!("Failed to remove the repository. Please check your permissions and try again.");
        }
    }
}

fn unzip_file(download_file_path: &Path, download_dir: &Path) {
    let tar_gz = std::fs::File::open(download_file_path).expect("Failed to open tar.gz file");
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive
        .unpack(download_dir)
        .expect("Failed to unpack tar.gz file");
}

fn move_files(download_dir: &Path, download_file_name: &OsStr, file_names: &[String]) {
    // file name has the following syntax ("stone-cli-macos-aarch64.tar.gz"), so we need to split by "." and take the last element
    let files_dir = download_file_name
        .to_str()
        .expect("Failed to convert OsStr to str")
        .split('.')
        .next()
        .unwrap();
    let download_dir = Path::new(env!("HOME")).join(download_dir);
    for filename in file_names.iter() {
        let file_path = download_dir.join(files_dir).join(filename);
        if !file_path.exists() {
            panic!("File {} does not exist", file_path.display());
        }
        let new_file_path = download_dir.join(filename);
        std::fs::rename(&file_path, &new_file_path).expect("Failed to move file");
    }
    // Remove the directory containing the unpacked files
    let files_dir_path = download_dir.join(files_dir);
    if files_dir_path.exists() {
        std::fs::remove_dir_all(&files_dir_path).unwrap_or_else(|e| {
            panic!(
                "Failed to remove directory {}: {}",
                files_dir_path.display(),
                e
            )
        });
    }
}

fn validate_unpacked_files(download_dir: &Path, file_names: &[String], sha256_sums: &[String]) {
    let unpacked_files: Vec<_> = std::fs::read_dir(download_dir)
        .expect("Failed to read download directory")
        .map(|entry| {
            entry
                .expect("Failed to read directory entry")
                .file_name()
                .into_string()
                .expect("Failed to convert OsString to String")
        })
        .collect();

    for unpacked_file in unpacked_files {
        if !file_names.contains(&unpacked_file) {
            panic!(
                "Unexpected file {} found in download directory",
                unpacked_file
            );
        }

        let index = file_names
            .iter()
            .position(|name| name == &unpacked_file)
            .unwrap();
        let sha256_sum = &sha256_sums[index];
        let file_path = download_dir.join(unpacked_file);
        let calculated_sha256 = sha256::try_digest(&file_path).unwrap();
        if calculated_sha256 != *sha256_sum {
            panic!("File {} has incorrect sha256 sum", file_path.display());
        }
    }
}

fn download_from_url(url: &str, download_file_path: &Path) {
    let response = reqwest::blocking::get(url).expect("Failed to download file");
    let mut file = std::fs::File::create(download_file_path).expect("Failed to create file");
    std::io::copy(
        &mut response.bytes().expect("Failed to read response").as_ref(),
        &mut file,
    )
    .expect("Failed to write to file");
}
