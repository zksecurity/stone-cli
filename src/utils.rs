use bincode::enc::write::Writer;
use cairo_vm::air_public_input::{PublicInput, PublicInputError};
use serde::Serialize;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

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
    linux_x86_64_url: String,
    #[allow(dead_code)]
    macos_aarch64_url: String,
    file_names: Vec<String>,
    #[allow(dead_code)]
    sha256_sums_linux_x86_64: Vec<String>,
    #[allow(dead_code)]
    sha256_sums_macos_aarch64: Vec<String>,
    #[allow(dead_code)]
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

// `CairoRunner.get_air_public_input()` returns a `PublicInput` object.
// This function converts it to a JSON string and formats the "public_memory" array
// by prefixing each value with "0x" if it doesn't already start with "0x".
pub fn get_formatted_air_public_input(
    air_public_input: &PublicInput,
) -> Result<String, PublicInputError> {
    let mut air_public_input: serde_json::Value =
        serde_json::from_str(&air_public_input.serialize_json()?)?;

    // Check if "public_memory" exists and is an array
    if let Some(public_memory) = air_public_input
        .get_mut("public_memory")
        .and_then(|v| v.as_array_mut())
    {
        // Iterate through each item in the "public_memory" array
        for item in public_memory {
            // Check if the item has a "value" field
            if let Some(value) = item.get_mut("value").and_then(|v| v.as_str()) {
                // Prepend "0x" to the value if it doesn't already start with "0x"
                if !value.starts_with("0x") {
                    let new_value = format!("0x{}", value);
                    item["value"] = serde_json::Value::String(new_value);
                }
            }
        }
    }
    // Convert the modified JSON back to a string
    let air_public_input_str = serde_json::to_string(&air_public_input)?;

    Ok(air_public_input_str)
}
