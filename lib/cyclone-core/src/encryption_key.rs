use std::{io, path::Path};

use base64::{engine::general_purpose, Engine};
use sodiumoxide::crypto::box_::PublicKey;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum EncryptionKeyError {
    #[error("failed to load key from bytes")]
    KeyParse,
    #[error("failed to load key from file: {0}")]
    LoadKeyIO(#[source] io::Error),
}

pub type EncryptionKeyResult<T> = Result<T, EncryptionKeyError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncryptionKey {
    public_key: PublicKey,
}

impl EncryptionKey {
    pub async fn load(encryption_key_path: impl AsRef<Path>) -> EncryptionKeyResult<Self> {
        trace!(
            encryption_key_path = %encryption_key_path.as_ref().display(),
            "loading cyclone encryption key from disk",
        );
        let mut file = File::open(encryption_key_path)
            .await
            .map_err(EncryptionKeyError::LoadKeyIO)?;
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        file.read_to_end(&mut buf)
            .await
            .map_err(EncryptionKeyError::LoadKeyIO)?;
        let public_key = PublicKey::from_slice(&buf).ok_or(EncryptionKeyError::KeyParse)?;

        Ok(Self { public_key })
    }

    pub fn encrypt_and_encode(&self, message: impl AsRef<[u8]>) -> String {
        let crypted = sodiumoxide::crypto::sealedbox::seal(message.as_ref(), &self.public_key);
        general_purpose::STANDARD_NO_PAD.encode(crypted)
    }
}

impl From<PublicKey> for EncryptionKey {
    fn from(value: PublicKey) -> Self {
        Self { public_key: value }
    }
}
