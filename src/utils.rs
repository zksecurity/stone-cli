use bincode::enc::write::Writer;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use serde::Serialize;

#[macro_export]
macro_rules! define_enum {
    ($name:ident, $($variant:ident => $str:expr),+ $(,)?) => {
        #[derive(Serialize, Deserialize, Debug, Clone, clap::ValueEnum, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn to_str(self) -> &'static str {
                match self {
                    $($name::$variant => $str),+
                }
            }
        }
    };
}

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
    #[allow(dead_code)]
    url: String,
    file_names: Vec<String>,
    #[allow(dead_code)]
    sha256_sums: Vec<String>,
    env_names: Vec<String>,
}

pub fn parse(config: &str) -> Config {
    serde_json::from_str(config).expect("Failed to parse config file")
}

pub fn set_env_vars(config: &Config) {
    let download_dir = Path::new(env!("HOME")).join(&config.download_dir);
    for (env_name, filename) in config.env_names.iter().zip(config.file_names.iter()) {
        let full_path = download_dir.join(filename);
        unsafe {
            std::env::set_var(env_name, full_path.clone());
        }
    }
}

pub struct FileWriter {
    buf_writer: BufWriter<std::fs::File>,
    bytes_written: usize,
}

impl Writer for FileWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), bincode::error::EncodeError> {
        self.buf_writer
            .write_all(bytes)
            .map_err(|e| bincode::error::EncodeError::Io {
                inner: e,
                index: self.bytes_written,
            })?;

        self.bytes_written += bytes.len();

        Ok(())
    }
}

impl FileWriter {
    pub fn new(buf_writer: BufWriter<std::fs::File>) -> Self {
        Self {
            buf_writer,
            bytes_written: 0,
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.buf_writer.flush()
    }
}
