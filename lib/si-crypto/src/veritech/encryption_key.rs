use crate::VeritechCryptoConfig;
use std::{io, path::Path};

use base64::{Engine, engine::general_purpose};
use si_hash::Hash;
use sodiumoxide::crypto::box_::PublicKey;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

/// An error that can be returned when working with a [`VeritechEncryptionKey`].
#[remain::sorted]
#[derive(Debug, Error)]
pub enum VeritechEncryptionKeyError {
    /// When a base64 encoded key fails to be decoded.
    #[error("failed to decode base64 encoded key")]
    Base64Decode(#[source] base64::DecodeError),
    /// When a key cannot be made from the supplied config
    #[error(
        "key cannot be made from the supplied config, must supply either a base64 string or a filepath"
    )]
    FromConfig,
    /// When a key fails to be parsed from bytes
    #[error("failed to load key from bytes")]
    KeyParse,
    /// When an error is return while reading from a key file
    #[error("failed to load key from file: {0}")]
    LoadKeyIO(#[source] io::Error),
}

/// A key that encrypts segements of a Veritech function request message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VeritechEncryptionKey {
    public_key: PublicKey,
    key_hash: Hash,
}

impl VeritechEncryptionKey {
    /// Creates an instance of [`VeritechEncryptionKey`] based on the
    /// supplied configuration.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key file was not readable (i.e. incorrect permission and/or ownership)
    /// - A key file could not be successfuly parsed
    /// - A key string could not be successfully parsed
    /// - An invalid configuration was passed in
    pub async fn from_config(
        config: VeritechCryptoConfig,
    ) -> Result<Self, VeritechEncryptionKeyError> {
        match (config.encryption_key_file, config.encryption_key_base64) {
            (Some(path), None) => Self::load(path).await,
            (None, Some(b64_string)) => Self::decode(b64_string).await,
            _ => Err(VeritechEncryptionKeyError::FromConfig),
        }
    }

    /// Loads a [`VeritechEncryptionKey`] from a file path.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key file was not readable (i.e. incorrect permission and/or ownership)
    /// - A key file could not be successfuly parsed
    pub async fn load(
        encryption_key_path: impl AsRef<Path>,
    ) -> Result<Self, VeritechEncryptionKeyError> {
        trace!(
            encryption_key_path = %encryption_key_path.as_ref().display(),
            "loading veritech encryption key from disk",
        );
        let mut file = File::open(encryption_key_path)
            .await
            .map_err(VeritechEncryptionKeyError::LoadKeyIO)?;
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        file.read_to_end(&mut buf)
            .await
            .map_err(VeritechEncryptionKeyError::LoadKeyIO)?;
        let public_key = PublicKey::from_slice(&buf).ok_or(VeritechEncryptionKeyError::KeyParse)?;

        let key_hash = Hash::new(public_key.as_ref());

        Ok(Self {
            public_key,
            key_hash,
        })
    }

    /// Loads a [`VeritechEncryptionKey`] from a base64 encoded string.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key string could not be successfully parsed
    pub async fn decode(encryption_key_string: String) -> Result<Self, VeritechEncryptionKeyError> {
        trace!(
            "loading veritech encryption key from base64 string {}",
            encryption_key_string
        );
        let buf = general_purpose::STANDARD
            .decode(encryption_key_string)
            .map_err(VeritechEncryptionKeyError::Base64Decode)?;
        let public_key = PublicKey::from_slice(&buf).ok_or(VeritechEncryptionKeyError::KeyParse)?;

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

impl From<PublicKey> for VeritechEncryptionKey {
    fn from(value: PublicKey) -> Self {
        let key_hash = Hash::new(value.as_ref());

        Self {
            public_key: value,
            key_hash,
        }
    }
}
