use std::path::Path;

const CONFIG: &[u8] = include_bytes!("./config.json");

#[derive(serde::Deserialize)]
struct Config {
    download_dir: String,
    url: String,
    file_names: Vec<String>,
    sha256_sums: Vec<String>,
}

fn main() {
    let config: Config = serde_json::from_slice(CONFIG).expect("Failed to parse config file");
    download_executables(&config);
    give_execute_permissions(&config);
    clone_corelib_repo(&config);
}

fn download_executables(config: &Config) {
    let download_dir = format!("{}/{}", env!("HOME"), config.download_dir);
    if !std::path::Path::new(&download_dir).exists() {
        std::fs::create_dir_all(&download_dir).expect("Failed to create download directory");
    }
    let all_files_exist = config.file_names.iter().all(|filename| {
        let file_path = format!("{}/{}", download_dir, filename);
        std::path::Path::new(&file_path).exists()
    });
    if all_files_exist {
        return;
    }
    let tar_file_path = format!("{}/starknet-adapter-cli-linux-x86-64.tar.gz", download_dir);
    let response = reqwest::blocking::get(&config.url).expect("Failed to download file");
    let mut file = std::fs::File::create(&tar_file_path).expect("Failed to create file");
    std::io::copy(
        &mut response.bytes().expect("Failed to read response").as_ref(),
        &mut file,
    )
    .expect("Failed to write to file");
    let tar_gz = std::fs::File::open(&tar_file_path).expect("Failed to open tar.gz file");
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive
        .unpack(&download_dir)
        .expect("Failed to unpack tar.gz file");

    // Validate the unpacked files
    for (filename, sha256_sum) in config.file_names.iter().zip(config.sha256_sums.iter()) {
        let file_path = format!("{}/{}", download_dir, filename);
        if !std::path::Path::new(&file_path).exists() {
            panic!("Expected file {} does not exist after unpacking", file_path);
        }

        let calculated_sha256 = sha256::try_digest(Path::new(&file_path)).unwrap();
        if calculated_sha256 != *sha256_sum {
            panic!("File {} has incorrect sha256 sum", file_path);
        }
    }
}

fn give_execute_permissions(config: &Config) {
    let download_dir = format!("{}/{}", env!("HOME"), config.download_dir);
    for filename in config.file_names.iter() {
        let file_path = format!("{}/{}", download_dir, filename);
        if !std::path::Path::new(&file_path).exists() {
            panic!("File {} does not exist", file_path);
        }
        if !std::process::Command::new("chmod")
            .args(&["+x", &file_path])
            .status()
            .expect("Failed to change file permissions")
            .success()
        {
            panic!("Failed to give execute permissions to {}", file_path);
        }
    }
}

fn clone_corelib_repo(config: &Config) {
    let download_dir = format!("{}/{}", env!("HOME"), config.download_dir);
    let repo_path = format!("{}/../corelib", download_dir);
    if !std::path::Path::new(&repo_path).exists() {
        if !std::process::Command::new("git")
            .args(&[
                "clone",
                "--depth=1",
                "-b",
                "v2.6.4",
                "https://github.com/starkware-libs/cairo.git",
            ])
            .status()
            .expect("Failed to clone the repository")
            .success()
        {
            panic!("Failed to clone the repository. Please check your internet connection and try again.");
        }

        if !std::process::Command::new("cp")
            .args(&["-r", "./cairo/corelib", &repo_path])
            .status()
            .expect("Failed to copy corelib directory")
            .success()
        {
            panic!("Failed to copy corelib directory. Please check if the directory exists in the current directory.");
        }

        if !std::process::Command::new("rm")
            .args(&["-rf", "cairo/"])
            .status()
            .expect("Failed to remove the repository")
            .success()
        {
            panic!("Failed to remove the repository. Please check your permissions and try again.");
        }
    }
}
