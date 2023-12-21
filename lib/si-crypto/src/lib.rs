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

mod cyclone;
mod symmetric;

pub use cyclone::config::CryptoConfig;
pub use cyclone::decryption_key::{CycloneDecryptionKey, CycloneDecryptionKeyError};
pub use cyclone::encryption_key::{CycloneEncryptionKey, CycloneEncryptionKeyError};
pub use cyclone::key_pair::{CycloneKeyPair, CycloneKeyPairError};

pub use symmetric::{
    SymmetricCryptoError, SymmetricCryptoResult, SymmetricCryptoService,
    SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile, SymmetricKey, SymmetricNonce,
};
