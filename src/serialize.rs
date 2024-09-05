mod vec252;

use crate::args::{Network, SerializeArgs};
use anyhow::Result;
use cairo_felt::Felt252;
use cairo_proof_parser::parse;
use itertools::chain;
use std::fs;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;
use vec252::VecFelt252;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to interact with the file system")]
    IO(#[from] std::io::Error),
    #[error("Failed to parse proof file")]
    Parse(#[from] anyhow::Error),
    #[error("Failed to parse serde_json")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Annotation file is required for serializing proofs for Ethereum")]
    AnnotationFileNotSpecified,
    #[error("Extra output file is required for serializing proofs for Ethereum")]
    ExtraOutputFileNotSpecified,
}

pub fn serialize_proof(args: SerializeArgs) -> Result<(), Error> {
    let proof_file = args.proof.clone();
    if args.network == Network::ethereum {
        let proof_with_annotations_json =
            parse_bootloader_proof_file(&proof_file, args.annotation_file, args.extra_output_file)?;

        std::fs::write(args.output.clone(), proof_with_annotations_json).unwrap();
    } else if args.network == Network::starknet {
        let (config, public_input, unsent_commitment, witness) = parse_proof_file(&proof_file)?;

        let proof = chain!(
            config.into_iter(),
            public_input.into_iter(),
            unsent_commitment.into_iter(),
            witness.into_iter()
        );

        let calldata = chain!(proof, std::iter::once(Felt252::from(1)));

        let calldata_string = calldata
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        fs::write(args.output.clone(), calldata_string)?;
    }
    Ok(())
}

fn parse_proof_file(
    proof_file: &Path,
) -> Result<(VecFelt252, VecFelt252, VecFelt252, VecFelt252), Error> {
    let proof_file_content = std::fs::read_to_string(proof_file)?;
    let parsed = parse(proof_file_content).map_err(Error::Parse)?;
    Ok((
        serde_json::from_str(&parsed.config.to_string())?,
        serde_json::from_str(&parsed.public_input.to_string())?,
        serde_json::from_str(&parsed.unsent_commitment.to_string())?,
        serde_json::from_str(&parsed.witness.to_string())?,
    ))
}

fn parse_bootloader_proof_file(
    proof_file: &Path,
    annotation_file: Option<PathBuf>,
    extra_output_file: Option<PathBuf>,
) -> Result<String, Error> {
    let annotation_file = annotation_file.ok_or_else(|| Error::AnnotationFileNotSpecified)?;
    let extra_output_file = extra_output_file.ok_or_else(|| Error::ExtraOutputFileNotSpecified)?;

    // load proof file as json
    let proof_reader = std::fs::File::open(proof_file).unwrap();
    let proof: serde_json::Value = serde_json::from_reader(proof_reader).unwrap();

    // load annotation file and save the lines as an array property (annotations) in the proof json file
    let annotation_reader = std::fs::File::open(annotation_file).unwrap();
    let annotation_lines: Vec<String> = std::io::BufReader::new(annotation_reader)
        .lines()
        .map(|line| line.unwrap())
        .collect();
    let mut proof_with_annotations = proof.clone();
    proof_with_annotations["annotations"] = serde_json::json!(annotation_lines);

    // load extra annotation file and save the lines as an array property (extra_annotations) in the proof json file
    let extra_annotation_reader = std::fs::File::open(extra_output_file).unwrap();
    let extra_annotation_lines: Vec<String> = std::io::BufReader::new(extra_annotation_reader)
        .lines()
        .map(|line| line.unwrap())
        .collect();
    proof_with_annotations["extra_annotations"] = serde_json::json!(extra_annotation_lines);

    // format json and write to file
    let proof_with_annotations_json =
        serde_json::to_string_pretty(&proof_with_annotations).unwrap();
    Ok(proof_with_annotations_json)
}
