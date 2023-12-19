//! System Initiative standard cryptography.

#![warn(
    clippy::unwrap_in_result,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn,
    missing_docs
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_inception,
    clippy::module_name_repetitions
)]

use serde::{Deserialize, Serialize};
use si_std::CanonicalFile;

mod cyclone;
mod symmetric;

pub use cyclone::decryption_key::{CycloneDecryptionKey, CycloneDecryptionKeyError};
pub use cyclone::encryption_key::{CycloneEncryptionKey, CycloneEncryptionKeyError};
pub use cyclone::key_pair::{CycloneKeyPair, CycloneKeyPairError};

pub use symmetric::{
    SymmetricCryptoError, SymmetricCryptoResult, SymmetricCryptoService,
    SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile, SymmetricKey, SymmetricNonce,
};

/// Configuration for how to load the key for [`CryptoConfig`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CryptoConfig {
    /// Key file encoded as a base64 string
    pub encryption_key_base64: Option<String>,
    /// Key file on disk
    pub encryption_key_file: Option<CanonicalFile>,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            encryption_key_base64: None,
            encryption_key_file: None,
        }
    }
}
