use std::path::PathBuf;
use thiserror::Error;

use crate::args::{StoneVersion, VerifyArgs};

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    CommandError(VerifierCommandError),
}

#[derive(Debug)]
pub struct VerifierCommandError {
    output: std::process::Output,
    stone_version: StoneVersion,
}

impl std::fmt::Display for VerifierCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "failed to run stone verifier with version: {:?}, status: {}, stderr: {}",
            self.stone_version,
            self.output.status,
            String::from_utf8(self.output.stderr.clone()).unwrap()
        )
    }
}

pub fn run_stone_verifier(args: VerifyArgs) -> Result<(), VerifierError> {
    println!("Running stone verifier...");

    run_verifier_from_command_line(
        &args.proof,
        args.annotation_file,
        args.extra_output_file,
        &args.stone_version,
    )?;

    println!("Verification successful!");
    Ok(())
}

fn run_verifier_from_command_line(
    in_file: &PathBuf,
    annotation_file: Option<PathBuf>,
    extra_output_file: Option<PathBuf>,
    stone_version: &StoneVersion,
) -> Result<(), VerifierError> {
    let verifier_run_path = match stone_version {
        StoneVersion::V5 => std::env::var("CPU_AIR_VERIFIER_V5").unwrap(),
        StoneVersion::V6 => std::env::var("CPU_AIR_VERIFIER_V6").unwrap(),
    };

    let mut command = std::process::Command::new(verifier_run_path);
    command.arg("--in_file").arg(in_file);

    if let Some(annotation_file) = annotation_file {
        command.arg("--annotation_file").arg(annotation_file);
    }

    if let Some(extra_output_file) = extra_output_file {
        command.arg("--extra_output_file").arg(extra_output_file);
    }

    let output = command.output()?;

    if !output.status.success() {
        return Err(VerifierError::CommandError(VerifierCommandError {
            output,
            stone_version: stone_version.clone(),
        }));
    }

    Ok(())
}
