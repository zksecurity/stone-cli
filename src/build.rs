use reqwest;

const URL: &str = "https://github.com/zksecurity/starknet-adapter/releases/download/v0.1.0-alpha/starknet-adapter-cli-linux-x86-64.tar.gz";
const FILENAMES: [&str; 3] = ["cairo1-run", "cpu_air_prover", "cpu_air_verifier"];
const DOWNLOAD_DIR: &str = concat!(env!("HOME"), "/.starknet-adapter-cli/v0.1.0");

pub fn setup() {
    download_executables();
    give_execute_permissions();
    clone_corelib_repo();
    set_env_vars();
}

fn download_executables() {
    if !std::path::Path::new(&DOWNLOAD_DIR).exists() {
        std::fs::create_dir_all(&DOWNLOAD_DIR).expect("Failed to create download directory");
    }

    let all_files_exist = FILENAMES.iter().all(|filename| {
        let file_path = format!("{}/{}", DOWNLOAD_DIR, filename);
        std::path::Path::new(&file_path).exists()
    });
    if all_files_exist {
        return;
    }

    let tar_file_path = format!("{}/starknet-adapter-cli-linux-x86-64.tar.gz", DOWNLOAD_DIR);
    let response = reqwest::blocking::get(URL).expect("Failed to download file");
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
        .unpack(DOWNLOAD_DIR)
        .expect("Failed to unpack tar.gz file");

    // Validate the unpacked files
    for filename in FILENAMES.iter() {
        let file_path = format!("{}/{}", DOWNLOAD_DIR, filename);
        if !std::path::Path::new(&file_path).exists() {
            panic!("Expected file {} does not exist after unpacking", file_path);
        }
    }
}

fn give_execute_permissions() {
    for filename in FILENAMES.iter() {
        let file_path = format!("{}/{}", DOWNLOAD_DIR, filename);
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

fn clone_corelib_repo() {
    let repo_path = format!("{}/../corelib", DOWNLOAD_DIR);
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

fn set_env_vars() {
    for filename in FILENAMES.iter() {
        let full_path = format!("{}/{}", DOWNLOAD_DIR, filename);
        std::env::set_var(filename.replace("-", "_").to_uppercase(), full_path.clone());
    }
}
