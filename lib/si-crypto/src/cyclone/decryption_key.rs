use std::{io, path::Path};

use base64::{engine::general_purpose, Engine};
use si_hash::Hash;
use sodiumoxide::crypto::box_::{PublicKey as BoxPublicKey, SecretKey as BoxSecretKey};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

/// An error that can be returned when working with a [`CycloneDecryptionKey`].
#[remain::sorted]
#[derive(Error, Debug)]
pub enum CycloneDecryptionKeyError {
    /// When deserializing a crypted message from base64 encoding fails
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    /// When a message fails to be decrypted using the decryption key
    #[error("failed to decrypt encryption key from bytes")]
    DecryptionFailed,
    /// When a key fails to be parsed from bytes
    #[error("failed to load key from bytes")]
    KeyParse,
    /// When an error is return while reading from a key file
    #[error("failed to load key from file: {0}")]
    LoadKeyIO(#[source] io::Error),
}

/// A key that decrypts segements of a Cyclone function request message.
#[derive(Debug, Clone)]
pub struct CycloneDecryptionKey {
    secret_key: BoxSecretKey,
    public_key: BoxPublicKey,
    public_key_hash: Hash,
    public_key_hash_string: String,
}

impl CycloneDecryptionKey {
    /// Loads a [`CycloneDecryptionKey`] from a file path.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key file was not readable (i.e. incorrect permission and/or ownership)
    /// - A key file could not be successfuly parsed
    pub async fn load(
        decryption_key_path: impl AsRef<Path>,
    ) -> Result<Self, CycloneDecryptionKeyError> {
        trace!(
            decryption_key_path = %decryption_key_path.as_ref().display(),
            "loading cyclone decryption key from disk",
        );
        let mut file = File::open(decryption_key_path)
            .await
            .map_err(CycloneDecryptionKeyError::LoadKeyIO)?;
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        file.read_to_end(&mut buf)
            .await
            .map_err(CycloneDecryptionKeyError::LoadKeyIO)?;
        let secret_key =
            BoxSecretKey::from_slice(&buf).ok_or(CycloneDecryptionKeyError::KeyParse)?;

        let public_key = secret_key.public_key();

        let public_key_hash = Hash::new(public_key.as_ref());
        let public_key_hash_string = public_key_hash.to_string();

        Ok(Self {
            secret_key,
            public_key,
            public_key_hash,
            public_key_hash_string,
        })
    }

    /// Decrypts an encrypted message which is Base64 encoded.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - Base64 decoding fails
    /// - Message cannot be decrypted using this key
    pub fn decode_and_decrypt(
        &self,
        base64_encoded: impl AsRef<str>,
    ) -> Result<Vec<u8>, CycloneDecryptionKeyError> {
        let crypted = general_purpose::STANDARD_NO_PAD.decode(base64_encoded.as_ref())?;
        sodiumoxide::crypto::sealedbox::open(&crypted, &self.public_key, &self.secret_key)
            .map_err(|_| CycloneDecryptionKeyError::DecryptionFailed)
    }

    /// Returns a [`struct@Hash`] of the encryption key which would have encoded a message.
    pub fn encryption_key_hash(&self) -> &Hash {
        &self.public_key_hash
    }

    /// Returns a string representaion of the hash of the encryption key which would have encoded a
    /// message.
    pub fn encryption_key_hash_str(&self) -> &str {
        self.public_key_hash_string.as_str()
    }
}

impl From<BoxSecretKey> for CycloneDecryptionKey {
    fn from(value: BoxSecretKey) -> Self {
        let public_key = value.public_key();
        let secret_key = value;
        let public_key_hash = Hash::new(public_key.as_ref());
        let public_key_hash_string = public_key_hash.to_string();

        Self {
            secret_key,
            public_key,
            public_key_hash,
            public_key_hash_string,
        }
    }
}
