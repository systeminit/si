use std::path::Path;

use once_cell::sync::Lazy;
use sodiumoxide::crypto::box_::{PublicKey as BoxPublicKey, SecretKey as BoxSecretKey};
use std::{fs, io::Read};
use telemetry::prelude::*;
use thiserror::Error;

pub trait DecryptRequest {
    fn decrypt_request(self) -> Result<serde_json::Value, KeyPairError>;
}

pub static KEY_PAIR: Lazy<KeyPair> = Lazy::new(|| {
    // TODO: improve this to be production ready
    let mut secret_key_path = "/run/veritech/secret_key.bin".to_string();
    let mut public_key_path = "/run/veritech/public_key.pub".to_string();

    // TODO(fnichol): okay, this goes away/changes when we determine where the key would be by
    // default, etc.
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        secret_key_path = Path::new(&dir)
            .join("../../lib/cyclone/src/dev.secret_key.bin")
            .to_string_lossy()
            .to_string();
        public_key_path = Path::new(&dir)
            .join("../../lib/cyclone/src/dev.public_key.pub")
            .to_string_lossy()
            .to_string();
        telemetry::tracing::warn!(
            secret_key_path = secret_key_path.as_str(),
            public_key_path = public_key_path.as_str(),
            "detected cargo run, setting *default* cyclone key pair paths from sources"
        );
    }

    KeyPair::load(secret_key_path, public_key_path)
        .expect("Unable to open secret or public key, cyclone can't run without them")
});

#[derive(Error, Debug)]
pub enum KeyPairError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("base64 error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("failed to load key from bytes")]
    KeyParse,
    #[error("encrypted secret not found")]
    EncryptedSecretNotFound,
    #[error("failed to decrypt encryption key from bytes")]
    DecryptionFailed,
    #[error("json pointer not found: {1} at {:0?}")]
    JSONPointerNotFound(serde_json::Value, String),
}

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub secret_key: BoxSecretKey,
    pub public_key: BoxPublicKey,
}

impl KeyPair {
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, KeyPairError> {
        sodiumoxide::crypto::sealedbox::open(data, &self.public_key, &self.secret_key)
            .map_err(|_| KeyPairError::DecryptionFailed)
    }

    pub fn load(
        secret_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
    ) -> Result<Self, KeyPairError> {
        info!(
            secret_key_path = secret_key_path.as_ref().to_string_lossy().as_ref(),
            public_key_path = public_key_path.as_ref().to_string_lossy().as_ref(),
            "loading cyclone key pair"
        );
        let mut file = fs::File::open(&secret_key_path)?;
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        file.read_to_end(&mut buf)?;
        let secret_key = BoxSecretKey::from_slice(&buf).ok_or(KeyPairError::KeyParse)?;

        let mut file = fs::File::open(&public_key_path)?;
        buf.clear();
        file.read_to_end(&mut buf)?;
        let public_key = BoxPublicKey::from_slice(&buf).ok_or(KeyPairError::KeyParse)?;

        Ok(Self {
            secret_key,
            public_key,
        })
    }
}
