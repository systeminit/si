//! Symmetric key cryptography.

use std::{
    collections::HashMap,
    fs::File,
    io::Cursor,
    path::PathBuf,
    sync::Arc,
};

use base64::{
    Engine,
    engine::general_purpose,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_hash::Hash;
use si_std::{
    CanonicalFile,
    CanonicalFileError,
};
use sodiumoxide::crypto::secretbox;
pub use sodiumoxide::crypto::secretbox::Nonce as SymmetricNonce;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

/// An error that can be returned when working with the [`SymmetricCryptoService`].
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SymmetricCryptoError {
    /// When a base64 encoded key fails to be decoded.
    #[error("failed to decode base64 encoded key")]
    Base64Decode(#[source] base64::DecodeError),
    /// When a file fails to be canonicalized
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    /// When a cipertext fails to decrypt
    #[error("error when decrypting ciphertext")]
    DecryptionFailed,
    /// When deserializing from a key format fails
    #[error("error deserializing key : {0}")]
    Deserialize(#[from] ciborium::de::Error<std::io::Error>),
    /// When failing to supply appropriate values to form_config
    #[error("error loading from_config, must supply a filepath or base64 string")]
    FromConfig,
    /// When an error is returned while reading or writing to a key file
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// When attempting to decrypt and provided with a hash for a key that is not present
    #[error("no key present matching provided hash")]
    MissingKeyForHash,
    /// When serializing to a key file format fails
    #[error("error serializing key file: {0}")]
    Serialize(#[from] ciborium::ser::Error<std::io::Error>),
    /// When a Tokio task join fails
    #[error("error joining task: {0}")]
    TaskJoin(#[from] JoinError),
}

/// A result type when working with a [`SymmetricCryptoService`].
pub type SymmetricCryptoResult<T> = Result<T, SymmetricCryptoError>;

/// A service that can encrypt and decrypt arbitrary data using a set of symmetric keys.
#[derive(Clone, Debug)]
pub struct SymmetricCryptoService {
    keys: Arc<HashMap<Hash, SymmetricKey>>,
    active_key_hash: Arc<Hash>,
}

/// A configuration that can be used to build a [`SymmetricCryptoService`].
///
/// A primary "active key" is used when encrypting data and may be used when decrypting data. A
/// [`Hash`] of a key is provided when decrypting data which is used to look up the appropriate
/// loaded key. In this way the service can take an arbitrary number of keys which is useful in
/// operations such as key rotation where at least 2 keys are needed (the new key and the old
/// keys).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SymmetricCryptoServiceConfig {
    /// The path to the active key file which will be used for all encryption.
    #[serde(skip_serializing)]
    pub active_key: Option<CanonicalFile>,
    /// The base64 representation of the active key file which will be used for all encryption.
    #[serde(skip_serializing)]
    pub active_key_base64: Option<String>,
    /// Extra keys which can be used when decrypting data.
    #[serde(skip_serializing)]
    pub extra_keys: Vec<CanonicalFile>,
}

/// A config file representation of a [`SymmetricCryptoService`] configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SymmetricCryptoServiceConfigFile {
    /// The path to the active key file which will be used for all encryption.
    pub active_key: Option<String>,
    /// The base64 representation of the active key file which will be used for all encryption.
    pub active_key_base64: Option<String>,
    /// Extra keys which can be used when decrypting data.
    pub extra_keys: Vec<String>,
}

impl TryFrom<SymmetricCryptoServiceConfigFile> for SymmetricCryptoServiceConfig {
    type Error = CanonicalFileError;

    fn try_from(value: SymmetricCryptoServiceConfigFile) -> Result<Self, Self::Error> {
        let mut active_key: Option<CanonicalFile> = None;
        let mut active_key_base64: Option<String> = None;
        if let Some(key) = value.active_key {
            active_key = Some(key.try_into()?);
        }
        if let Some(key) = value.active_key_base64 {
            active_key_base64 = Some(key);
        }
        let mut extra_keys = Vec::new();
        for extra_key_str in value.extra_keys {
            extra_keys.push(extra_key_str.try_into()?);
        }

        Ok(Self {
            active_key,
            extra_keys,
            active_key_base64,
        })
    }
}

impl SymmetricCryptoService {
    /// Creates and returns a new service loaded with the given [`SymmetricKey`]s.
    pub fn new(active_key: SymmetricKey, extra_keys: Vec<SymmetricKey>) -> Self {
        let mut keys = HashMap::new();

        let active_key_hash = Hash::new(active_key.0.as_ref());
        keys.insert(active_key_hash, active_key);

        for key in extra_keys {
            keys.insert(Hash::new(key.0.as_ref()), key);
        }

        Self {
            keys: Arc::new(keys),
            active_key_hash: Arc::new(active_key_hash),
        }
    }

    /// Creates and returns a new service from the given [`SymmetricCryptoServiceConfig`].
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key file was not readable (i.e. incorrect permissions and/or ownership)
    /// - A key file could not be successfully parsed
    /// - The [`SymmetricKey`] could not be successfully resolved from loading the key file
    pub async fn from_config(config: &SymmetricCryptoServiceConfig) -> SymmetricCryptoResult<Self> {
        let active_key = match (&config.active_key, &config.active_key_base64) {
            (Some(key), None) => Ok(SymmetricKey::load(key).await?),
            (None, Some(b64_string)) => Ok(SymmetricKey::decode(b64_string.to_string()).await?),
            _ => Err(SymmetricCryptoError::FromConfig),
        }?;
        let mut extra_keys = vec![];

        for key_path in config.extra_keys.iter() {
            extra_keys.push(SymmetricKey::load(&key_path).await?);
        }

        Ok(Self::new(active_key, extra_keys))
    }

    /// Generates a new [`SymmetricKey`].
    pub fn generate_key() -> SymmetricKey {
        SymmetricKey(secretbox::gen_key())
    }

    #[allow(clippy::missing_panics_doc)]
    /// Encrypts a message and returns the crypted bytes, a nonce, and a [`Hash`] of the encrypting
    /// [`SymmetricKey`].
    pub fn encrypt(&self, message: &[u8]) -> (Vec<u8>, SymmetricNonce, &Hash) {
        let key = self
            .keys
            .get(self.active_key_hash.as_ref())
            .expect("active_key value not present in keys hashmap; this is bug!");
        let nonce = secretbox::gen_nonce();

        (
            secretbox::seal(message, &nonce, &key.0),
            nonce,
            self.active_key_hash.as_ref(),
        )
    }

    /// Decrypts a ciphertext provided with a nonce and a [`Hash`] of the encrypting
    /// [`SymmetricKey`] and returns the decrypted message.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - No key was loaded for the given key hash
    /// - An invalid nonce is provided
    /// - An incorrect key hash is provided (i.e. referring to another loaded key that was not used
    ///   to encrypt the message)
    /// - An invalid ciphertext was provided
    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce: &SymmetricNonce,
        key_hash: &Hash,
    ) -> SymmetricCryptoResult<Vec<u8>> {
        let key = self
            .keys
            .get(key_hash)
            .ok_or(SymmetricCryptoError::MissingKeyForHash)?;

        secretbox::open(ciphertext, nonce, &key.0)
            .map_err(|_| SymmetricCryptoError::DecryptionFailed)
    }
}

/// A symmetric encryption key (i.e. a key which can encrypt *and* decrypt data).
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct SymmetricKey(secretbox::Key);

impl SymmetricKey {
    /// Save a simple key to a file on the given path.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - The key file could not be created (i.e. permissions/ownership issues)
    /// - The key file's parent directory is not created or not accessible due to
    ///   permissions/ownship issues
    pub async fn save(&self, path: impl Into<PathBuf>) -> SymmetricCryptoResult<()> {
        let file_data = SymmetricKeyFile { key: self.clone() };

        file_data.save(path).await
    }

    /// Load a key from a file on the given path.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - The key file was not found
    /// - The key file was not readable (i.e. incorrect permissions and/or ownership)
    /// - The key file could not be successfully parsed
    /// - The [`SymmetricKey`] could not be successfully resolved from loading the key file
    pub async fn load(path: impl Into<PathBuf>) -> SymmetricCryptoResult<Self> {
        Ok(SymmetricKeyFile::load(path).await?.into())
    }

    /// Load a key from a base64 string.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - The key string could not be successfully parsed
    /// - The [`SymmetricKey`] could not be successfully resolved
    pub async fn decode(key_string: String) -> SymmetricCryptoResult<Self> {
        Ok(SymmetricKeyFile::decode(key_string).await?.into())
    }
}

impl From<SymmetricKeyFile> for SymmetricKey {
    fn from(value: SymmetricKeyFile) -> Self {
        value.key
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct SymmetricKeyFile {
    key: SymmetricKey,
}

impl SymmetricKeyFile {
    async fn save(&self, path: impl Into<PathBuf>) -> SymmetricCryptoResult<()> {
        let path = path.into();
        let self_clone = self.clone();

        tokio::task::spawn_blocking(move || {
            let file = File::create(&path)?;

            ciborium::into_writer(&self_clone, file)
        })
        .await?
        .map_err(Into::into)
    }

    async fn decode(key_string: String) -> SymmetricCryptoResult<Self> {
        let buf = general_purpose::STANDARD
            .decode(key_string)
            .map_err(SymmetricCryptoError::Base64Decode)?;
        ciborium::from_reader(Cursor::new(&buf)).map_err(Into::into)
    }

    async fn load(path: impl Into<PathBuf>) -> SymmetricCryptoResult<Self> {
        let path = path.into();

        tokio::task::spawn_blocking(move || {
            let file = File::open(path)?;
            ciborium::from_reader(file)
        })
        .await?
        .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn encryption_decryption_round_trip() {
        let key = SymmetricCryptoService::generate_key();
        let service = SymmetricCryptoService::new(key, vec![]);

        let message = b"Leave the gun. Take the cannoli.";

        let (ciphertext, nonce, key_hash) = service.encrypt(message);

        let decrypted = service
            .decrypt(ciphertext.as_ref(), &nonce, key_hash)
            .expect("Should be able to decrypt");

        assert_eq!(message.as_slice(), decrypted);
    }

    #[test]
    fn key_rotation() {
        let old_key = SymmetricCryptoService::generate_key();
        let old_service = SymmetricCryptoService::new(old_key.clone(), vec![]);

        let message = b"My father made him an offer he couldn't refuse.";

        let (ciphertext, nonce, old_key_hash) = old_service.encrypt(message);

        let new_key = SymmetricCryptoService::generate_key();
        let new_service = SymmetricCryptoService::new(new_key, vec![old_key]);

        let decrypted = new_service
            .decrypt(ciphertext.as_ref(), &nonce, old_key_hash)
            .expect("Should be able to decrypt");

        assert_eq!(message.as_slice(), decrypted);
    }

    #[test]
    fn missing_key() {
        let old_key = SymmetricCryptoService::generate_key();
        let old_service = SymmetricCryptoService::new(old_key.clone(), vec![]);

        let message = b"My father made him an offer he couldn't refuse.";

        let (ciphertext, nonce, old_key_hash) = old_service.encrypt(message);

        let new_key = SymmetricCryptoService::generate_key();
        let new_service = SymmetricCryptoService::new(new_key, vec![]);

        let result = new_service.decrypt(ciphertext.as_ref(), &nonce, old_key_hash);

        assert!(matches!(
            result,
            Err(SymmetricCryptoError::MissingKeyForHash)
        ));
    }

    #[tokio::test]
    async fn filesystem_round_trip() {
        let key = SymmetricCryptoService::generate_key();

        let file = NamedTempFile::new().expect("Should create temp file");
        key.save(file.path()).await.expect("Should write to file");

        let loaded_key = SymmetricKey::load(file.path())
            .await
            .expect("Should load from file");

        assert_eq!(key, loaded_key);
    }
}
