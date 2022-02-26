use std::{path::Path, pin::Pin};

use once_cell::sync::OnceCell;
use sodiumoxide::crypto::box_::{self, PublicKey as BoxPublicKey};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
};

pub static CYCLONE_PUBLIC_KEY: OnceCell<CyclonePublicKey> = OnceCell::new();

#[derive(Error, Debug)]
pub enum CyclonePublicKeyError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to load encryption key from bytes")]
    KeyParse,
}

pub type CyclonePublicKeyResult<T> = Result<T, CyclonePublicKeyError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CyclonePublicKey {
    pub key: BoxPublicKey,
}

impl CyclonePublicKey {
    pub async fn create(
        secret_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
    ) -> CyclonePublicKeyResult<Self> {
        let (public_key, secret_key) = box_::gen_keypair();

        let mut file = fs::File::create(&public_key_path).await?;
        file.write_all(&public_key.0).await?;

        let mut file = fs::File::create(&secret_key_path).await?;
        file.write_all(&secret_key.0).await?;

        Ok(Self { key: public_key })
    }

    pub fn encrypt_and_encode(&self, data: &str) -> String {
        let encrypted = sodiumoxide::crypto::sealedbox::seal(data.as_bytes(), &self.key);
        base64::encode(&encrypted)
    }

    pub async fn load(path: impl AsRef<Path>) -> CyclonePublicKeyResult<Self> {
        info!(
            path = path.as_ref().to_string_lossy().as_ref(),
            "loading cyclone public key"
        );
        let mut file = fs::File::open(path).await?;
        Self::from_reader(Pin::new(&mut file)).await
    }

    pub async fn from_reader(mut reader: Pin<&mut impl AsyncRead>) -> CyclonePublicKeyResult<Self> {
        let mut buf: Vec<u8> = Vec::with_capacity(32);
        reader.read_to_end(&mut buf).await?;
        let key = BoxPublicKey::from_slice(&buf).ok_or(CyclonePublicKeyError::KeyParse)?;

        Ok(Self { key })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn create() {
        let public_path = temp_dir().join("dev.dal_cyclone_secret_key_test.bin");
        let public_key = CyclonePublicKey::create(
            temp_dir().join("dev.dal_cyclone_public_key_test.pub"),
            &public_path,
        )
        .await
        .expect("Unable to create key pair");

        let mut buf = Vec::new();
        fs::File::open(public_path)
            .await
            .expect("Unable to open file")
            .read_to_end(&mut buf)
            .await
            .expect("Read from public key failed");
        let key = box_::PublicKey::from_slice(&buf).expect("Unable to make public key from bytes");
        assert_eq!(public_key.key, key);
    }
}
