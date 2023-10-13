//! This library provides the ability to encode (serialize) and decode (deserialize)
//! [CBOR](https://en.wikipedia.org/wiki/CBOR) objects.

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::BufReader;
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum CborError {
    #[error("ciborium deserialization error: {0}")]
    CiboriumDeserialization(#[from] ciborium::de::Error<std::io::Error>),
    #[error("ciborium serialization error: {0}")]
    CiboriumSerialization(#[from] ciborium::ser::Error<std::io::Error>),
}

type CborResult<T> = Result<T, CborError>;

/// Serialize the given value to CBOR.
pub fn encode<T>(value: &T) -> CborResult<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let mut encoded = Vec::new();
    ciborium::into_writer(value, &mut encoded)?;
    Ok(encoded)
}

/// Deserialize from CBOR to a provided type.
pub fn decode<T>(value: &[u8]) -> CborResult<T>
where
    T: DeserializeOwned,
{
    let reader = BufReader::new(value);
    Ok(ciborium::from_reader(reader)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string() {
        let original = "mybrainhurts";

        let bytes = encode(original).expect("could not encode");
        let round_trip: String = decode(&bytes).expect("could not decode");

        assert_eq!(original, round_trip.as_str());
    }
}
