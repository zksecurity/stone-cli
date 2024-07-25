use std::fs::{metadata, remove_file, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const CONFIG: &str = include_str!("configs/env.json");

#[derive(serde::Deserialize)]
struct Config {
    download_dir: String,
    url: String,
    file_names: Vec<String>,
    sha256_sums: Vec<String>,
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
    let download_file_name = Path::new(&config.url)
        .file_name()
        .expect("Failed to get the last path of the URL");
    let download_file_path = download_dir.join(download_file_name);
    download_from_url(&config.url, &download_file_path);
    unzip_file(&download_file_path, &download_dir);
    remove_file(&download_file_path).expect("Failed to remove tar file");
    validate_unpacked_files(&download_dir, &config.file_names, &config.sha256_sums);
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
    let download_dir = Path::new(env!("HOME")).join(".starknet-adapter-cli");
    let corelib_dir = Path::new(env!("HOME")).join(download_dir.join("corelib"));
    let url = "https://github.com/starkware-libs/cairo/releases/download/v2.6.3/release-x86_64-unknown-linux-musl.tar.gz";
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
