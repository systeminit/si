use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
};

use deadpool_cyclone::{
    instance::cyclone::{LocalHttpInstanceSpec, LocalUdsInstance, LocalUdsInstanceSpec},
    Instance,
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data::NatsConfig;
use telemetry::prelude::*;
use thiserror::Error;

pub use si_settings::{StandardConfig, StandardConfigFile};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Builder(#[from] ConfigBuilderError),
    #[error("cyclone spec build error")]
    CycloneSpecBuild(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(setter(into, strip_option), default)]
    subject_prefix: Option<String>,

    cyclone_spec: CycloneSpec,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CycloneSpec {
    LocalUds(LocalUdsInstanceSpec),
    LocalHttp(LocalHttpInstanceSpec),
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    nats: NatsConfig,
}

impl StandardConfigFile for ConfigFile {
    type Error = ConfigError;
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(value: ConfigFile) -> Result<Self> {
        let mut config = Config::builder();
        config.nats(value.nats);
        config.cyclone_spec(create_hardcoded_cyclone_spec()?);
        config.build().map_err(Into::into)
    }
}

fn create_hardcoded_cyclone_spec() -> Result<CycloneSpec> {
    // TODO(fnichol): I'm asserting a default here that can eventually come from config
    // file/cli args etc, but for the moment--we all get a local uds setup

    // TODO(fnichol): okay, this goes away/changes when we determine what the right developer
    // defaults are vs. the right production/artifact defaults are
    let (cyclone_cmd_path, lang_server_cmd_path) = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(_) => {
            warn!("detected cargo run, setting *default* cyclone and lang-js paths under target");
            (
                "../../target/debug/cyclone",
                "../../bin/lang-js/target/lang-js",
            )
        }
        Err(_) => ("/usr/local/bin/cyclone", "/usr/local/bin/lang-js"),
    };
    Ok(CycloneSpec::LocalUds(
        LocalUdsInstance::spec()
            .try_cyclone_cmd_path(cyclone_cmd_path)
            .map_err(|err| ConfigError::CycloneSpecBuild(Box::new(err)))?
            .try_lang_server_cmd_path(lang_server_cmd_path)
            .map_err(|err| ConfigError::CycloneSpecBuild(Box::new(err)))?
            .resolver()
            .qualification()
            .build()
            .map_err(|err| ConfigError::CycloneSpecBuild(Box::new(err)))?,
    ))
}

impl Config {
    /// Gets a reference to the config's cyclone spec.
    pub fn cyclone_spec(&self) -> &CycloneSpec {
        &self.cyclone_spec
    }

    /// Gets a reference to the config's nats.
    #[must_use]
    pub fn nats(&self) -> &NatsConfig {
        &self.nats
    }

    /// Gets a reference to the config's subject prefix.
    pub fn subject_prefix(&self) -> Option<&str> {
        self.subject_prefix.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CycloneStream {
    HttpSocket(SocketAddr),
    UnixDomainSocket(PathBuf),
}

impl Default for CycloneStream {
    fn default() -> Self {
        Self::HttpSocket(SocketAddr::from(([0, 0, 0, 0], 5157)))
    }
}

impl CycloneStream {
    pub fn http_socket(socket_addrs: impl ToSocketAddrs) -> Result<Self> {
        let socket_addr = socket_addrs
            .to_socket_addrs()
            .map_err(ConfigError::SocketAddrResolve)?
            .into_iter()
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::HttpSocket(socket_addr))
    }
    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}
