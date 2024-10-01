mod vec252;

use crate::args::{LayoutName, SerializationType};
use crate::args::{Network, SerializeArgs};
use anyhow::Result;
use cairo_felt::Felt252;
use itertools::chain;
use itertools::Itertools;
use num_traits::Num;
use starknet_crypto::Felt;
use std::fs::write;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use swiftness_air::layout::*;
use swiftness_fri::{CONST_STATE, VAR_STATE, WITNESS};
use swiftness_proof_parser::{parse, parse_as_exprs, Expr, ParseStarkProof};
use swiftness_stark::stark;
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
    #[error("Failed to verify proof: {0}")]
    Verify(#[from] stark::Error),
    #[error("Serialization is not supported for the {0} layout")]
    UnsupportedLayout(LayoutName),
    #[error("Serialization type is not specified")]
    SerializationTypeNotSpecified,
}

pub fn serialize_proof(args: SerializeArgs) -> Result<(), Error> {
    let proof_file = args.proof.clone();
    match args.network {
        Network::ethereum => {
            let proof_with_annotations_json = parse_bootloader_proof_file(
                &proof_file,
                args.annotation_file,
                args.extra_output_file,
            )?;

            std::fs::write(args.output.clone().unwrap(), proof_with_annotations_json).unwrap();
        }
        Network::starknet => match args.serialization_type {
            Some(SerializationType::monolith) => {
                let output = args.output.clone().unwrap();
                let input = std::fs::read_to_string(proof_file.clone())?;
                let stark_proof: ParseStarkProof = parse_as_exprs(input)?;
                let config: VecFelt252 =
                    serde_json::from_str(&stark_proof.config.to_string()).unwrap();
                let public_input: VecFelt252 =
                    serde_json::from_str(&stark_proof.public_input.to_string()).unwrap();
                let unsent_commitment: VecFelt252 =
                    serde_json::from_str(&stark_proof.unsent_commitment.to_string()).unwrap();
                let witness: VecFelt252 =
                    serde_json::from_str(&stark_proof.witness.to_string()).unwrap();

                let proof = chain!(
                    config.into_iter(),
                    public_input.into_iter(),
                    unsent_commitment.into_iter(),
                    witness.into_iter()
                );

                let calldata_string = proof
                    .into_iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");

                std::fs::write(output, calldata_string)?;
            }
            Some(SerializationType::split) => {
                let output_dir = args.output_dir.clone().unwrap();
                let layout = args.layout.unwrap();
                let input = std::fs::read_to_string(proof_file.clone())?;
                let stark_proof = parse(input.clone())?;
                let security_bits = stark_proof.config.security_bits();

                match layout {
                    LayoutName::dex => {
                        stark_proof.verify::<dex::Layout>(security_bits)?;
                    }
                    LayoutName::recursive => {
                        stark_proof.verify::<recursive::Layout>(security_bits)?;
                    }
                    LayoutName::recursive_with_poseidon => {
                        stark_proof.verify::<recursive_with_poseidon::Layout>(security_bits)?;
                    }
                    LayoutName::small => {
                        stark_proof.verify::<small::Layout>(security_bits)?;
                    }
                    LayoutName::starknet => {
                        stark_proof.verify::<starknet::Layout>(security_bits)?;
                    }
                    LayoutName::starknet_with_keccak => {
                        stark_proof.verify::<starknet_with_keccak::Layout>(security_bits)?;
                    }
                    layout @ (LayoutName::plain
                    | LayoutName::recursive_large_output
                    | LayoutName::all_solidity
                    | LayoutName::all_cairo
                    | LayoutName::dynamic) => {
                        return Err(Error::UnsupportedLayout(layout));
                    }
                }

                let (const_state, mut var_state, mut witness) =
                    unsafe { (CONST_STATE.clone(), VAR_STATE.clone(), WITNESS.clone()) };
                let cairo_version = Felt252::from(0);
                let initial = serialize(input, cairo_version)?
                    .split_whitespace()
                    .map(|s| Felt::from_dec_str(s).unwrap().to_hex_string())
                    .join(" ");

                let final_ = format!(
                    "{} {} {}",
                    const_state,
                    var_state.pop().unwrap(),
                    witness.pop().unwrap()
                );

                std::fs::create_dir_all(&output_dir)?;

                write(output_dir.join("initial"), initial)?;
                write(output_dir.join("final"), final_)?;

                for (i, (v, w)) in var_state.iter().zip(witness.iter()).enumerate() {
                    write(
                        output_dir.join(format!("step{}", i + 1)),
                        format!("{} {} {}", const_state, v, w),
                    )?;
                }
            }
            None => return Err(Error::SerializationTypeNotSpecified),
        },
    }
    Ok(())
}

fn serialize(input: String, cairo_version: Felt252) -> Result<String, Error> {
    let mut parsed = parse_as_exprs(input)?;

    let config: VecFelt252 = serde_json::from_str(&parsed.config.to_string()).unwrap();
    let public_input: VecFelt252 = serde_json::from_str(&parsed.public_input.to_string()).unwrap();
    let unsent_commitment: VecFelt252 =
        serde_json::from_str(&parsed.unsent_commitment.to_string()).unwrap();

    let fri_witness = match parsed.witness.0.pop().unwrap() {
        Expr::Array(witness) => witness,
        _ => panic!("Expected witness to be an array"),
    };
    let mut fri_layers = vec![];
    let mut i = Felt252::from(0);
    let mut reach_0_count = 2;
    fri_witness.into_iter().for_each(|elem| {
        let elem = match elem {
            Expr::Value(s) => <Felt252 as Num>::from_str_radix(s.as_str(), 10).unwrap(),
            _ => panic!("Expected value"),
        };
        if i == Felt252::from(0) {
            if reach_0_count == 2 {
                fri_layers.push(vec![]);
                reach_0_count = 0;
            }
            reach_0_count += 1;

            i = elem.clone();
        } else {
            i -= Felt252::from(1);
        }
        fri_layers.last_mut().unwrap().push(elem);
    });

    parsed.witness.0.push(Expr::Array(vec![]));
    let witness: VecFelt252 = serde_json::from_str(&parsed.witness.to_string()).unwrap();

    let proof = chain!(
        config.into_iter(),
        public_input.into_iter(),
        unsent_commitment.into_iter(),
        witness.into_iter()
    );

    let calldata = chain!(proof, vec![cairo_version].into_iter());

    let calldata_string = calldata
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(" ");

    Ok(calldata_string)
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
