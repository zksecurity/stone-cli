use std::path::PathBuf;
use stone_prover_sdk::error::VerifierError;

use crate::args::VerifyArgs;

pub fn run_stone_verifier(args: VerifyArgs) -> Result<(), VerifierError> {
    println!("Running stone verifier...");

    run_verifier_from_command_line(&args.proof, args.annotation_file, args.extra_output_file)?;

    println!("Verification successful!");
    Ok(())
}

fn run_verifier_from_command_line(
    in_file: &PathBuf,
    annotation_file: Option<PathBuf>,
    extra_output_file: Option<PathBuf>,
) -> Result<(), VerifierError> {
    let verifier_run_path = std::env::var("CPU_AIR_VERIFIER").unwrap();

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
        return Err(VerifierError::CommandError(output));
    }

    Ok(())
}
