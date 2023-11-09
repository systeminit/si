use std::{io, path::Path};

use base64::{engine::general_purpose, Engine};
use si_hash::Hash;
use sodiumoxide::crypto::box_::PublicKey;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

/// An error that can be returned when working with a [`CycloneEncryptionKey`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum CycloneEncryptionKeyError {
    /// When a key fails to be parsed from bytes
    #[error("failed to load key from bytes")]
    KeyParse,
    /// When an error is return while reading from a key file
    #[error("failed to load key from file: {0}")]
    LoadKeyIO(#[source] io::Error),
}

/// A key that encrypts segements of a Cyclone function request message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CycloneEncryptionKey {
    public_key: PublicKey,
    key_hash: Hash,
}

impl CycloneEncryptionKey {
    /// Loads a [`CycloneEncryptionKey`] from a file path.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key file was not readable (i.e. incorrect permission and/or ownership)
    /// - A key file could not be successfuly parsed
    pub async fn load(
        encryption_key_path: impl AsRef<Path>,
    ) -> Result<Self, CycloneEncryptionKeyError> {
        trace!(
            encryption_key_path = %encryption_key_path.as_ref().display(),
            "loading cyclone encryption key from disk",
        );
        let mut file = File::open(encryption_key_path)
            .await
            .map_err(CycloneEncryptionKeyError::LoadKeyIO)?;
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        file.read_to_end(&mut buf)
            .await
            .map_err(CycloneEncryptionKeyError::LoadKeyIO)?;
        let public_key = PublicKey::from_slice(&buf).ok_or(CycloneEncryptionKeyError::KeyParse)?;

        let key_hash = Hash::new(public_key.as_ref());

        Ok(Self {
            public_key,
            key_hash,
        })
    }

    /// Encrypts an message and encodes it as a Base64 string.
    pub fn encrypt_and_encode(&self, message: impl AsRef<[u8]>) -> String {
        let crypted = sodiumoxide::crypto::sealedbox::seal(message.as_ref(), &self.public_key);
        general_purpose::STANDARD_NO_PAD.encode(crypted)
    }

    /// Returns a [`Hash`] of this key.
    pub fn key_hash(&self) -> &Hash {
        &self.key_hash
    }
}

impl From<PublicKey> for CycloneEncryptionKey {
    fn from(value: PublicKey) -> Self {
        let key_hash = Hash::new(value.as_ref());

        Self {
            public_key: value,
            key_hash,
        }
    }
}
