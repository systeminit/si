use std::{
    env,
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
    time::Duration,
};

use buck2_resources::Buck2Resources;
use deadpool_cyclone::{
    instance::cyclone::{
        LocalHttpInstance, LocalHttpInstanceSpec, LocalHttpSocketStrategy, LocalUdsInstance,
        LocalUdsInstanceSpec, LocalUdsRuntimeStrategy, LocalUdsSocketStrategy,
    },
    Instance,
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
use telemetry::prelude::*;
use thiserror::Error;

pub use si_settings::{StandardConfig, StandardConfigFile};

#[remain::sorted]
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

impl ConfigError {
    fn cyclone_spec_build(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::CycloneSpecBuild(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    cyclone_spec: CycloneSpec,
}

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum CycloneSpec {
    LocalHttp(LocalHttpInstanceSpec),
    LocalUds(LocalUdsInstanceSpec),
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    pub nats: NatsConfig,
    pub cyclone: CycloneConfig,
}

impl ConfigFile {
    pub fn default_local_http() -> Self {
        Self {
            nats: Default::default(),
            cyclone: CycloneConfig::default_local_http(),
        }
    }

    pub fn default_local_uds() -> Self {
        Self {
            nats: Default::default(),
            cyclone: CycloneConfig::default_local_uds(),
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
        config.cyclone_spec(value.cyclone.try_into()?);
        config.build().map_err(Into::into)
    }
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
        self.nats.subject_prefix.as_deref()
    }

    // Consumes into a [`CycloneSpec`].
    pub fn into_cyclone_spec(self) -> CycloneSpec {
        self.cyclone_spec
    }
}

#[remain::sorted]
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
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::HttpSocket(socket_addr))
    }
    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum CycloneConfig {
    LocalHttp {
        #[serde(default = "default_cyclone_cmd_path")]
        cyclone_cmd_path: String,
        #[serde(default = "default_cyclone_decryption_key_path")]
        cyclone_decryption_key_path: String,
        #[serde(default = "default_lang_server_cmd_path")]
        lang_server_cmd_path: String,
        #[serde(default)]
        socket_strategy: LocalHttpSocketStrategy,
        #[serde(default)]
        watch_timeout: Option<Duration>,
        #[serde(default = "default_limit_requests")]
        limit_requets: Option<u32>,
        #[serde(default = "default_enable_endpoint")]
        ping: bool,
        #[serde(default = "default_enable_endpoint")]
        resolver: bool,
        #[serde(default = "default_enable_endpoint")]
        action: bool,
    },
    LocalUds {
        #[serde(default = "default_cyclone_cmd_path")]
        cyclone_cmd_path: String,
        #[serde(default = "default_cyclone_decryption_key_path")]
        cyclone_decryption_key_path: String,
        #[serde(default = "default_lang_server_cmd_path")]
        lang_server_cmd_path: String,
        #[serde(default)]
        socket_strategy: LocalUdsSocketStrategy,
        #[serde(default)]
        runtime_strategy: LocalUdsRuntimeStrategy,
        #[serde(default)]
        watch_timeout: Option<Duration>,
        #[serde(default = "default_limit_requests")]
        limit_requets: Option<u32>,
        #[serde(default = "default_enable_endpoint")]
        ping: bool,
        #[serde(default = "default_enable_endpoint")]
        resolver: bool,
        #[serde(default = "default_enable_endpoint")]
        action: bool,
    },
}

impl CycloneConfig {
    pub fn default_local_http() -> Self {
        Self::LocalHttp {
            cyclone_cmd_path: default_cyclone_cmd_path(),
            cyclone_decryption_key_path: default_cyclone_decryption_key_path(),
            lang_server_cmd_path: default_lang_server_cmd_path(),
            socket_strategy: Default::default(),
            watch_timeout: Default::default(),
            limit_requets: default_limit_requests(),
            ping: default_enable_endpoint(),
            resolver: default_enable_endpoint(),
            action: default_enable_endpoint(),
        }
    }

    pub fn default_local_uds() -> Self {
        Self::LocalUds {
            cyclone_cmd_path: default_cyclone_cmd_path(),
            cyclone_decryption_key_path: default_cyclone_decryption_key_path(),
            lang_server_cmd_path: default_lang_server_cmd_path(),
            socket_strategy: Default::default(),
            runtime_strategy: Default::default(),
            watch_timeout: Default::default(),
            limit_requets: default_limit_requests(),
            ping: default_enable_endpoint(),
            resolver: default_enable_endpoint(),
            action: default_enable_endpoint(),
        }
    }

    pub fn cyclone_cmd_path(&self) -> &str {
        match self {
            CycloneConfig::LocalUds {
                cyclone_cmd_path, ..
            } => cyclone_cmd_path,
            CycloneConfig::LocalHttp {
                cyclone_cmd_path, ..
            } => cyclone_cmd_path,
        }
    }

    pub fn set_cyclone_cmd_path(&mut self, value: String) {
        match self {
            CycloneConfig::LocalUds {
                cyclone_cmd_path, ..
            } => *cyclone_cmd_path = value,
            CycloneConfig::LocalHttp {
                cyclone_cmd_path, ..
            } => *cyclone_cmd_path = value,
        };
    }

    pub fn cyclone_decryption_key_path(&self) -> &str {
        match self {
            CycloneConfig::LocalUds {
                cyclone_decryption_key_path,
                ..
            } => cyclone_decryption_key_path,
            CycloneConfig::LocalHttp {
                cyclone_decryption_key_path,
                ..
            } => cyclone_decryption_key_path,
        }
    }

    pub fn set_cyclone_decryption_key_path(&mut self, value: String) {
        match self {
            CycloneConfig::LocalUds {
                cyclone_decryption_key_path,
                ..
            } => *cyclone_decryption_key_path = value,
            CycloneConfig::LocalHttp {
                cyclone_decryption_key_path,
                ..
            } => *cyclone_decryption_key_path = value,
        };
    }

    pub fn lang_server_cmd_path(&self) -> &str {
        match self {
            CycloneConfig::LocalUds {
                lang_server_cmd_path,
                ..
            } => lang_server_cmd_path,
            CycloneConfig::LocalHttp {
                lang_server_cmd_path,
                ..
            } => lang_server_cmd_path,
        }
    }

    pub fn set_lang_server_cmd_path(&mut self, value: String) {
        match self {
            CycloneConfig::LocalUds {
                lang_server_cmd_path,
                ..
            } => *lang_server_cmd_path = value,
            CycloneConfig::LocalHttp {
                lang_server_cmd_path,
                ..
            } => *lang_server_cmd_path = value,
        };
    }

    pub fn set_limit_requests(&mut self, value: impl Into<Option<u32>>) {
        match self {
            CycloneConfig::LocalUds { limit_requets, .. } => *limit_requets = value.into(),
            CycloneConfig::LocalHttp { limit_requets, .. } => *limit_requets = value.into(),
        };
    }

    pub fn set_ping(&mut self, value: bool) {
        match self {
            CycloneConfig::LocalUds { ping, .. } => *ping = value,
            CycloneConfig::LocalHttp { ping, .. } => *ping = value,
        };
    }

    pub fn set_resolver(&mut self, value: bool) {
        match self {
            CycloneConfig::LocalUds { resolver, .. } => *resolver = value,
            CycloneConfig::LocalHttp { resolver, .. } => *resolver = value,
        };
    }

    pub fn set_action(&mut self, value: bool) {
        match self {
            CycloneConfig::LocalUds { action, .. } => *action = value,
            CycloneConfig::LocalHttp { action, .. } => *action = value,
        };
    }
}

impl Default for CycloneConfig {
    fn default() -> Self {
        Self::default_local_uds()
    }
}

impl TryFrom<CycloneConfig> for CycloneSpec {
    type Error = ConfigError;

    fn try_from(value: CycloneConfig) -> std::result::Result<Self, Self::Error> {
        match value {
            CycloneConfig::LocalUds {
                cyclone_cmd_path,
                cyclone_decryption_key_path,
                lang_server_cmd_path,
                socket_strategy,
                runtime_strategy,
                watch_timeout,
                limit_requets,
                ping,
                resolver,
                action,
            } => {
                let mut builder = LocalUdsInstance::spec();
                builder
                    .try_cyclone_cmd_path(cyclone_cmd_path)
                    .map_err(ConfigError::cyclone_spec_build)?;
                builder.cyclone_decryption_key_path(cyclone_decryption_key_path);
                builder
                    .try_lang_server_cmd_path(lang_server_cmd_path)
                    .map_err(ConfigError::cyclone_spec_build)?;
                builder.socket_strategy(socket_strategy);
                builder.runtime_strategy(runtime_strategy);
                if let Some(watch_timeout) = watch_timeout {
                    builder.watch_timeout(watch_timeout);
                }
                builder.limit_requests(limit_requets);
                if ping {
                    builder.ping();
                }
                if resolver {
                    builder.resolver();
                }
                if action {
                    builder.action();
                }

                Ok(Self::LocalUds(
                    builder.build().map_err(ConfigError::cyclone_spec_build)?,
                ))
            }
            CycloneConfig::LocalHttp {
                cyclone_cmd_path,
                cyclone_decryption_key_path,
                lang_server_cmd_path,
                socket_strategy,
                watch_timeout,
                limit_requets,
                ping,
                resolver,
                action,
            } => {
                let mut builder = LocalHttpInstance::spec();
                builder
                    .try_cyclone_cmd_path(cyclone_cmd_path)
                    .map_err(ConfigError::cyclone_spec_build)?;
                builder.cyclone_decryption_key_path(cyclone_decryption_key_path);
                builder
                    .try_lang_server_cmd_path(lang_server_cmd_path)
                    .map_err(ConfigError::cyclone_spec_build)?;
                builder.socket_strategy(socket_strategy);
                if let Some(watch_timeout) = watch_timeout {
                    builder.watch_timeout(watch_timeout);
                }
                builder.limit_requests(limit_requets);
                if ping {
                    builder.ping();
                }
                if resolver {
                    builder.resolver();
                }
                if action {
                    builder.action();
                }

                Ok(Self::LocalHttp(
                    builder.build().map_err(ConfigError::cyclone_spec_build)?,
                ))
            }
        }
    }
}

fn default_cyclone_cmd_path() -> String {
    "/usr/local/bin/cyclone".to_string()
}

fn default_cyclone_decryption_key_path() -> String {
    "/run/cyclone/decryption.key".to_string()
}

fn default_lang_server_cmd_path() -> String {
    "/usr/local/bin/lang-js".to_string()
}

fn default_limit_requests() -> Option<u32> {
    Some(1)
}

fn default_enable_endpoint() -> bool {
    true
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
    let resources = Buck2Resources::read().map_err(ConfigError::cyclone_spec_build)?;

    let cyclone_cmd_path = resources
        .get_ends_with("cyclone")
        .map_err(ConfigError::cyclone_spec_build)?
        .to_string_lossy()
        .to_string();
    let cyclone_decryption_key_path = resources
        .get_ends_with("dev.decryption.key")
        .map_err(ConfigError::cyclone_spec_build)?
        .to_string_lossy()
        .to_string();
    let lang_server_cmd_path = resources
        .get_ends_with("lang-js")
        .map_err(ConfigError::cyclone_spec_build)?
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_cmd_path = cyclone_cmd_path.as_str(),
        cyclone_decryption_key_path = cyclone_decryption_key_path.as_str(),
        lang_server_cmd_path = lang_server_cmd_path.as_str(),
        "detected development run",
    );

    config.cyclone.set_cyclone_cmd_path(cyclone_cmd_path);
    config
        .cyclone
        .set_cyclone_decryption_key_path(cyclone_decryption_key_path);
    config
        .cyclone
        .set_lang_server_cmd_path(lang_server_cmd_path);

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    let cyclone_cmd_path = Path::new(&dir)
        .join("../../target/debug/cyclone")
        .canonicalize()
        .expect("failed to canonicalize local dev build of <root>/target/debug/cyclone")
        .to_string_lossy()
        .to_string();
    let cyclone_decryption_key_path = Path::new(&dir)
        .join("../../lib/cyclone-server/src/dev.decryption.key")
        .canonicalize()
        .expect(
            "failed to canonicalize local key at <root>/lib/cyclone-server/src/dev.decryption.key",
        )
        .to_string_lossy()
        .to_string();
    let lang_server_cmd_path = Path::new(&dir)
        .join("../../bin/lang-js/target/lang-js")
        .canonicalize()
        .expect("failed to canonicalize local dev build of <root>/bin/lang-js/target/lang-js")
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_cmd_path = cyclone_cmd_path.as_str(),
        cyclone_decryption_key_path = cyclone_decryption_key_path.as_str(),
        lang_server_cmd_path = lang_server_cmd_path.as_str(),
        "detected development run",
    );

    config.cyclone.set_cyclone_cmd_path(cyclone_cmd_path);
    config
        .cyclone
        .set_cyclone_decryption_key_path(cyclone_decryption_key_path);
    config
        .cyclone
        .set_lang_server_cmd_path(lang_server_cmd_path);

    Ok(())
}
