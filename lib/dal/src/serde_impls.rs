use base64::{engine::general_purpose, Engine};

fn base64_encode_bytes(bytes: &[u8]) -> String {
    general_purpose::STANDARD_NO_PAD.encode(bytes)
}

pub mod base64_bytes_serde {
    use base64::{engine::general_purpose, Engine};
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::base64_encode_bytes;

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = base64_encode_bytes(bytes);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let buffer = general_purpose::STANDARD_NO_PAD
            .decode(s)
            .map_err(serde::de::Error::custom)?;
        Ok(buffer)
    }
}

pub mod nonce_serde {
    use serde::{self, Deserializer, Serializer};
    use si_crypto::SymmetricNonce;

    use super::base64_bytes_serde;

    pub fn serialize<S>(nonce: &SymmetricNonce, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        base64_bytes_serde::serialize(nonce.as_ref(), serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SymmetricNonce, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nonce = SymmetricNonce::from_slice(&base64_bytes_serde::deserialize(deserializer)?)
            .ok_or(serde::de::Error::custom(
                "length of bytes is invalid for nonce value",
            ))?;

        Ok(nonce)
    }
}
