use std::sync::Arc;

use base64::{
    Engine as _,
    engine::general_purpose::STANDARD,
};
use rustls::{
    RootCertStore,
    pki_types::{
        CertificateDer,
        PrivateKeyDer,
        TrustAnchor,
        UnixTime,
    },
    server::{
        WebPkiClientVerifier,
        danger::ClientCertVerifier,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use si_std::CanonicalFile;
use thiserror::Error;
use tokio::io;
use tokio_rustls::rustls;

type Result<T> = std::result::Result<T, TlsError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum TlsError {
    #[error("Failed to decode base64 certificate: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("{0}")]
    CertificateVerification(String),
    #[error("Failed to fetch remote certificate: {0}")]
    FetchRemote(#[from] reqwest::Error),
    #[error("Failed to read cert: {0}")]
    Io(#[from] io::Error),
    #[error("Rustls error: {0}")]
    Rustls(#[from] rustls::Error),
    #[error("Verifier bulder error: {0}")]
    Verifier(#[from] rustls::server::VerifierBuilderError),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "source")]
pub enum CertificateSource {
    AsString(String),
    Base64(String),
    Path(CanonicalFile),
    Remote(String),
}

impl CertificateSource {
    /// Load certs from any CertificateSource
    pub async fn load_certificates(&self) -> Result<Vec<CertificateDer<'static>>> {
        Self::get_certificate_from_bytes(self.load_certificates_as_bytes().await?).await
    }

    /// Load certs as bytes from any CertificateSource
    pub async fn load_certificates_as_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::AsString(string) => Ok(string.as_bytes().to_vec()),

            Self::Base64(data) => Ok(STANDARD.decode(data)?),
            Self::Path(path) => Ok(tokio::fs::read(path).await?),
            Self::Remote(url) => Ok(reqwest::get(url).await?.bytes().await?.to_vec()),
        }
    }

    /// Creates a certificate object from bytes
    async fn get_certificate_from_bytes(bytes: Vec<u8>) -> Result<Vec<CertificateDer<'static>>> {
        let mut reader = std::io::BufReader::new(&*bytes);
        let pem_certs = rustls_pemfile::certs(&mut reader)
            .map(|result| result.map_err(TlsError::Io))
            .collect::<Result<Vec<_>>>()?;

        if !pem_certs.is_empty() {
            return Ok(pem_certs);
        }

        if let Ok(content) = std::str::from_utf8(&bytes) {
            let content = content.trim();
            if let Ok(der_bytes) = STANDARD.decode(content) {
                return Ok(vec![CertificateDer::from(der_bytes)]);
            }
        }

        if !bytes.is_empty() && bytes[0] == 0x30 {
            return Ok(vec![CertificateDer::from(bytes)]);
        }

        Err(TlsError::Rustls(rustls::Error::General(
            "Failed to parse certificate: not a valid PEM, base64-encoded DER, or raw DER".into(),
        )))
    }

    /// Add certificates from any source to a RootCertStore
    pub async fn add_to_cert_store(&self, store: &mut RootCertStore) -> Result<()> {
        let certs = self.load_certificates().await?;
        store.add_parsable_certificates(certs);
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "source")]
pub enum KeySource {
    AsString(String),
    Base64(String),
    Path(CanonicalFile),
    Remote(String),
}

impl KeySource {
    /// Load private key from any CertificateSource
    pub async fn load_private_key(&self) -> Result<PrivateKeyDer<'static>> {
        Self::get_private_key_from_bytes(&self.load_private_key_as_bytes().await?).await
    }

    /// Load private keys as bytes from any CertificateSource
    pub async fn load_private_key_as_bytes(&self) -> Result<Vec<u8>> {
        match self {
            Self::AsString(string) => Ok(string.as_bytes().to_vec()),
            Self::Base64(data) => Ok(STANDARD.decode(data)?),
            Self::Path(path) => Ok(tokio::fs::read(path).await?),
            Self::Remote(url) => Ok(reqwest::get(url).await?.bytes().await?.to_vec()),
        }
    }

    /// Creates a private key from bytes, trying both PKCS8 and RSA formats
    async fn get_private_key_from_bytes(bytes: &[u8]) -> Result<PrivateKeyDer<'static>> {
        let mut reader = std::io::BufReader::new(bytes);

        // Try PKCS8 first
        let pkcs8_keys =
            rustls_pemfile::pkcs8_private_keys(&mut reader).collect::<std::io::Result<Vec<_>>>()?;
        if let Some(key) = pkcs8_keys.into_iter().next() {
            return Ok(PrivateKeyDer::Pkcs8(key));
        }

        // Reset reader and try RSA format
        reader = std::io::BufReader::new(bytes);
        let rsa_keys =
            rustls_pemfile::rsa_private_keys(&mut reader).collect::<std::io::Result<Vec<_>>>()?;
        if let Some(key) = rsa_keys.into_iter().next() {
            return Ok(PrivateKeyDer::Pkcs1(key));
        }

        Err(TlsError::Rustls(rustls::Error::General(
            "No valid private key found in input".into(),
        )))
    }
}

pub struct CertificateResolver;

impl CertificateResolver {
    /// Creates a root cert store from the common tls sources
    pub async fn create_root_store() -> Result<RootCertStore> {
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| TrustAnchor {
            subject: ta.subject.into(),
            subject_public_key_info: ta.spki.into(),
            name_constraints: ta.name_constraints.map(Into::into),
        }));
        Ok(root_cert_store)
    }

    /// Create an identity by combining certificate and private key
    pub async fn create_identity(
        cert_source: &CertificateSource,
        key_source: &KeySource,
    ) -> Result<Vec<u8>> {
        let cert_bytes = cert_source.load_certificates_as_bytes().await?;
        let key_bytes = key_source.load_private_key_as_bytes().await?;

        let mut identity = Vec::new();
        identity.extend_from_slice(&cert_bytes);
        identity.extend_from_slice(&key_bytes);

        Ok(identity)
    }
}

#[derive(Debug, Clone)]
pub struct ClientCertificateVerifier {
    verifier: Arc<dyn ClientCertVerifier>,
}

impl ClientCertificateVerifier {
    pub async fn new(ca_cert: CertificateSource) -> Result<Self> {
        let mut cert_store = RootCertStore::empty();
        ca_cert.add_to_cert_store(&mut cert_store).await?;

        let verifier = WebPkiClientVerifier::builder(Arc::new(cert_store)).build()?;

        Ok(ClientCertificateVerifier { verifier })
    }

    pub async fn verify_client_cert(&self, cert: &CertificateDer<'static>) -> Result<()> {
        if self
            .verifier
            .verify_client_cert(cert, &[], UnixTime::now())
            .is_err()
        {
            return Err(TlsError::CertificateVerification(
                "Client certificate verification failed".into(),
            ));
        }

        Ok(())
    }
}
