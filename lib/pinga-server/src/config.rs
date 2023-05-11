use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_settings::{CanonicalFile, CanonicalFileError};
use telemetry::prelude::*;
use thiserror::Error;

pub use dal::CycloneKeyPair;
pub use si_settings::{StandardConfig, StandardConfigFile};
use ulid::Ulid;

const DEFAULT_CONCURRENCY_LIMIT: usize = 50;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error(transparent)]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
}

impl ConfigError {
    fn development(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::Development(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    cyclone_encryption_key_path: CanonicalFile,

    #[builder(default = "default_concurrency_limit()")]
    concurrency: usize,

    #[builder(default = "random_instance_id()")]
    instance_id: String,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets a reference to the config's pg pool.
    #[must_use]
    pub fn pg_pool(&self) -> &PgPoolConfig {
        &self.pg_pool
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

    /// Gets a reference to the config's cyclone public key path.
    #[must_use]
    pub fn cyclone_encryption_key_path(&self) -> &Path {
        self.cyclone_encryption_key_path.as_path()
    }

    /// Gets the config's concurrency limit.
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default = "default_cyclone_encryption_key_path")]
    cyclone_encryption_key_path: String,
    #[serde(default = "default_concurrency_limit")]
    concurrency_limit: usize,
    #[serde(default = "random_instance_id")]
    instance_id: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            nats: Default::default(),
            cyclone_encryption_key_path: default_cyclone_encryption_key_path(),
            concurrency_limit: default_concurrency_limit(),
            instance_id: random_instance_id(),
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
        config.pg_pool(value.pg);
        config.nats(value.nats);
        config.cyclone_encryption_key_path(value.cyclone_encryption_key_path.try_into()?);
        config.instance_id(value.instance_id);
        config.build().map_err(Into::into)
    }
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}

fn default_cyclone_encryption_key_path() -> String {
    "/run/pinga/cyclone_encryption.key".to_string()
}

fn default_concurrency_limit() -> usize {
    DEFAULT_CONCURRENCY_LIMIT
}

fn detect_and_configure_development(
    config: &mut ConfigFile,
) -> std::result::Result<(), ConfigError> {
    if std::env::var("BUCK_RUN_BUILD_ID").is_ok() {
        buck2_development(config)
    } else if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        cargo_development(dir, config)
    } else {
        Ok(())
    }
}

fn buck2_development(config: &mut ConfigFile) -> std::result::Result<(), ConfigError> {
    let resources = Buck2Resources::read()?;

    let cyclone_encryption_key_path = resources
        .get("bin/pinga/dev.encryption.key")?
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        "detected development run",
    );

    config.cyclone_encryption_key_path = cyclone_encryption_key_path;

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> std::result::Result<(), ConfigError> {
    let cyclone_encryption_key_path = Path::new(&dir)
        .join("../../lib/cyclone-server/src/dev.encryption.key")
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        "detected development run",
    );

    config.cyclone_encryption_key_path = cyclone_encryption_key_path;

    Ok(())
}

// TODO(fnichol): extract Buck2 resource code into small common crate
#[derive(Debug, Error)]
enum Buck2ResourcesError {
    #[error("failed canonicalize path: `{path}`")]
    Canonicalize {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("failed to look up our own executable path")]
    NoCurrentExe { source: std::io::Error },
    #[error("executable doesn't have a filename: `{executable_path}`")]
    NoFileName { executable_path: PathBuf },
    #[error("failed to find parent directory of executable: `{executable_path}`")]
    NoParentDir { executable_path: PathBuf },
    #[error("no resource named `{name}` found in manifest file: `{manifest_path}`")]
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

// TODO(fnichol): extract Buck2 resource code into small common crate
struct Buck2Resources {
    inner: HashMap<String, PathBuf>,
    parent_dir: PathBuf,
    manifest_path: PathBuf,
}

impl Buck2Resources {
    fn read() -> std::result::Result<Self, ConfigError> {
        let executable_path = std::env::current_exe().map_err(|source| {
            ConfigError::development(Buck2ResourcesError::NoCurrentExe { source })
        })?;
        let parent_dir = match executable_path.parent() {
            Some(p) => p,
            None => {
                return Err(ConfigError::development(Buck2ResourcesError::NoParentDir {
                    executable_path,
                }))
            }
        };
        let file_name = match executable_path.file_name() {
            Some(f) => f,
            None => {
                return Err(ConfigError::development(Buck2ResourcesError::NoFileName {
                    executable_path,
                }))
            }
        };
        let manifest_path =
            parent_dir.join(format!("{}.resources.json", file_name.to_string_lossy()));
        let manifest_string = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(source) => {
                return Err(ConfigError::development(Buck2ResourcesError::ReadFailed {
                    manifest_path,
                    source,
                }))
            }
        };
        let inner: HashMap<String, PathBuf> =
            serde_json::from_str(&manifest_string).map_err(|source| {
                ConfigError::development(Buck2ResourcesError::ParsingFailed {
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
            ConfigError::development(Buck2ResourcesError::NoSuchResource {
                name: name.as_ref().to_string(),
                manifest_path: self.manifest_path.clone(),
            })
        })?;

        let path = self.parent_dir.join(rel_path);
        let path = path.canonicalize().map_err(|source| {
            ConfigError::development(Buck2ResourcesError::Canonicalize { source, path })
        })?;

        Ok(path)
    }
}
