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
use x509_parser::{
    parse_x509_certificate,
    pem::{
        self,
    },
    prelude::X509Certificate,
};

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
    #[error("Verifier builder error: {0}")]
    Verifier(#[from] rustls::server::VerifierBuilderError),
    #[error("x509 parse error: {0}")]
    X509Parse(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CertificateIssuer {
    common_name: Option<String>,
    organization: Option<String>,
    organization_unit: Option<String>,
}

impl CertificateIssuer {
    pub fn common_name(&self) -> Option<&String> {
        self.common_name.as_ref()
    }
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

    /// Get issuer details from a cert
    pub async fn get_issuer_details(&self) -> Result<CertificateIssuer> {
        let bytes = self.load_certificates_as_bytes().await?;

        // Try to extract certificate data, handling both PEM and DER formats
        // Try parsing as PEM first
        if let Ok((_, pem)) = pem::parse_x509_pem(&bytes) {
            if let Ok((_, cert)) = parse_x509_certificate(&pem.contents) {
                return Ok(Self::extract_issuer_from_cert(&cert));
            }
        }

        // Fall back to DER format if PEM parsing failed
        if let Ok((_, cert)) = parse_x509_certificate(&bytes) {
            return Ok(Self::extract_issuer_from_cert(&cert));
        }

        Err(TlsError::X509Parse(
            "Failed to parse certificate in either PEM or DER format".to_string(),
        ))
    }

    // Helper function to extract issuer details from a certificate
    fn extract_issuer_from_cert(cert: &X509Certificate) -> CertificateIssuer {
        CertificateIssuer {
            common_name: cert
                .issuer()
                .iter_common_name()
                .next()
                .and_then(|cn| cn.as_str().ok().map(String::from)),
            organization: cert
                .issuer()
                .iter_organization()
                .next()
                .and_then(|cn| cn.as_str().ok().map(String::from)),
            organization_unit: cert
                .issuer()
                .iter_organizational_unit()
                .next()
                .and_then(|cn| cn.as_str().ok().map(String::from)),
        }
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
    pub async fn new(ca_certs: &[CertificateSource]) -> Result<Self> {
        let mut cert_store = RootCertStore::empty();

        for ca_cert in ca_certs {
            ca_cert.add_to_cert_store(&mut cert_store).await?;
        }

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
