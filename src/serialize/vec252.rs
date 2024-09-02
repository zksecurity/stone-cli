// This code is adapted from the following GitHub repository:
// https://github.com/HerodotusDev/integrity/blob/main/runner/src/vec252.rs

use std::{ops::Deref, str::FromStr};

use cairo_felt::Felt252;
use num_bigint;
use serde::{de::Visitor, Deserialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VecFelt252Error {
    #[error("failed to parse number: {0}")]
    NumberParseError(#[from] std::num::ParseIntError),
    #[error("failed to parse bigint: {0}")]
    BigIntParseError(#[from] num_bigint::ParseBigIntError),
    #[error("number out of range")]
    NumberOutOfRange,
    #[error("invalid type")]
    InvalidType,
}

/// `VecFelt252` is a wrapper around a vector of `Arg`.
///
/// It provides convenience methods for working with a vector of `Arg` and implements
/// `Deref` to allow it to be treated like a vector of `Arg`.
#[derive(Debug, Clone)]
pub struct VecFelt252(Vec<Felt252>);

impl VecFelt252 {
    /// Creates a new `VecFelt252` from a vector of `Arg`.
    ///
    /// # Arguments
    ///
    /// * `args` - A vector of `Arg`.
    ///
    /// # Returns
    ///
    /// * `VecFelt252` - A new `VecFelt252` instance.
    #[must_use]
    pub fn new(args: Vec<Felt252>) -> Self {
        Self(args)
    }
}

impl IntoIterator for VecFelt252 {
    type Item = Felt252;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for VecFelt252 {
    type Target = Vec<Felt252>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<VecFelt252> for Vec<Felt252> {
    fn from(args: VecFelt252) -> Self {
        args.0
    }
}

impl From<Vec<Felt252>> for VecFelt252 {
    fn from(args: Vec<Felt252>) -> Self {
        Self(args)
    }
}

impl VecFelt252 {
    fn visit_seq_helper(seq: &[Value]) -> Result<Self, VecFelt252Error> {
        let iterator = seq.iter();
        let mut args = Vec::new();

        for arg in iterator {
            match arg {
                Value::Number(n) => {
                    let n = n.as_u64().ok_or(VecFelt252Error::NumberOutOfRange)?;
                    args.push(Felt252::from(n));
                }
                Value::String(n) => {
                    let n = num_bigint::BigUint::from_str(n)?;
                    args.push(Felt252::from_bytes_be(&n.to_bytes_be()));
                }
                Value::Array(a) => {
                    args.push(Felt252::from(a.len()));
                    let result = Self::visit_seq_helper(a)?;
                    args.extend(result.0);
                }
                _ => return Err(VecFelt252Error::InvalidType),
            }
        }

        Ok(Self::new(args))
    }
}

impl<'de> Visitor<'de> for VecFelt252 {
    type Value = VecFelt252;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of arguments")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut args = Vec::new();
        while let Some(arg) = seq.next_element()? {
            args.push(arg);
        }

        Self::visit_seq_helper(&args).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for VecFelt252 {
    fn deserialize<D>(deserializer: D) -> Result<VecFelt252, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(VecFelt252(Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_vecfelt252() {
        let json_data = json!([1, 2, 3, 4, 5]);
        let vecfelt: VecFelt252 = serde_json::from_value(json_data).unwrap();
        assert_eq!(vecfelt.0.len(), 5);
        assert_eq!(vecfelt.0[0], Felt252::from(1));
        assert_eq!(vecfelt.0[1], Felt252::from(2));
        assert_eq!(vecfelt.0[2], Felt252::from(3));
        assert_eq!(vecfelt.0[3], Felt252::from(4));
        assert_eq!(vecfelt.0[4], Felt252::from(5));
    }

    #[test]
    fn test_deserialize_vecfelt252_with_strings() {
        let json_data = json!(["1", "2", "3", "4", "5"]);
        let vecfelt: VecFelt252 = serde_json::from_value(json_data).unwrap();
        assert_eq!(vecfelt.0.len(), 5);
        assert_eq!(vecfelt.0[0], Felt252::from(1));
        assert_eq!(vecfelt.0[1], Felt252::from(2));
        assert_eq!(vecfelt.0[2], Felt252::from(3));
        assert_eq!(vecfelt.0[3], Felt252::from(4));
        assert_eq!(vecfelt.0[4], Felt252::from(5));
    }

    #[test]
    fn test_deserialize_vecfelt252_with_nested_arrays() {
        let json_data = json!([1, [2, 3], 4, [5, 6, 7]]);
        let vecfelt: VecFelt252 = serde_json::from_value(json_data).unwrap();
        assert_eq!(vecfelt.0.len(), 9);
        assert_eq!(vecfelt.0[0], Felt252::from(1));
        assert_eq!(vecfelt.0[1], Felt252::from(2)); // length of nested array [2, 3]
        assert_eq!(vecfelt.0[2], Felt252::from(2));
        assert_eq!(vecfelt.0[3], Felt252::from(3));
        assert_eq!(vecfelt.0[4], Felt252::from(4));
        assert_eq!(vecfelt.0[5], Felt252::from(3)); // length of nested array [5, 6, 7]
        assert_eq!(vecfelt.0[6], Felt252::from(5));
        assert_eq!(vecfelt.0[7], Felt252::from(6));
        assert_eq!(vecfelt.0[8], Felt252::from(7));
    }

    #[test]
    fn test_deserialize_vecfelt252_invalid_type() {
        let json_data = json!([1, "invalid", 3]);
        let result: Result<VecFelt252, _> = serde_json::from_value(json_data);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "failed to parse bigint: invalid digit found in string"
        );

        let json_data = json!([1, -1, 3]);
        let result: Result<VecFelt252, _> = serde_json::from_value(json_data);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "number out of range");

        let json_data = json!([1, true, 3]);
        let result: Result<VecFelt252, _> = serde_json::from_value(json_data);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().to_string(), "invalid type");
    }
}
