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

mod sensitive_strings;
mod symmetric;
mod veritech;

pub use sensitive_strings::SensitiveStrings;
pub use symmetric::{
    SymmetricCryptoError, SymmetricCryptoResult, SymmetricCryptoService,
    SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile, SymmetricKey, SymmetricNonce,
};
pub use veritech::{
    config::CryptoConfig,
    decryption_key::{VeritechDecryptionKey, VeritechDecryptionKeyError},
    encryption_key::{VeritechEncryptionKey, VeritechEncryptionKeyError},
    key_pair::{VeritechKeyPair, VeritechKeyPairError},
};
