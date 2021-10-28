use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use si_data::{NatsConfig, PgPoolConfig};
use strum_macros::{Display, EnumString, EnumVariantNames};
use thiserror::Error;

pub use si_settings::{StandardConfig, StandardConfigFile};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
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
    #[builder(default = "IncomingStream::default()")]
    incoming_stream: IncomingStream,

    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = "MigrationMode::default()")]
    migration_mode: MigrationMode,

    #[builder(default = "JwtSigningKey::default()")]
    jwt_signing_key: JwtSigningKey,
}

pub type JwtSigningKey = dal::JwtEncrypt;

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets a reference to the config's incoming stream.
    #[must_use]
    pub fn incoming_stream(&self) -> &IncomingStream {
        &self.incoming_stream
    }

    /// Gets a reference to the config's pg pool.
    #[must_use]
    pub fn pg_pool(&self) -> &PgPoolConfig {
        &self.pg_pool
    }

    /// Gets a reference to the config's migration mode.
    #[must_use]
    pub fn migration_mode(&self) -> &MigrationMode {
        &self.migration_mode
    }

    /// Gets a reference to the config's nats.
    #[must_use]
    pub fn nats(&self) -> &NatsConfig {
        &self.nats
    }

    /// Gets a reference to the config's nats.
    #[must_use]
    pub fn jwt_signing_key(&self) -> &JwtSigningKey {
        &self.jwt_signing_key
    }
}

impl ConfigBuilder {
    pub fn http_socket(&mut self, socket_addrs: impl ToSocketAddrs) -> Result<&mut Self> {
        Ok(self.incoming_stream(IncomingStream::http_socket(socket_addrs)?))
    }

    pub fn unix_domain_socket(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.incoming_stream(IncomingStream::unix_domain_socket(path))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    pg: PgPoolConfig,
    nats: NatsConfig,
    migration_mode: MigrationMode,
    jwt_signing_key: JwtSigningKey,
}

impl StandardConfigFile for ConfigFile {
    type Error = ConfigError;
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(value: ConfigFile) -> Result<Self> {
        let mut config = Config::builder();
        config.pg_pool(value.pg);
        config.nats(value.nats);
        config.migration_mode(value.migration_mode);
        config.jwt_signing_key(value.jwt_signing_key);
        config.build().map_err(Into::into)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingStream {
    HTTPSocket(SocketAddr),
    UnixDomainSocket(PathBuf),
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self::HTTPSocket(SocketAddr::from(([0, 0, 0, 0], 5156)))
    }
}

impl IncomingStream {
    pub fn http_socket(socket_addrs: impl ToSocketAddrs) -> Result<Self> {
        let socket_addr = socket_addrs
            .to_socket_addrs()
            .map_err(ConfigError::SocketAddrResolve)?
            .into_iter()
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::HTTPSocket(socket_addr))
    }

    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}

#[derive(
    Clone,
    Debug,
    DeserializeFromStr,
    Display,
    EnumString,
    EnumVariantNames,
    Eq,
    PartialEq,
    SerializeDisplay,
)]
#[strum(serialize_all = "camelCase")]
pub enum MigrationMode {
    Run,
    RunAndQuit,
    Skip,
}

impl Default for MigrationMode {
    fn default() -> Self {
        Self::Run
    }
}

impl MigrationMode {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <MigrationMode as strum::VariantNames>::VARIANTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod migration_mode {
        use super::*;

        #[test]
        fn display() {
            assert_eq!("run", MigrationMode::Run.to_string());
            assert_eq!("runAndQuit", MigrationMode::RunAndQuit.to_string());
            assert_eq!("skip", MigrationMode::Skip.to_string());
        }

        #[test]
        fn from_str() {
            assert_eq!(MigrationMode::Run, "run".parse().expect("failed to parse"));
            assert_eq!(
                MigrationMode::RunAndQuit,
                "runAndQuit".parse().expect("failed to parse")
            );
            assert_eq!(
                MigrationMode::Skip,
                "skip".parse().expect("failed to parse")
            );
        }

        #[test]
        fn deserialize() {
            #[derive(Deserialize)]
            struct Test {
                mode: MigrationMode,
            }

            let test: Test =
                serde_json::from_str(r#"{"mode":"runAndQuit"}"#).expect("failed to deserialize");
            assert_eq!(MigrationMode::RunAndQuit, test.mode);
        }

        #[test]
        fn serialize() {
            #[derive(Serialize)]
            struct Test {
                mode: MigrationMode,
            }

            let test = serde_json::to_string(&Test {
                mode: MigrationMode::RunAndQuit,
            })
            .expect("failed to serialize");
            assert_eq!(r#"{"mode":"runAndQuit"}"#, test);
        }
    }
}
