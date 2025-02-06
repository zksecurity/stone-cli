use bincode::enc::write::Writer;
use cairo1_run::FuncArg;
use cairo_vm::air_public_input::{PublicInput, PublicInputError};
use cairo_vm::Felt252;
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
    pub download_dir: String,
    pub file_names: Vec<String>,
    pub env_names: Vec<String>,
}

pub fn parse(config: &str) -> Config {
    serde_json::from_str(config).expect("Failed to parse config file")
}

pub fn set_env_vars(config: &Config) {
    let download_dir = Path::new(env!("HOME"))
        .join(&config.download_dir)
        .join("executables");
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

/// Copied from `cairo-vm`
#[derive(Debug, Clone, Default)]
pub struct FuncArgs(pub Vec<FuncArg>);

/// Processes an iterator of format [s1, s2,.., sn, "]", ...], stopping at the first "]" string
/// and returning the array [f1, f2,.., fn] where fi = Felt::from_dec_str(si)
pub fn process_array<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Result<FuncArg, String> {
    let mut array = vec![];
    for value in iter {
        match value {
            "]" => break,
            _ => array.push(
                Felt252::from_dec_str(value)
                    .map_err(|_| format!("\"{}\" is not a valid felt", value))?,
            ),
        }
    }
    Ok(FuncArg::Array(array))
}

/// Parses a string of ascii whitespace separated values, containing either numbers or series of numbers wrapped in brackets
/// Returns an array of felts and felt arrays
pub fn process_args(value: &str) -> Result<FuncArgs, String> {
    let mut args = Vec::new();
    // Split input string into numbers and array delimiters
    let mut input = value.split_ascii_whitespace().flat_map(|mut x| {
        // We don't have a way to split and keep the separate delimiters so we do it manually
        let mut res = vec![];
        if let Some(val) = x.strip_prefix('[') {
            res.push("[");
            x = val;
        }
        if let Some(val) = x.strip_suffix(']') {
            if !val.is_empty() {
                res.push(val)
            }
            res.push("]")
        } else if !x.is_empty() {
            res.push(x)
        }
        res
    });
    // Process iterator of numbers & array delimiters
    while let Some(value) = input.next() {
        match value {
            "[" => args.push(process_array(&mut input)?),
            _ => args.push(FuncArg::Single(
                Felt252::from_dec_str(value)
                    .map_err(|_| format!("\"{}\" is not a valid felt", value))?,
            )),
        }
    }
    Ok(FuncArgs(args))
}
