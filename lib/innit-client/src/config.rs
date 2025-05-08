use std::{
    env,
    path::{
        Path,
        PathBuf,
    },
};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
pub use si_settings::{
    StandardConfig,
    StandardConfigFile,
};
use si_std::{
    CanonicalFile,
    CanonicalFileError,
};
use si_tls::{
    CertificateSource,
    KeySource,
};
use telemetry::prelude::*;
use thiserror::Error;
use url::Url;

use crate::auth::AuthConfig;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error("si settings error: {0}")]
    SiSettings(#[from] si_settings::SettingsError),
}

impl ConfigError {
    fn development(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::Development(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

/// The config for the innit client.
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "default_auth_config()")]
    auth_config: AuthConfig,

    #[builder(default = None)]
    client_ca_arn: Option<String>,

    #[builder(default = "default_url()")]
    base_url: Url,

    #[builder(default = "default_env()")]
    environment: String,

    #[builder(default = "default_cert_cache_location()")]
    generated_cert_location: Option<PathBuf>,

    #[builder(default = "default_app_name()")]
    for_app: String,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    pub fn auth_config(&self) -> &AuthConfig {
        &self.auth_config
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn client_ca_arn(&self) -> Option<String> {
        self.client_ca_arn.clone()
    }

    pub fn for_app(&self) -> String {
        self.for_app.clone()
    }

    pub fn environment(&self) -> &str {
        &self.environment
    }

    pub fn generated_cert_location(&self) -> Option<&PathBuf> {
        self.generated_cert_location.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default = "default_auth_config")]
    auth_config: AuthConfig,
    #[serde(default)]
    client_ca_arn: Option<String>,
    #[serde(default = "default_url")]
    base_url: Url,
    #[serde(default = "default_env")]
    environment: String,
    #[serde(default = "default_app_name")]
    for_app: String,
    #[serde(default = "default_cert_cache_location")]
    generated_cert_location: Option<PathBuf>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            auth_config: Default::default(),
            client_ca_arn: None,
            base_url: default_url(),
            environment: default_env(),
            for_app: default_app_name(),
            generated_cert_location: default_cert_cache_location(),
        }
    }
}

impl StandardConfigFile for ConfigFile {
    type Error = ConfigError;
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(mut value: ConfigFile) -> Result<Self> {
        detect_and_configure_development(&mut value)?;

        let mut config = Config::builder();
        config.auth_config(value.auth_config);
        config.client_ca_arn(value.client_ca_arn);
        config.base_url(value.base_url);
        config.for_app(value.for_app);
        config.generated_cert_location(value.generated_cert_location);
        config.build().map_err(Into::into)
    }
}

fn default_cert_cache_location() -> Option<PathBuf> {
    Some("/etc/ssl/private/si-cert".to_string().into())
}

fn default_app_name() -> String {
    "innit-client".to_string()
}

fn default_env() -> String {
    "local".to_string()
}

fn default_url() -> Url {
    Url::parse("https://innit.systeminit.com").expect("Unable to parse default base url!")
}

fn default_auth_config() -> AuthConfig {
    AuthConfig {
        client_cert: None,
        client_key: None,
    }
}

#[allow(clippy::disallowed_methods)] // Used to determine if running in development
pub fn detect_and_configure_development(config: &mut ConfigFile) -> Result<()> {
    if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        buck2_development(config)
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        cargo_development(dir, config)
    } else {
        Ok(())
    }
}

fn buck2_development(config: &mut ConfigFile) -> Result<()> {
    let resources = Buck2Resources::read().map_err(ConfigError::development)?;

    let client_cert = resources
        .get_ends_with("innit-client.dev.crt")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();
    let client_key = resources
        .get_ends_with("innit-client.dev.key")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        client_cert = client_cert,
        client_key = client_key,
        "detected development run",
    );

    config.base_url = Url::parse("http://0.0.0.0:5166").expect("Unable to parse default base url!");
    config.auth_config.client_cert = Some(CertificateSource::Path(CanonicalFile::try_from(
        client_cert,
    )?));
    config.auth_config.client_key = Some(KeySource::Path(CanonicalFile::try_from(client_key)?));

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    let client_cert = Path::new(&dir)
        .join("../../lib/innit-client/innit-client.dev.crt")
        .to_string_lossy()
        .to_string();

    let client_key = Path::new(&dir)
        .join("../../lib/innit-client/innit-client.dev.key")
        .to_string_lossy()
        .to_string();

    warn!(
        client_cert = client_cert,
        client_key = client_key,
        "detected development run",
    );

    config.base_url = Url::parse("http://0.0.0.0:5166").expect("Unable to parse default base url!");
    config.auth_config.client_cert = Some(CertificateSource::Path(CanonicalFile::try_from(
        client_cert,
    )?));
    config.auth_config.client_key = Some(KeySource::Path(CanonicalFile::try_from(client_key)?));

    Ok(())
}
