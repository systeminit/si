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
use telemetry::prelude::*;
use thiserror::Error;

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

/// The config for bedrock.
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default)]
    config_directory: CanonicalFile,

    #[builder(default)]
    output_directory: CanonicalFile,

    #[builder(default)]
    honeycomb_api_key: String,

    #[builder(default)]
    host_environment: String,

    #[builder(default)]
    instance_id: String,

    #[builder(default)]
    prometheus_remote_write_url: String,

    #[builder(default)]
    service_name: String,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    pub fn honeycomb_api_key(&self) -> &str {
        &self.honeycomb_api_key
    }

    pub fn host_environment(&self) -> &str {
        &self.host_environment
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn prometheus_remote_write_url(&self) -> &str {
        &self.prometheus_remote_write_url
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn config_directory(&self) -> &CanonicalFile {
        &self.config_directory
    }

    pub fn output_directory(&self) -> &CanonicalFile {
        &self.output_directory
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    config_directory: CanonicalFile,

    #[serde(default)]
    output_directory: CanonicalFile,

    #[serde(default)]
    honeycomb_api_key: String,

    #[serde(default)]
    host_environment: String,

    #[serde(default)]
    instance_id: String,

    #[serde(default)]
    prometheus_remote_write_url: String,

    #[serde(default)]
    service_name: String,
}

impl StandardConfigFile for ConfigFile {
    type Error = ConfigError;
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(mut value: ConfigFile) -> Result<Self> {
        detect_and_configure_development(&mut value)?;

        let mut config = Config::builder();
        config.config_directory(value.config_directory);
        config.output_directory(value.output_directory);
        config.honeycomb_api_key(value.honeycomb_api_key);
        config.host_environment(value.host_environment);
        config.instance_id(value.instance_id);
        config.prometheus_remote_write_url(value.prometheus_remote_write_url);
        config.service_name(value.service_name);
        config.build().map_err(Into::into)
    }
}

impl TryFrom<&Config> for ConfigFile {
    type Error = ConfigError;

    fn try_from(value: &Config) -> Result<Self> {
        Ok(ConfigFile {
            config_directory: value.config_directory.clone(),
            output_directory: value.output_directory.clone(),
            honeycomb_api_key: value.honeycomb_api_key.clone(),
            host_environment: value.host_environment.clone(),
            instance_id: value.instance_id.clone(),
            prometheus_remote_write_url: value.prometheus_remote_write_url.clone(),
            service_name: value.service_name.clone(),
        })
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

fn buck2_development(_config: &mut ConfigFile) -> Result<()> {
    let _resources = Buck2Resources::read().map_err(ConfigError::development)?;

    Ok(())
}

fn cargo_development(_dir: String, _config: &mut ConfigFile) -> Result<()> {
    Ok(())
}
