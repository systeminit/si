use std::env;

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

/// The config for the forklift server.
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "default_auth_config()")]
    auth_config: AuthConfig,

    #[builder(default = "default_url()")]
    base_url: Url,
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default = "default_auth_config")]
    auth_config: AuthConfig,
    #[serde(default = "default_url")]
    base_url: Url,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            auth_config: Default::default(),
            base_url: default_url(),
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
        config.base_url(value.base_url);
        config.build().map_err(Into::into)
    }
}

fn default_url() -> Url {
    Url::parse("http://localhost:5166").expect("Unable to parse default base url!")
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
    println!("buck2");
    let resources = Buck2Resources::read().map_err(ConfigError::development)?;

    let client_cert = resources
        .get_ends_with("innit-client.dev.cert")
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

fn cargo_development(_dir: String, _config: &mut ConfigFile) -> Result<()> {
    Ok(())
}
