use std::{io, path::Path};

use base64::{engine::general_purpose, Engine};
use sodiumoxide::crypto::box_::{PublicKey as BoxPublicKey, SecretKey as BoxSecretKey};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DecryptionKeyError {
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("failed to decrypt encryption key from bytes")]
    DecryptionFailed,
    #[error("encrypted secret not found")]
    EncryptedSecretNotFound,
    #[error("json pointer not found: {1} at {0}")]
    JSONPointerNotFound(serde_json::Value, String),
    #[error("failed to load key from bytes")]
    KeyParse,
    #[error("failed to load key from file: {0}")]
    LoadKeyIO(#[source] io::Error),
    #[error("json serialize/deseialize error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct DecryptionKey {
    secret_key: BoxSecretKey,
    public_key: BoxPublicKey,
}

impl DecryptionKey {
    pub async fn load(decryption_key_path: impl AsRef<Path>) -> Result<Self, DecryptionKeyError> {
        trace!(
            decryption_key_path = %decryption_key_path.as_ref().display(),
            "loading cyclone decryption key from disk",
        );
        let mut file = File::open(decryption_key_path)
            .await
            .map_err(DecryptionKeyError::LoadKeyIO)?;
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        file.read_to_end(&mut buf)
            .await
            .map_err(DecryptionKeyError::LoadKeyIO)?;
        let secret_key = BoxSecretKey::from_slice(&buf).ok_or(DecryptionKeyError::KeyParse)?;

        let public_key = secret_key.public_key();

        Ok(Self {
            secret_key,
            public_key,
        })
    }

    pub fn decode_and_decrypt(
        &self,
        base64_encoded: impl AsRef<str>,
    ) -> Result<Vec<u8>, DecryptionKeyError> {
        let crypted = general_purpose::STANDARD_NO_PAD.decode(base64_encoded.as_ref())?;
        sodiumoxide::crypto::sealedbox::open(&crypted, &self.public_key, &self.secret_key)
            .map_err(|_| DecryptionKeyError::DecryptionFailed)
    }
}

impl From<BoxSecretKey> for DecryptionKey {
    fn from(value: BoxSecretKey) -> Self {
        let public_key = value.public_key();
        let secret_key = value;

        Self {
            secret_key,
            public_key,
        }
    }
}
