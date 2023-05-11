use std::path::Path;

use sodiumoxide::crypto::box_;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{fs::File, io::AsyncWriteExt};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum CycloneKeyPairError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to load encryption key from bytes")]
    KeyParse,
}

pub type CycloneKeyPairResult<T> = Result<T, CycloneKeyPairError>;

pub struct CycloneKeyPair;

impl CycloneKeyPair {
    pub async fn create(
        secret_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
    ) -> CycloneKeyPairResult<()> {
        let (public_key, secret_key) = box_::gen_keypair();

        let mut file = File::create(&secret_key_path).await?;
        file.write_all(&secret_key.0).await?;

        let mut file = File::create(&public_key_path).await?;
        file.write_all(&public_key.0).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use tokio::io::AsyncReadExt;

    use super::*;

    #[tokio::test]
    async fn create() {
        sodiumoxide::init().expect("failed to init sodiumoxide");

        let secret_key_path = NamedTempFile::new()
            .expect("failed to create named tempfile")
            .into_temp_path();
        let public_key_path = NamedTempFile::new()
            .expect("failed to create named tempfile")
            .into_temp_path();

        CycloneKeyPair::create(&secret_key_path, &public_key_path)
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
