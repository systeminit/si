//! Serialization helpers for sodiumoxide types

use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Deserializer, Serializer};
use sodiumoxide::crypto::box_::PublicKey;


/// Encode bytes to base64 string
fn base64_encode_bytes(bytes: &[u8]) -> String {
    general_purpose::STANDARD_NO_PAD.encode(bytes)
}

/// Serialize a PublicKey to base64 string
pub fn serialize_public_key<S>(public_key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = base64_encode_bytes(public_key.as_ref());
    serializer.serialize_str(&s)
}

/// Deserialize a PublicKey from base64 string
pub fn deserialize_public_key<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = general_purpose::STANDARD_NO_PAD
        .decode(s)
        .map_err(serde::de::Error::custom)?;

    PublicKey::from_slice(&bytes)
        .ok_or_else(|| serde::de::Error::custom("cannot deserialize public key"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestStruct {
        #[serde(
            serialize_with = "serialize_public_key",
            deserialize_with = "deserialize_public_key"
        )]
        key: PublicKey,
    }

    #[test]
    fn test_public_key_serde() {
        crate::init().expect("Failed to initialize sodiumoxide");
        
        let (public_key, _) = crate::key_pair::generate_keypair();
        let test_struct = TestStruct { key: public_key };
        
        let json = serde_json::to_string(&test_struct).expect("Failed to serialize");
        let deserialized: TestStruct = serde_json::from_str(&json).expect("Failed to deserialize");
        
        assert_eq!(test_struct.key.as_ref(), deserialized.key.as_ref());
    }
}