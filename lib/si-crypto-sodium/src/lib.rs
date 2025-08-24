//! Cryptographic utilities using sodiumoxide
//!
//! This crate provides high-level cryptographic operations using the sodiumoxide library,
//! including sealed box encryption/decryption and key pair management.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use thiserror::Error;

pub mod sealed_box;
pub mod key_pair;
pub mod serde_helpers;

// Re-export commonly used types
pub use sodiumoxide::crypto::box_::{PublicKey, SecretKey};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SodiumCryptoError {
    #[error("decryption failed")]
    DecryptionFailed,
    #[error("encryption failed")]
    EncryptionFailed,
    #[error("invalid key format")]
    InvalidKeyFormat,
    #[error("sodiumoxide initialization failed")]
    SodiumInitFailed,
}

pub type SodiumCryptoResult<T> = Result<T, SodiumCryptoError>;

/// Initialize sodiumoxide library
pub fn init() -> SodiumCryptoResult<()> {
    sodiumoxide::init().map_err(|()| SodiumCryptoError::SodiumInitFailed)
}