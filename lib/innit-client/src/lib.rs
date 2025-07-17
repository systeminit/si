use std::{
    env,
    path::PathBuf,
};

use async_trait::async_trait;
use base64::{
    Engine as _,
    engine::general_purpose::STANDARD,
};
use config::{
    Config,
    ConfigFile,
};
use config_file::parameter_provider::{
    self,
    Parameter as ParameterProviderParameter,
    ParameterError,
    ParameterProvider,
};
use reqwest::Url;
use si_data_acmpca::{
    PrivateCertManagerClient,
    PrivateCertManagerClientError,
};
use si_settings::StandardConfigFile;
use si_std::{
    CanonicalFile,
    CanonicalFileError,
};
use si_tls::CertificateSource;
use telemetry::tracing::info;
use thiserror::Error;

pub mod auth;
pub mod config;

pub use innit_core::*;

const DEFAULT_CLIENT_NAME: &str = "innit";
const DEFAULT_ENVIRONMENT_ENV_VAR: &str = "SI_HOSTENV";
const DEFAULT_ENVIRONMENT: &str = "local";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum InnitClientError {
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error(transparent)]
    CertificateClient(#[from] PrivateCertManagerClientError),
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("Deserialization error: {0}")]
    Deserialization(serde_json::Error),
    #[error("Request error: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("module not found (module id: {0})")]
    ModuleNotFound(String),
    #[error("ParameterProvider error: {0}")]
    ParameterProvider(#[from] parameter_provider::ParameterError),
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
    environment: String,
    base_url: Url,
}

impl InnitClient {
    pub async fn new(config: Config) -> Result<Self> {
        let client_builder = reqwest::Client::builder();

        let mut environment = config.environment().to_string();

        let client_builder = if let Some(cert) = &config.auth_config().client_cert {
            environment = get_host_environment_from_cert_or_env_vars(cert).await?;
            info!("Determined we are running in environment: {environment}");
            Self::configure_client_with_certs(client_builder, cert).await?
        } else if let Some(ca_arn) = &config.client_ca_arn() {
            let cert = get_or_generate_cert(
                ca_arn.to_string(),
                config.for_app(),
                config.generated_cert_location().cloned(),
            )
            .await?;

            environment = get_host_environment_from_cert_or_env_vars(&cert).await?;
            info!("Determined we are running in environment: {environment}");
            Self::configure_client_with_certs(client_builder, &cert).await?
        } else {
            client_builder
        };

        let client = client_builder.build()?;

        Ok(Self {
            client,
            environment,
            base_url: config.base_url().clone(),
        })
    }

    pub async fn new_from_environment(for_app: String) -> Result<Self> {
        InnitClient::new(
            ConfigFile::layered_load(DEFAULT_CLIENT_NAME, |config_map| {
                config_map.set("for_app", for_app);
            })?
            .try_into()?,
        )
        .await
    }

    async fn configure_client_with_certs(
        client_builder: reqwest::ClientBuilder,
        cert_source: &CertificateSource,
    ) -> Result<reqwest::ClientBuilder> {
        let mut builder = client_builder;

        let certs = cert_source.load_certificates().await?;
        if let Some(first_cert) = certs.first() {
            let cert_base64 = STANDARD.encode(first_cert);
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                "X-Forwarded-Client-Cert",
                reqwest::header::HeaderValue::from_str(&cert_base64)?,
            );
            builder = builder.default_headers(headers);
        }

        Ok(builder)
    }

    pub async fn check_health(&self) -> Result<CheckHealthResponse> {
        let url = self.base_url.join("/")?;
        let resp = self.client.get(url).send().await?.error_for_status()?;
        let healthy = resp.json::<CheckHealthResponse>().await?;

        Ok(healthy)
    }

    pub async fn create_parameter(&self, name: String, value: String) -> Result<()> {
        let name = name.trim_start_matches('/');
        let url = self.join_path("parameter", name)?;
        let body = serde_json::json!({
            "value": value,
        });

        self.client
            .put(url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        info!("Created parameter: {name}");
        Ok(())
    }

    pub async fn get_parameter(&self, name: String) -> Result<GetParameterResponse> {
        let url = self.join_path("parameter", &name)?;
        let resp = self.client.get(url).send().await?.error_for_status()?;
        let parameter = resp.json::<GetParameterResponse>().await?;

        info!("Got parameter: {name}");
        Ok(parameter)
    }

    pub async fn get_parameters_by_path(&self, path: String) -> Result<ListParametersResponse> {
        let url = self.join_path("parameters", &path)?;
        let resp = self.client.get(url).send().await?.error_for_status()?;
        let parameters = resp.json::<ListParametersResponse>().await?;

        info!("Got parameters at: {path}");
        Ok(parameters)
    }

    pub async fn clear_parameter_cache(&self) -> Result<RefreshCacheResponse> {
        let url = self.join_path("cache", "clear")?;
        let resp = self.client.post(url).send().await?.error_for_status()?;

        let result = resp.json::<RefreshCacheResponse>().await?;
        info!("Cleared parameter cache.");
        Ok(result)
    }

    fn join_path(&self, base_segment: &str, path: &str) -> Result<Url> {
        let clean_path = path.trim_start_matches('/');
        let full_path = format!("{base_segment}/{clean_path}");
        Ok(self.base_url.join(&full_path)?)
    }

    pub fn environment(&self) -> String {
        self.environment.to_string()
    }
}

async fn generate_cert_from_acmpca(ca_arn: String, for_app: String) -> Result<CertificateSource> {
    let acmpca_client = PrivateCertManagerClient::new().await?;
    info!("Generating cert for ARN: {ca_arn}");
    let (cert, _) = acmpca_client
        .get_new_cert_from_ca(ca_arn, for_app, "innit".to_string())
        .await?;
    Ok(cert)
}

/// Generate a certificate, first checking the cache location if provided
async fn get_or_generate_cert(
    ca_arn: String,
    for_app: String,
    cached_cert: Option<PathBuf>,
) -> Result<CertificateSource> {
    // If we have a cache location, try to load from it first
    if let Some(cert_path) = &cached_cert {
        if cert_path.exists() {
            let cert = CanonicalFile::try_from(cert_path.as_path())?;
            let cached_source = CertificateSource::Path(cert);
            if cached_source.is_expired().await? {
                info!(
                    "Cached cert is expired, generating a new one: {:?}",
                    cert_path
                );
            } else if cached_source.load_certificates().await.is_ok() {
                info!("Using cached certificate from: {:?}", cert_path);
                return Ok(cached_source);
            }
        }
        info!("Failed to load cached certificate. Generating new one.");
    }

    let cert = generate_cert_from_acmpca(ca_arn, for_app).await?;

    if let Some(cert_path) = cached_cert {
        let cert_bytes = cert.load_certificates_as_bytes().await?;
        info!("Writing new certificate to cache: {:?}", cert_path);
        if let Some(parent) = cert_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&cert_path, &cert_bytes).await?;
        Ok(CertificateSource::Path(CanonicalFile::try_from(cert_path)?))
    } else {
        Ok(cert)
    }
}

// Attempt to pull the env the env var then from our issuing cert, failing that
async fn get_host_environment_from_cert_or_env_vars(cert: &CertificateSource) -> Result<String> {
    Ok(
        #[allow(clippy::disallowed_methods)]
        if let Ok(env) = env::var(DEFAULT_ENVIRONMENT_ENV_VAR) {
            env
        } else if let Some(cn) = cert.get_issuer_details().await?.common_name() {
            cn.to_string()
        } else {
            DEFAULT_ENVIRONMENT.to_string()
        },
    )
}

#[async_trait]
impl ParameterProvider for InnitClient {
    async fn get_parameters_by_path(
        &self,
        path: String,
    ) -> std::result::Result<Vec<ParameterProviderParameter>, ParameterError> {
        self.get_parameters_by_path(path)
            .await
            .map(|response| {
                response
                    .parameters
                    .into_iter()
                    .map(ParameterProviderParameter::from)
                    .collect()
            })
            .map_err(|e| ParameterError::Other(Box::new(e)))
    }

    async fn environment(&self) -> String {
        self.environment()
    }
}
