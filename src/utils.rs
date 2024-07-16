use std::fs::File;
use std::path::Path;

use serde::Serialize;

pub fn write_json_to_file<T: Serialize, P: AsRef<Path>>(
    obj: T,
    path: P,
) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    serde_json::to_writer(&mut file, &obj)?;
    Ok(())
}

pub fn cleanup_tmp_files(tmp_dir: &tempfile::TempDir) {
    if let Err(e) = std::fs::remove_dir_all(tmp_dir) {
        eprintln!("Failed to clean up temporary directory: {}", e);
    }
}

#[derive(serde::Deserialize)]
pub struct Config {
    download_dir: String,
    url: String,
    file_names: Vec<String>,
    sha256_sums: Vec<String>,
}

pub fn set_env_vars(config: &[u8]) {
    let config: Config = serde_json::from_slice(config).expect("Failed to parse config file");
    let download_dir = format!("{}/{}", env!("HOME"), config.download_dir);
    for filename in config.file_names.iter() {
        let full_path = format!("{}/{}", download_dir, filename);
        let filename = filename.split('.').next().unwrap().replace("-", "_");
        std::env::set_var(filename.to_uppercase(), full_path.clone());
    }
}
