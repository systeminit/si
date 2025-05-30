use std::{
    env,
    net::SocketAddr,
};

use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::NatsConfig;
pub use si_settings::{
    StandardConfig,
    StandardConfigFile,
};
use si_std::CanonicalFileError;
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

type Result<T> = std::result::Result<T, ConfigError>;

/// The config for the bedrock server.
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = "get_default_socket_addr()")]
    socket_addr: SocketAddr,

    #[builder(default)]
    dev_mode: bool,

    #[builder(default)]
    aws_access_key_id: String,

    #[builder(default)]
    aws_secret_access_key: String,

    #[builder(default)]
    aws_session_token: String,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets a reference to the config's nats.
    #[must_use]
    pub fn nats(&self) -> &NatsConfig {
        &self.nats
    }

    /// Gets the socket address
    pub fn socket_addr(&self) -> &SocketAddr {
        &self.socket_addr
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode
    }

    pub fn aws_secret_access_key(&self) -> String {
        self.aws_secret_access_key.clone()
    }

    pub fn aws_access_key_id(&self) -> String {
        self.aws_access_key_id.clone()
    }

    pub fn aws_session_token(&self) -> String {
        self.aws_session_token.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default = "get_default_socket_addr")]
    socket_addr: SocketAddr,
    #[serde(default)]
    pub dev_mode: bool,
    #[serde(default)]
    aws_secret_access_key: Option<String>,
    #[serde(default)]
    aws_access_key_id: Option<String>,
    #[serde(default)]
    aws_session_token: Option<String>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            nats: Default::default(),
            socket_addr: get_default_socket_addr(),
            dev_mode: false,
            aws_access_key_id: None,
            aws_secret_access_key: None,
            aws_session_token: None,
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
        config.nats(value.nats);
        config.dev_mode(value.dev_mode);

        let aws_access_key_id = value.aws_access_key_id;
        let aws_secret_access_key = value.aws_secret_access_key;
        let aws_session_token = value.aws_session_token;

        match &aws_access_key_id {
            Some(_) => {}
            None => warn!("AWS_ACCESS_KEY_ID not set — publishing may not be possible."),
        }

        match &aws_secret_access_key {
            Some(_) => {}
            None => warn!("AWS_SECRET_ACCESS_KEY not set — publishing may not be possible."),
        }

        config.aws_access_key_id(aws_access_key_id.unwrap_or_default());
        config.aws_secret_access_key(aws_secret_access_key.unwrap_or_default());
        config.aws_session_token(aws_session_token.unwrap_or_default());

        config.build().map_err(Into::into)
    }
}

fn get_default_socket_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 3020))
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
    info!("detected development run");

    config.dev_mode = true;

    Ok(())
}

fn cargo_development(_dir: String, _config: &mut ConfigFile) -> Result<()> {
    Ok(())
}
