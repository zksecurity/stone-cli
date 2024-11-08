use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::fs::{metadata, remove_dir_all, remove_file, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use thiserror::Error;

const CONFIG: &str = include_str!("configs/env.json");

static DISTS: Lazy<HashMap<(Os, Arch), Vec<Artifacts>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(
        (Os::Linux, Arch::Amd64),
        vec![
            Artifacts {
                url: "https://github.com/zksecurity/stone-cli/releases/download/v0.1.0/cairo1-run-v2.0.0-rc0-x86_64.tar.gz".to_string(),
                sha256_sum: "3c15ed3a8b9fce2e18c7923a0c9c5e07058f3432a2f6d2ef3f66384ad0298e23".to_string(),
            },
            Artifacts {
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.1/cpu_air_prover-x86_64".to_string(),
                sha256_sum: "d5345e3e72a6180dabcec79ef35cefc735ea72864742e1cc117869da7d122ee5".to_string(),
            },
            Artifacts {
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.1/cpu_air_verifier-x86_64".to_string(),
                sha256_sum: "8ed3cad6cf3fb10f5a600af861c28b8f427244b0c2de920f1c18ea78371a66a9".to_string(),
            },
            Artifacts {
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.2/cpu_air_prover-x86_64".to_string(),
                sha256_sum: "ec33129a15b888b7946f17fe46ca888bfed2f4d86ac4e3fc7fae787f8162ca9e".to_string(),
            },
            Artifacts {
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.2/cpu_air_verifier-x86_64".to_string(),
                sha256_sum: "f83d66f5f9cd60c070fee02524d4ccb86b1c37865d75c022fbd54c349d7d972b".to_string(),
            },
        ],
    );
    m.insert(
        (Os::MacOS, Arch::Aarch64),
        vec![
            Artifacts {
                url: "https://github.com/zksecurity/stone-cli/releases/download/v0.1.0/cairo1-run-v2.0.0-rc0-aarch64.tar.gz".to_string(),
                sha256_sum: "136dbf856b5909a3f4197a038ad57dc10b34201e8f97a3045a0699592d2442db".to_string(),
            },
            Artifacts {
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.1/cpu_air_prover-arm64".to_string(),
                sha256_sum: "d91e8328b7a228445dda0b9d1acb21a86ab894727737e2d70a0210179b90f00e".to_string(),
            },
            Artifacts {
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.1/cpu_air_verifier-arm64".to_string(),
                sha256_sum: "fc4090e3395e101f3481efc247ad590e5db7704c31321480522904d68ba5d009".to_string(),
            },
            Artifacts {
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.2/cpu_air_prover-arm64".to_string(),
                sha256_sum: "9d56eaa56eda5caa6853761f93d363dc3e9e9af27cf142cd0178dbcd4f61d405".to_string(),
            },
            Artifacts {
                url: "https://github.com/dipdup-io/stone-packaging/releases/download/v3.0.2/cpu_air_verifier-arm64".to_string(),
                sha256_sum: "bfd92c9f8c6be41a0486c936b0f12df153ee2743edbf782e21f15fa56e3bdb70".to_string(),
            },
        ],
    );
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
    sha256_sum: String,
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
    let tmp_dir = tempfile::Builder::new()
        .prefix("stone-cli-")
        .tempdir()
        .expect("Failed to create temp dir");
    download_executables(&config, tmp_dir.path());
    download_corelib_repo(&config, tmp_dir.path());
}

fn download_executables(config: &Config, tmp_dir_path: &Path) {
    let download_dir = Path::new(env!("HOME"))
        .join(&config.download_dir)
        .join("executables");
    if download_dir.exists() {
        if config
            .file_names
            .iter()
            .all(|filename| download_dir.join(filename).exists())
        {
            return;
        } else {
            std::fs::remove_dir_all(&download_dir)
                .expect("Failed to remove existing download directory");
        }
    }

    let tmp_download_dir = tmp_dir_path.join("executables");
    std::fs::create_dir_all(&tmp_download_dir).expect("Failed to create tmp_download_dir");

    // look up the stone-prover distribution for the current OS and architecture
    let os = OS.try_into().unwrap();
    let arch = ARCH.try_into().unwrap();
    let dist = match DISTS.get(&(os, arch)) {
        Some(dist) => dist,
        None => panic!("Unsupported OS or architecture {}/{}", OS, ARCH),
    };

    let cairo1_run_url = &dist[0].url;
    let cairo1_run_sha256_sum = &dist[0].sha256_sum;
    let cairo1_run_zip_file_name = Path::new(cairo1_run_url)
        .file_name()
        .expect("Failed to get the last path of the URL");
    let cairo1_run_zip_file_path = tmp_download_dir.join(cairo1_run_zip_file_name);
    download_from_url(cairo1_run_url, &cairo1_run_zip_file_path);
    unzip_file(&cairo1_run_zip_file_path, &tmp_download_dir);
    // file name has the following syntax `cairo1-run-v2.0.0-rc0-x86_64.tar.gz`, so we need to split by "." and remove the last two elements
    let cairo1_run_unzip_dir_name = cairo1_run_zip_file_name
        .to_str()
        .expect("Failed to convert OsStr to str")
        .split('.')
        .collect::<Vec<&str>>()
        .split_at(
            cairo1_run_zip_file_name
                .to_str()
                .unwrap()
                .split('.')
                .count()
                - 2,
        )
        .0
        .join(".");
    let cairo1_run_file_name = &config.file_names[0];
    let cairo1_run_new_file_path = tmp_download_dir.join(cairo1_run_file_name);
    // move the file from the unzip directory to the download directory
    std::fs::rename(
        tmp_download_dir
            .join(&cairo1_run_unzip_dir_name)
            .join(cairo1_run_file_name),
        &cairo1_run_new_file_path,
    )
    .expect("Failed to move file");
    remove_dir_all(tmp_download_dir.join(cairo1_run_unzip_dir_name))
        .expect("Failed to remove the directory containing the unpacked files");
    remove_file(&cairo1_run_zip_file_path).expect("Failed to remove tar file");
    validate_file(&cairo1_run_new_file_path, cairo1_run_sha256_sum);
    set_execute_permissions(&cairo1_run_new_file_path);

    // download the stone prover and verifier binaries
    for (i, item) in dist.iter().enumerate().skip(1) {
        let url = &item.url;
        let sha256_sum = &item.sha256_sum;
        let download_file_name = Path::new(url)
            .file_name()
            .expect("Failed to get the last path of the URL");
        let download_file_path = tmp_download_dir.join(download_file_name);
        download_from_url(url, &download_file_path);
        let new_file_name = &config.file_names[i];
        let new_file_path = tmp_download_dir.join(new_file_name);
        // move the file from the unzip directory to the download directory
        std::fs::rename(&download_file_path, &new_file_path).expect("Failed to move file");
        validate_file(&new_file_path, sha256_sum);
        set_execute_permissions(&new_file_path);
    }

    // copy all files from tmp_download_dir to download_dir
    std::fs::create_dir_all(&download_dir).expect("Failed to create download directory");
    for entry in std::fs::read_dir(&tmp_download_dir).expect("Failed to read tmp directory") {
        let entry = entry.expect("Failed to get directory entry");
        let file_name = entry.file_name();
        let source = entry.path();
        let destination = download_dir.join(&file_name);
        std::fs::copy(&source, &destination).unwrap_or_else(|_| {
            // In case of failure, remove all existing files in download_dir
            for entry in
                std::fs::read_dir(&download_dir).expect("Failed to read download directory")
            {
                let entry = entry.expect("Failed to get directory entry");
                let path = entry.path();
                if path.is_file() {
                    std::fs::remove_file(&path).expect("Failed to remove existing file");
                }
            }
            panic!(
                "Failed to copy {} to {}",
                source.display(),
                destination.display()
            )
        });
    }

    // clean up temporary directory
    remove_dir_all(&tmp_download_dir).expect("Failed to remove temporary directory");
}

fn set_execute_permissions(file_path: &Path) {
    if !file_path.exists() {
        panic!("File {} does not exist", file_path.display());
    }
    let mut permissions = metadata(file_path)
        .expect("Failed to get file metadata")
        .permissions();
    permissions.set_mode(0o755);
    set_permissions(file_path, permissions).expect("Failed to set file permissions");
}

fn download_corelib_repo(config: &Config, tmp_dir_path: &Path) {
    let download_dir = Path::new(env!("HOME")).join(&config.download_dir);
    let corelib_dir = download_dir.join("corelib");
    if corelib_dir.exists() {
        return;
    }

    let url = "https://github.com/starkware-libs/cairo/releases/download/v2.9.0-dev.0/release-x86_64-unknown-linux-musl.tar.gz";
    let tmp_download_file_path = tmp_dir_path.join("release-x86_64-unknown-linux-musl.tar.gz");
    download_from_url(url, &tmp_download_file_path);
    unzip_file(&tmp_download_file_path, tmp_dir_path);

    if !std::process::Command::new("cp")
        .args([
            "-r",
            &tmp_dir_path.join("cairo").join("corelib").to_string_lossy(),
            &download_dir.to_string_lossy(),
        ])
        .status()
        .expect("Failed to copy corelib directory")
        .success()
    {
        panic!("Failed to copy corelib directory. Please check if the directory exists in the current directory.");
    }

    if !std::process::Command::new("rm")
        .args(["-rf", &tmp_dir_path.join("cairo").to_string_lossy()])
        .status()
        .expect("Failed to remove the repository")
        .success()
    {
        panic!(
            "Failed to remove the cairo directory. Please check your permissions and try again."
        );
    }

    // clean up temporary directory
    remove_dir_all(tmp_dir_path).expect("Failed to remove temporary directory");
}

fn unzip_file(download_file_path: &Path, download_dir: &Path) {
    let tar_gz = std::fs::File::open(download_file_path).expect("Failed to open tar.gz file");
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive
        .unpack(download_dir)
        .expect("Failed to unpack tar.gz file");
}

fn validate_file(file_path: &Path, sha256_sum: &str) {
    let calculated_sha256 = sha256::try_digest(&file_path).unwrap();
    if calculated_sha256 != *sha256_sum {
        panic!("File {} has incorrect sha256 sum", file_path.display());
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
