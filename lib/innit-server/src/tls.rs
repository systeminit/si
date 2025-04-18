use rustls::pki_types::PrivateKeyDer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

use rustls::{
    pki_types::CertificateDer, server::WebPkiClientVerifier, RootCertStore, ServerConfig,
};
use rustls_pemfile::Item;
use si_std::CanonicalFile;
use tokio::{fs, io};

type Result<T> = std::result::Result<T, TlsError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum TlsError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Rustls(#[from] rustls::Error),
    #[error(transparent)]
    Verifier(#[from] rustls::server::VerifierBuilderError),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub cert_path: CanonicalFile,
    pub key_path: CanonicalFile,
    pub client_ca_cert_path: CanonicalFile,
}

pub async fn setup_tls_config(config: &TlsConfig) -> Result<Arc<ServerConfig>> {
    let cert = fs::read(&config.cert_path).await?;
    let key = fs::read(&config.key_path).await?;
    let client_ca_cert = fs::read(&config.client_ca_cert_path).await?;

    let cert = vec![rustls::pki_types::CertificateDer::from(cert)];
    let key = match rustls_pemfile::read_one(&mut key.as_ref()).unwrap() {
        Some(Item::Pkcs8Key(key)) => key,
        _ => panic!("private key invalid or not supported"),
    };

    let mut client_root_cert_store = RootCertStore::empty();
    client_root_cert_store.add(CertificateDer::from(client_ca_cert))?;

    let client_verifier =
        WebPkiClientVerifier::builder(Arc::new(client_root_cert_store)).build()?;

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(cert, PrivateKeyDer::Pkcs8(key))?;

    Ok(Arc::new(config))
}
