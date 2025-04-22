use base64::{
    Engine as _,
    engine::general_purpose::STANDARD,
};
use config::Config;
use reqwest::{
    Identity,
    Url,
};
use si_tls::CertificateResolver;
use thiserror::Error;

pub mod auth;
pub mod config;

pub use innit_core::*;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum InnitClientError {
    #[error("Deserialization error: {0}")]
    Deserialization(serde_json::Error),
    #[error("Request error: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("module not found (module id: {0})")]
    ModuleNotFound(String),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(serde_json::Error),
    #[error(transparent)]
    Tls(#[from] si_tls::TlsError),
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

type Result<T> = std::result::Result<T, InnitClientError>;

#[derive(Debug, Clone)]
pub struct InnitClient {
    client: reqwest::Client,
    base_url: Url,
}

impl InnitClient {
    pub async fn new(config: Config) -> Result<Self> {
        let use_https = config.base_url().scheme() == "https";
        let mut client_builder = reqwest::Client::builder();

        if let (Some(cert), Some(key)) = (
            &config.auth_config().client_cert,
            &config.auth_config().client_key,
        ) {
            let certs = cert.load_certificates().await?;
            if let Some(first_cert) = certs.first() {
                let cert_base64 = STANDARD.encode(first_cert);
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "X-Forwarded-Client-Cert",
                    reqwest::header::HeaderValue::from_str(&cert_base64)?,
                );
                client_builder = client_builder.default_headers(headers);
            }

            if use_https {
                let identity = CertificateResolver::create_identity(cert, key).await?;
                client_builder = client_builder.identity(Identity::from_pem(&identity)?);
            }
        }

        let client = client_builder.build()?;

        Ok(Self {
            client,
            base_url: config.base_url().clone(),
        })
    }

    pub async fn check_health(&self) -> Result<CheckHealthResponse> {
        let url = self.base_url.join("/")?;
        let resp = self.client.get(url).send().await?.error_for_status()?;
        let healthy = resp.json::<CheckHealthResponse>().await?;

        Ok(healthy)
    }

    pub async fn get_parameter(&self, name: String) -> Result<GetParameterResponse> {
        let url = self.base_url.join("parameter")?.join(&name)?;
        let resp = self.client.get(url).send().await?.error_for_status()?;
        let parameter = resp.json::<GetParameterResponse>().await?;

        Ok(parameter)
    }

    pub async fn get_parameters_by_path(&self, path: String) -> Result<ListParametersResponse> {
        let url = self.base_url.join("parameters")?.join(&path)?;
        let resp = self.client.get(url).send().await?.error_for_status()?;
        let parameters = resp.json::<ListParametersResponse>().await?;

        Ok(parameters)
    }
}
