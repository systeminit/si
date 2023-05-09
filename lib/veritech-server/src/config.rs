use std::{
    collections::HashMap,
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
    time::Duration,
};

use deadpool_cyclone::{
    instance::cyclone::{
        LocalHttpInstance, LocalHttpInstanceSpec, LocalHttpSocketStrategy, LocalUdsInstance,
        LocalUdsInstanceSpec, LocalUdsSocketStrategy,
    },
    Instance,
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
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
    cyclone: CycloneConfig,
}

impl StandardConfigFile for ConfigFile {
    type Error = ConfigError;
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(value: ConfigFile) -> Result<Self> {
        let mut config = Config::builder();
        config.nats(value.nats);
        config.cyclone_spec(detect_and_configure_development_spec(value.cyclone)?);
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
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::HttpSocket(socket_addr))
    }
    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum CycloneConfig {
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
        watch_timeout: Option<Duration>,
        #[serde(default = "default_limit_requests")]
        limit_requets: Option<u32>,
        #[serde(default = "default_enable_endpoint")]
        ping: bool,
        #[serde(default = "default_enable_endpoint")]
        resolver: bool,
        #[serde(default = "default_enable_endpoint")]
        workflow: bool,
        #[serde(default = "default_enable_endpoint")]
        command: bool,
    },
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
        workflow: bool,
        #[serde(default = "default_enable_endpoint")]
        command: bool,
    },
}

impl CycloneConfig {
    fn set_cyclone_cmd_path(&mut self, value: String) {
        match self {
            CycloneConfig::LocalUds {
                cyclone_cmd_path, ..
            } => *cyclone_cmd_path = value,
            CycloneConfig::LocalHttp {
                cyclone_cmd_path, ..
            } => *cyclone_cmd_path = value,
        };
    }

    fn set_cyclone_decryption_key_path(&mut self, value: String) {
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

    fn set_lang_server_cmd_path(&mut self, value: String) {
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
}

impl Default for CycloneConfig {
    fn default() -> Self {
        Self::LocalUds {
            cyclone_cmd_path: default_cyclone_cmd_path(),
            cyclone_decryption_key_path: default_cyclone_decryption_key_path(),
            lang_server_cmd_path: default_lang_server_cmd_path(),
            socket_strategy: Default::default(),
            watch_timeout: Default::default(),
            limit_requets: default_limit_requests(),
            ping: default_enable_endpoint(),
            resolver: default_enable_endpoint(),
            workflow: default_enable_endpoint(),
            command: default_enable_endpoint(),
        }
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
                watch_timeout,
                limit_requets,
                ping,
                resolver,
                workflow,
                command,
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
                if workflow {
                    builder.workflow();
                }
                if command {
                    builder.command();
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
                workflow,
                command,
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
                if workflow {
                    builder.workflow();
                }
                if command {
                    builder.command();
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

fn detect_and_configure_development_spec(
    cyclone: CycloneConfig,
) -> std::result::Result<CycloneSpec, ConfigError> {
    if std::env::var("BUCK_RUN_BUILD_ID").is_ok() {
        buck2_development_cyclone_spec(cyclone)
    } else if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        cargo_development_cyclone_spec(dir, cyclone)
    } else {
        cyclone.try_into()
    }
}

fn buck2_development_cyclone_spec(
    mut cyclone: CycloneConfig,
) -> std::result::Result<CycloneSpec, ConfigError> {
    let resources = Buck2Resources::read()?;

    let cyclone_cmd_path = resources
        .get("bin/veritech/cyclone")?
        .canonicalize()
        .expect("failed to canonicalize local dev build of a `cyclone` binary")
        .to_string_lossy()
        .to_string();
    let cyclone_decryption_key_path = resources
        .get("bin/veritech/dev.decryption.key")?
        .canonicalize()
        .expect("failed to canonicalize local dev build of a `cyclone` binary")
        .to_string_lossy()
        .to_string();
    let lang_server_cmd_path = resources
        .get("bin/veritech/lang-js")?
        // TODO(fnichol): tweak build rule to produce binary as its output
        .join("lang-js")
        .canonicalize()
        .expect("failed to canonicalize local dev build of a `lang-js` binary")
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_cmd_path = cyclone_cmd_path.as_str(),
        cyclone_decryption_key_path = cyclone_decryption_key_path.as_str(),
        lang_server_cmd_path = lang_server_cmd_path.as_str(),
        "detected development run",
    );

    cyclone.set_cyclone_cmd_path(cyclone_cmd_path);
    cyclone.set_cyclone_decryption_key_path(cyclone_decryption_key_path);
    cyclone.set_lang_server_cmd_path(lang_server_cmd_path);

    cyclone.try_into()
}

fn cargo_development_cyclone_spec(
    dir: String,
    mut cyclone: CycloneConfig,
) -> std::result::Result<CycloneSpec, ConfigError> {
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

    cyclone.set_cyclone_cmd_path(cyclone_cmd_path);
    cyclone.set_cyclone_decryption_key_path(cyclone_decryption_key_path);
    cyclone.set_lang_server_cmd_path(lang_server_cmd_path);

    cyclone.try_into()
}

#[derive(Debug, Error)]
enum Buck2ResourcesError {
    #[error("Failed to look up our own executable path")]
    NoCurrentExe { source: std::io::Error },
    #[error("Executable doesn't have a filename: `{executable_path}`")]
    NoFileName { executable_path: PathBuf },
    #[error("Failed to find parent directory of executable: `{executable_path}`")]
    NoParentDir { executable_path: PathBuf },
    #[error("No resource named `{name}` found in manifest file: `{manifest_path}`")]
    NoSuchResource {
        name: String,
        manifest_path: PathBuf,
    },
    #[error("Failed to parse manifest file: `{manifest_path}`")]
    ParsingFailed {
        manifest_path: PathBuf,
        source: serde_json::Error,
    },
    #[error("Failed to read manifest file: `{manifest_path}`")]
    ReadFailed {
        manifest_path: PathBuf,
        source: std::io::Error,
    },
}

struct Buck2Resources {
    inner: HashMap<String, PathBuf>,
    parent_dir: PathBuf,
    manifest_path: PathBuf,
}

impl Buck2Resources {
    fn read() -> std::result::Result<Self, ConfigError> {
        let executable_path = std::env::current_exe().map_err(|source| {
            ConfigError::cyclone_spec_build(Buck2ResourcesError::NoCurrentExe { source })
        })?;
        let parent_dir = match executable_path.parent() {
            Some(p) => p,
            None => {
                return Err(ConfigError::cyclone_spec_build(
                    Buck2ResourcesError::NoParentDir { executable_path },
                ))
            }
        };
        let file_name = match executable_path.file_name() {
            Some(f) => f,
            None => {
                return Err(ConfigError::cyclone_spec_build(
                    Buck2ResourcesError::NoFileName { executable_path },
                ))
            }
        };
        let manifest_path =
            parent_dir.join(format!("{}.resources.json", file_name.to_string_lossy()));
        let manifest_string = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(source) => {
                return Err(ConfigError::cyclone_spec_build(
                    Buck2ResourcesError::ReadFailed {
                        manifest_path,
                        source,
                    },
                ))
            }
        };
        let inner: HashMap<String, PathBuf> =
            serde_json::from_str(&manifest_string).map_err(|source| {
                ConfigError::cyclone_spec_build(Buck2ResourcesError::ParsingFailed {
                    manifest_path: manifest_path.clone(),
                    source,
                })
            })?;

        Ok(Self {
            inner,
            parent_dir: parent_dir.to_path_buf(),
            manifest_path,
        })
    }

    fn get(&self, name: impl AsRef<str>) -> std::result::Result<PathBuf, ConfigError> {
        let rel_path = self.inner.get(name.as_ref()).ok_or_else(|| {
            ConfigError::cyclone_spec_build(Buck2ResourcesError::NoSuchResource {
                name: name.as_ref().to_string(),
                manifest_path: self.manifest_path.clone(),
            })
        })?;

        Ok(self.parent_dir.join(rel_path))
    }
}
