//! Key pair generation and management utilities

use sodiumoxide::crypto::box_::{self, PublicKey, SecretKey};

use crate::{SodiumCryptoError, SodiumCryptoResult};

/// Generate a new cryptographic key pair
pub fn generate_keypair() -> (PublicKey, SecretKey) {
    box_::gen_keypair()
}

/// Create a PublicKey from raw bytes
pub fn public_key_from_slice(bytes: &[u8]) -> SodiumCryptoResult<PublicKey> {
    PublicKey::from_slice(bytes).ok_or(SodiumCryptoError::InvalidKeyFormat)
}

/// Create a SecretKey from raw bytes  
pub fn secret_key_from_slice(bytes: &[u8]) -> SodiumCryptoResult<SecretKey> {
    SecretKey::from_slice(bytes).ok_or(SodiumCryptoError::InvalidKeyFormat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        crate::init().expect("Failed to initialize sodiumoxide");
        
        let (public_key, secret_key) = generate_keypair();
        
        // Keys should have the correct length
        assert_eq!(public_key.as_ref().len(), 32);
        assert_eq!(secret_key.as_ref().len(), 32);
    }

    #[test]
    fn test_key_from_slice() {
        let public_bytes = [0u8; 32];
        let secret_bytes = [1u8; 32];
        
        let public_key = public_key_from_slice(&public_bytes).expect("Failed to create public key");
        let secret_key = secret_key_from_slice(&secret_bytes).expect("Failed to create secret key");
        
        assert_eq!(public_key.as_ref(), &public_bytes);
        assert_eq!(secret_key.as_ref(), &secret_bytes);
    }
}