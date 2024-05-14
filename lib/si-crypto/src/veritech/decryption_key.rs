use crate::VeritechCryptoConfig;
use std::{io, path::Path};

use base64::{engine::general_purpose, Engine};
use si_hash::Hash;
use sodiumoxide::crypto::box_::{PublicKey as BoxPublicKey, SecretKey as BoxSecretKey};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

/// An error that can be returned when working with a [`VeritechDecryptionKey`].
#[remain::sorted]
#[derive(Error, Debug)]
pub enum VeritechDecryptionKeyError {
    /// When deserializing a crypted message from base64 encoding fails
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    /// When a message fails to be decrypted using the decryption key
    #[error("failed to decrypt encryption key from bytes")]
    DecryptionFailed,
    /// When a key cannot be made from the supplied config
    #[error("key cannot be made from the supplied config, must supply either a base64 string or a filepath")]
    FromConfig,
    /// When a key fails to be parsed from bytes
    #[error("failed to load key from bytes")]
    KeyParse,
    /// When an error is return while reading from a key file
    #[error("failed to load key from file: {0}")]
    LoadKeyIO(#[source] io::Error),
}

/// A key that decrypts segements of a Veritech function request message.
#[derive(Debug, Clone)]
pub struct VeritechDecryptionKey {
    secret_key: BoxSecretKey,
    public_key: BoxPublicKey,
    public_key_hash: Hash,
    public_key_hash_string: String,
}

impl VeritechDecryptionKey {
    /// Creates an instance of [`VeritechDecryptionKey`] based on the
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
    ) -> Result<Self, VeritechDecryptionKeyError> {
        match (config.decryption_key_file, config.decryption_key_base64) {
            (Some(path), None) => Self::load(path).await,
            (None, Some(b64_string)) => Self::decode(b64_string).await,
            _ => Err(VeritechDecryptionKeyError::FromConfig),
        }
    }
    /// Loads a [`VeritechDecryptionKey`] from a file path.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key file was not readable (i.e. incorrect permission and/or ownership)
    /// - A key file could not be successfuly parsed
    pub async fn load(
        decryption_key_path: impl AsRef<Path>,
    ) -> Result<Self, VeritechDecryptionKeyError> {
        trace!(
            decryption_key_path = %decryption_key_path.as_ref().display(),
            "loading veritech decryption key from disk",
        );
        let mut file = File::open(decryption_key_path)
            .await
            .map_err(VeritechDecryptionKeyError::LoadKeyIO)?;
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        file.read_to_end(&mut buf)
            .await
            .map_err(VeritechDecryptionKeyError::LoadKeyIO)?;
        let secret_key =
            BoxSecretKey::from_slice(&buf).ok_or(VeritechDecryptionKeyError::KeyParse)?;

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

    /// Loads a [`VeritechDecryptionKey`] from a base64 encoded string.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key string could not be successfully parsed
    pub async fn decode(encryption_key_string: String) -> Result<Self, VeritechDecryptionKeyError> {
        trace!(
            "loading veritech encryption key from base64 string {}",
            encryption_key_string
        );
        let buf = general_purpose::STANDARD
            .decode(encryption_key_string)
            .map_err(VeritechDecryptionKeyError::Base64Decode)?;
        let secret_key =
            BoxSecretKey::from_slice(&buf).ok_or(VeritechDecryptionKeyError::KeyParse)?;
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
    ) -> Result<Vec<u8>, VeritechDecryptionKeyError> {
        let crypted = general_purpose::STANDARD_NO_PAD.decode(base64_encoded.as_ref())?;
        sodiumoxide::crypto::sealedbox::open(&crypted, &self.public_key, &self.secret_key)
            .map_err(|_| VeritechDecryptionKeyError::DecryptionFailed)
    }

    /// Returns a [`Hash`] of the encryption key which would have encoded a message.
    pub fn encryption_key_hash(&self) -> &Hash {
        &self.public_key_hash
    }

    /// Returns a string representaion of the hash of the encryption key which would have encoded a
    /// message.
    pub fn encryption_key_hash_str(&self) -> &str {
        self.public_key_hash_string.as_str()
    }
}

impl From<BoxSecretKey> for VeritechDecryptionKey {
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
