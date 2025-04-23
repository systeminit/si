use std::path::Path;

use sodiumoxide::crypto::box_;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
};

use crate::{
    VeritechDecryptionKey,
    VeritechEncryptionKey,
};

/// An error that can be returned when working with a [`VeritechKeyPair`].
#[remain::sorted]
#[derive(Error, Debug)]
pub enum VeritechKeyPairError {
    /// When an error is return while creating and writing key files
    #[error("write io error: {0}")]
    WriteIo(#[from] std::io::Error),
}

/// A Veritech encryption/decryption key pair generator.
pub struct VeritechKeyPair;

impl VeritechKeyPair {
    /// Generates and writes a new pair of keys to the given file paths.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - A key file path cannot be created or is not writable (i.e. incorrect permission and/or
    ///   ownership)
    pub async fn create_and_write_files(
        secret_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
    ) -> Result<(), VeritechKeyPairError> {
        let (public_key, secret_key) = box_::gen_keypair();

        let mut file = File::create(&secret_key_path).await?;
        file.write_all(&secret_key.0).await?;

        let mut file = File::create(&public_key_path).await?;
        file.write_all(&public_key.0).await?;

        Ok(())
    }

    /// Generates a new pair of keys.
    pub fn create() -> (VeritechEncryptionKey, VeritechDecryptionKey) {
        let (public_key, secret_key) = box_::gen_keypair();

        (public_key.into(), secret_key.into())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use tokio::io::AsyncReadExt;

    use super::*;

    #[tokio::test]
    async fn create_and_write_files() {
        sodiumoxide::init().expect("failed to init sodiumoxide");

        let secret_key_path = NamedTempFile::new()
            .expect("failed to create named tempfile")
            .into_temp_path();
        let public_key_path = NamedTempFile::new()
            .expect("failed to create named tempfile")
            .into_temp_path();

        VeritechKeyPair::create_and_write_files(&secret_key_path, &public_key_path)
            .await
            .expect("unable to create key pair");

        let mut buf = Vec::new();
        File::open(&secret_key_path)
            .await
            .expect("unable to open secret key file")
            .read_to_end(&mut buf)
            .await
            .expect("failed to read from secret key file");
        let secret_key =
            box_::SecretKey::from_slice(&buf).expect("unable to parse secret key from bytes");

        buf.clear();
        File::open(&public_key_path)
            .await
            .expect("unable to open public key file")
            .read_to_end(&mut buf)
            .await
            .expect("failed to read from public key file");
        let public_key =
            box_::PublicKey::from_slice(&buf).expect("unable to parse public key from bytes");

        // Attempt an encryption/decryption round trip to ensure that both keys are related
        let message = "our-lady-peace".to_string();
        let crypted = sodiumoxide::crypto::sealedbox::seal(message.as_bytes(), &public_key);
        let decrypted = sodiumoxide::crypto::sealedbox::open(&crypted, &public_key, &secret_key)
            .expect("failed to decrypt");
        assert_eq!(message.as_bytes(), &decrypted);
    }
}
