//! Sealed box encryption/decryption utilities
//!
//! Sealed boxes provide anonymous encryption where only the public key is needed to encrypt
//! and both the public and secret key are needed to decrypt.

use sodiumoxide::crypto::{
    box_::{PublicKey, SecretKey},
    sealedbox,
};

use crate::{SodiumCryptoError, SodiumCryptoResult};

/// Encrypt data using sealed box encryption
pub fn seal(data: &[u8], public_key: &PublicKey) -> Vec<u8> {
    sealedbox::seal(data, public_key)
}

/// Decrypt data using sealed box decryption
pub fn open(
    ciphertext: &[u8],
    public_key: &PublicKey,
    secret_key: &SecretKey,
) -> SodiumCryptoResult<Vec<u8>> {
    sealedbox::open(ciphertext, public_key, secret_key)
        .map_err(|()| SodiumCryptoError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sodiumoxide::crypto::box_;

    #[test]
    fn test_seal_and_open() {
        crate::init().expect("Failed to initialize sodiumoxide");
        
        let (public_key, secret_key) = box_::gen_keypair();
        let message = b"Hello, world!";
        
        let sealed = seal(message, &public_key);
        let opened = open(&sealed, &public_key, &secret_key).expect("Failed to decrypt");
        
        assert_eq!(message, opened.as_slice());
    }
}