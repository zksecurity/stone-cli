mod vec252;

use crate::args::SerializeArgs;
use anyhow::Result;
use cairo_felt::Felt252;
use cairo_proof_parser::parse;
use itertools::chain;
use std::fs;
use std::path::Path;
use vec252::VecFelt252;

pub fn serialize_proof(args: &SerializeArgs) -> Result<()> {
    let proof_file = args.proof.clone();
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
    Ok(())
}

fn parse_proof_file(proof_file: &Path) -> Result<(VecFelt252, VecFelt252, VecFelt252, VecFelt252)> {
    let proof_file_content = std::fs::read_to_string(proof_file)?;
    let parsed = parse(proof_file_content)?;
    Ok((
        serde_json::from_str(&parsed.config.to_string())?,
        serde_json::from_str(&parsed.public_input.to_string())?,
        serde_json::from_str(&parsed.unsent_commitment.to_string())?,
        serde_json::from_str(&parsed.witness.to_string())?,
    ))
}
