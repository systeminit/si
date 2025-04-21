use base64::{Engine as _, engine::general_purpose::STANDARD};
use rustls::pki_types::TrustAnchor;
use std::path::Path;

use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;
use serde::{Deserialize, Serialize};
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
    Base64(String),
    Path(CanonicalFile),
    Remote(String),
}

pub struct CertificateResolver;

impl CertificateResolver {
    // Load certificated based on the source location
    pub async fn load_certificates(
        source: &CertificateSource,
    ) -> Result<Vec<CertificateDer<'static>>> {
        match source {
            CertificateSource::Path(path) => Self::get_certificate_from_path(path).await,
            CertificateSource::Base64(data) => {
                Self::get_certificate_from_base64(data.clone()).await
            }
            CertificateSource::Remote(url) => Self::get_certificate_from_remote(url.clone()).await,
        }
    }

    /// Add certificates from any source to a RootCertStore
    pub async fn add_to_cert_store(
        source: &CertificateSource,
        store: &mut RootCertStore,
    ) -> Result<()> {
        let certs = Self::load_certificates(source).await?;
        store.add_parsable_certificates(certs);
        Ok(())
    }

    /// Creates a certificate object from bytes
    async fn get_certificate_from_bytes(bytes: &[u8]) -> Result<Vec<CertificateDer<'static>>> {
        let mut reader = std::io::BufReader::new(bytes);
        rustls_pemfile::certs(&mut reader)
            .map(|result| result.map_err(TlsError::Io))
            .collect::<Result<Vec<_>>>()
    }

    /// Creates a Certificate object from a base64 encoded certificate
    async fn get_certificate_from_base64(
        key_string: String,
    ) -> Result<Vec<CertificateDer<'static>>> {
        let buf = STANDARD.decode(key_string)?;
        Self::get_certificate_from_bytes(&buf).await
    }

    /// Creates a Certificate object from a certificate file
    async fn get_certificate_from_path(
        path: impl AsRef<Path>,
    ) -> Result<Vec<CertificateDer<'static>>> {
        let buf = std::fs::read(path)?;
        Self::get_certificate_from_bytes(&buf).await
    }

    /// Creates a Certificate object from a remote certificate
    async fn get_certificate_from_remote(url: String) -> Result<Vec<CertificateDer<'static>>> {
        let contents = &reqwest::get(&url).await?.bytes().await?;
        Self::get_certificate_from_bytes(contents).await
    }

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
}

// Leaving commented until in-use by innit
//
// #[derive(Clone)]
// pub struct ClientCertificateVerifier {
//     verifier: Arc<dyn ClientCertVerifier>,
// }
//
// pub async fn setup_client_verifier(tls_config: &TlsConfig) -> Result<ClientCertificateVerifier> {
//     let ca_cert_data = tokio::fs::read(&tls_config.client_ca_cert_path).await?;
//
//     let ca_certs = rustls_pemfile::certs(&mut ca_cert_data.as_slice())
//         .flatten()
//         .collect::<Vec<_>>();
//
//     let mut cert_store = RootCertStore::empty();
//     for cert in ca_certs {
//         cert_store.add(cert)?;
//     }
//
//     let verifier = WebPkiClientVerifier::builder(Arc::new(cert_store)).build()?;
//
//     Ok(ClientCertificateVerifier { verifier })
// }
//
// impl ClientCertificateVerifier {
//     pub fn verify_client_cert(&self, cert_der: &[u8]) -> Result<()> {
//         let cert = CertificateDer::from(cert_der.to_vec());
//         if self
//             .verifier
//             .verify_client_cert(&cert, &[], UnixTime::now())
//             .is_err()
//         {
//             return Err(TlsError::CertificateVerification(
//                 "Client certificate verification failed".into(),
//             ));
//         }
//
//         Ok(())
//     }
// }
