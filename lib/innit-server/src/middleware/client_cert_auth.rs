use std::{
    net::{
        IpAddr,
        Ipv4Addr,
        SocketAddr,
    },
    sync::Arc,
};

use axum::{
    body::{
        Body,
        BoxBody,
    },
    extract::{
        ConnectInfo,
        State,
    },
    http::{
        Request,
        StatusCode,
        header::HeaderMap,
    },
    middleware::Next,
    response::{
        IntoResponse,
        Response,
    },
};
use rustls::pki_types::CertificateDer;
use si_tls::{
    CertificateSource,
    ClientCertificateVerifier,
};
use telemetry::tracing::info;
use thiserror::Error;

type Result<T> = std::result::Result<T, ClientCertError>;

#[derive(Debug, Error)]
pub enum ClientCertError {
    #[error("Client certificate missing")]
    MissingCertificate,
    #[error("TLS Error: {0}")]
    Tls(#[from] si_tls::TlsError),
    #[error("Certificate validation failed")]
    ValidationFailed,
    #[error("Invalid header value: {0}")]
    InvalidHeader(String),
    #[error("Invalid certificate format: {0}")]
    InvalidFormat(String),
}

impl IntoResponse for ClientCertError {
    fn into_response(self) -> Response {
        let status = match self {
            ClientCertError::MissingCertificate => StatusCode::UNAUTHORIZED,
            ClientCertError::Tls(_) => StatusCode::BAD_REQUEST,
            ClientCertError::ValidationFailed => StatusCode::FORBIDDEN,
            ClientCertError::InvalidHeader(_) => StatusCode::BAD_REQUEST,
            ClientCertError::InvalidFormat(_) => StatusCode::BAD_REQUEST,
        };
        status.into_response()
    }
}

async fn extract_cert_from_headers(headers: &HeaderMap) -> Result<CertificateDer<'static>> {
    if let Some(cert_header) = headers.get("X-Forwarded-Client-Cert") {
        let cert_str = cert_header
            .to_str()
            .map_err(|e| ClientCertError::InvalidHeader(e.to_string()))?;

        let source = if cert_str.contains("BEGIN CERTIFICATE") {
            CertificateSource::AsString(cert_str.to_owned())
        } else {
            CertificateSource::Base64(cert_str.to_owned())
        };

        let certs = source.load_certificates().await?;
        let cert = certs
            .first()
            .ok_or_else(|| ClientCertError::InvalidFormat("No certificate found".into()))?
            .clone();

        return Ok(cert);
    }

    Err(ClientCertError::MissingCertificate)
}

pub async fn verify_client_cert_middleware(
    State(verifier): State<Arc<ClientCertificateVerifier>>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<BoxBody>> {
    let client_cert = req.extensions().get::<CertificateDer<'static>>().cloned();

    let peer_addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0)
        .unwrap_or_else(|| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0));

    for (name, value) in req.headers() {
        let value_str = value.to_str().unwrap_or("<binary value>");
        info!("CERT-DEBUG Header: {} = {}", name, value_str);
    }

    info!(
        "Verifying client certificate: peer={}, uri={}",
        peer_addr,
        req.uri()
    );

    let client_cert = match client_cert {
        Some(cert) => cert,
        None => extract_cert_from_headers(req.headers()).await?,
    };

    if verifier.verify_client_cert(&client_cert).await.is_err() {
        return Err(ClientCertError::ValidationFailed);
    }

    Ok(next.run(req).await)
}
