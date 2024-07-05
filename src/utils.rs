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

// TODO: Add more specific handling of errors
pub fn handle_error(err: anyhow::Error) {
    eprintln!("Error: {}", err);
    std::process::exit(1);
}
