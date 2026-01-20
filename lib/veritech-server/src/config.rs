use std::{
    env,
    net::{
        SocketAddr,
        ToSocketAddrs,
    },
    path::{
        Path,
        PathBuf,
    },
    time::Duration,
};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use si_crypto::VeritechCryptoConfig;
use si_data_nats::NatsConfig;
use si_pool_noodle::{
    Instance,
    instance::cyclone::{
        LocalHttpInstance,
        LocalHttpInstanceSpec,
        LocalHttpSocketStrategy,
        LocalUdsInstance,
        LocalUdsInstanceSpec,
        LocalUdsRuntimeStrategy,
        LocalUdsSocketStrategy,
    },
};
use si_service_endpoints::ServiceEndpointsConfig;
pub use si_settings::{
    StandardConfig,
    StandardConfigFile,
};
use si_std::CanonicalFileError;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

const DEFAULT_POOL_SIZE: u32 = 50;

const DEFAULT_POOL_GET_RETRY_LIMIT: u32 = 30;
const DEFAULT_CONSUMER_MAX_DELIVER: i64 = 10;

const DEFAULT_CYCLONE_CLIENT_EXECUTION_TIMEOUT_SECS: u64 = 60 * 35;
const DEFAULT_CYCLONE_CLIENT_EXECUTION_TIMEOUT: Duration =
    Duration::from_secs(DEFAULT_CYCLONE_CLIENT_EXECUTION_TIMEOUT_SECS);

const DEFAULT_HEARTBEAT_APP_SLEEP_SECONDS: u64 = 15;
const DEFAULT_HEARTBEAT_APP_PUBLISH_TIMEOUT_SECONDS: u64 = 10;

const DEFAULT_HEARTBEAT_APP_SLEEP_DURATION: Duration =
    Duration::from_secs(DEFAULT_HEARTBEAT_APP_SLEEP_SECONDS);
const DEFAULT_HEARTBEAT_APP_PUBLISH_TIMEOUT_DURATION: Duration =
    Duration::from_secs(DEFAULT_HEARTBEAT_APP_PUBLISH_TIMEOUT_SECONDS);

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("builder error: {0}")]
    Builder(#[from] ConfigBuilderError),
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("cyclone spec build error")]
    CycloneSpecBuild(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error("settings error: {0}")]
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

#[derive(Debug, Builder, Serialize)]
pub struct Config {
    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[serde(skip_serializing)]
    cyclone_spec: CycloneSpec,

    #[builder(default = "VeritechCryptoConfig::default()")]
    crypto: VeritechCryptoConfig,

    #[builder(default = "default_healthcheck_pool()")]
    healthcheck_pool: bool,

    #[builder(default = "default_cyclone_client_execution_timeout()")]
    cyclone_client_execution_timeout: Duration,

    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "default_heartbeat_app()")]
    heartbeat_app: bool,

    #[builder(default = "default_heartbeat_app_sleep_duration()")]
    heartbeat_app_sleep_duration: Duration,

    #[builder(default = "default_heartbeat_app_publish_timeout_duration()")]
    heartbeat_app_publish_timeout_duration: Duration,

    #[builder(default = "default_service_endpoints_config()")]
    service_endpoints: ServiceEndpointsConfig,

    #[builder(default = "default_pool_get_retry_limit()")]
    pool_get_retry_limit: u32,

    #[builder(default = "default_consumer_max_deliver()")]
    consumer_max_deliver: i64,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
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

    /// Gets a reference to the config's cyclone public key path.
    pub fn crypto(&self) -> &VeritechCryptoConfig {
        &self.crypto
    }

    /// Gets the config's healthcheck value.
    pub fn healthcheck_pool(&self) -> bool {
        self.healthcheck_pool
    }

    /// Consumes into a [`CycloneSpec`].
    pub fn into_cyclone_spec(self) -> CycloneSpec {
        self.cyclone_spec
    }

    /// Gets the config's cyclone client execution timeout.
    pub fn cyclone_client_execution_timeout(&self) -> Duration {
        self.cyclone_client_execution_timeout
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }

    /// Indicates if the heartbeat app will be enabled.
    pub fn heartbeat_app(&self) -> bool {
        self.heartbeat_app
    }

    /// Gets the config's sleep duration.
    pub fn heartbeat_app_sleep_duration(&self) -> Duration {
        self.heartbeat_app_sleep_duration
    }

    /// Gets the config's publish timeout duration.
    pub fn heartbeat_app_publish_timeout_duration(&self) -> Duration {
        self.heartbeat_app_publish_timeout_duration
    }

    /// Gets a reference to the config's service endpoints configuration.
    #[must_use]
    pub fn service_endpoints(&self) -> &ServiceEndpointsConfig {
        &self.service_endpoints
    }

    /// Gets the config's pool get retry limit.
    pub fn pool_get_retry_limit(&self) -> u32 {
        self.pool_get_retry_limit
    }

    /// Gets the config's consumer max deliver.
    pub fn consumer_max_deliver(&self) -> i64 {
        self.consumer_max_deliver
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub nats: NatsConfig,
    pub cyclone: CycloneConfig,
    #[serde(default)]
    pub crypto: VeritechCryptoConfig,
    #[serde(default = "default_healthcheck_pool")]
    healthcheck_pool: bool,
    #[serde(default = "default_cyclone_client_execution_timeout_secs")]
    cyclone_client_execution_timeout_secs: u64,
    #[serde(default = "random_instance_id")]
    instance_id: String,
    #[serde(default = "default_heartbeat_app")]
    pub heartbeat_app: bool,
    #[serde(default = "default_heartbeat_app_sleep_secs")]
    heartbeat_app_sleep_secs: u64,
    #[serde(default = "default_heartbeat_app_publish_timeout_secs")]
    heartbeat_app_publish_timeout_secs: u64,
    #[serde(default = "default_service_endpoints_config")]
    service_endpoints: ServiceEndpointsConfig,
    #[serde(default = "default_pool_get_retry_limit")]
    pool_get_retry_limit: u32,
    #[serde(default = "default_consumer_max_deliver")]
    consumer_max_deliver: i64,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self::default_local_uds()
    }
}

impl ConfigFile {
    pub fn default_local_http() -> Self {
        Self {
            nats: Default::default(),
            cyclone: CycloneConfig::default_local_http(),
            crypto: Default::default(),
            healthcheck_pool: default_healthcheck_pool(),
            cyclone_client_execution_timeout_secs: default_cyclone_client_execution_timeout_secs(),
            instance_id: random_instance_id(),
            heartbeat_app: default_heartbeat_app(),
            heartbeat_app_sleep_secs: default_heartbeat_app_sleep_secs(),
            heartbeat_app_publish_timeout_secs: default_heartbeat_app_publish_timeout_secs(),
            service_endpoints: default_service_endpoints_config(),
            pool_get_retry_limit: default_pool_get_retry_limit(),
            consumer_max_deliver: default_consumer_max_deliver(),
        }
    }

    pub fn default_local_uds() -> Self {
        Self {
            nats: Default::default(),
            cyclone: CycloneConfig::default_local_uds(),
            crypto: Default::default(),
            healthcheck_pool: default_healthcheck_pool(),
            cyclone_client_execution_timeout_secs: default_cyclone_client_execution_timeout_secs(),
            instance_id: random_instance_id(),
            heartbeat_app: default_heartbeat_app(),
            heartbeat_app_sleep_secs: default_heartbeat_app_sleep_secs(),
            heartbeat_app_publish_timeout_secs: default_heartbeat_app_publish_timeout_secs(),
            service_endpoints: default_service_endpoints_config(),
            pool_get_retry_limit: default_pool_get_retry_limit(),
            consumer_max_deliver: default_consumer_max_deliver(),
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
        config.crypto(value.crypto);
        config.cyclone_client_execution_timeout(Duration::from_secs(
            value.cyclone_client_execution_timeout_secs,
        ));
        config.instance_id(value.instance_id);

        config.heartbeat_app(value.heartbeat_app);
        config.heartbeat_app_sleep_duration(Duration::from_secs(value.heartbeat_app_sleep_secs));
        config.heartbeat_app_publish_timeout_duration(Duration::from_secs(
            value.heartbeat_app_publish_timeout_secs,
        ));
        config.service_endpoints(value.service_endpoints);
        config.pool_get_retry_limit(value.pool_get_retry_limit);
        config.consumer_max_deliver(value.consumer_max_deliver);

        config.build().map_err(Into::into)
    }
}

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum CycloneSpec {
    LocalHttp(LocalHttpInstanceSpec),
    LocalUds(LocalUdsInstanceSpec),
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
        #[serde(default = "default_lang_server_cmd_path")]
        lang_server_cmd_path: String,
        #[serde(default)]
        lang_server_function_timeout: Option<usize>,
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
        #[serde(default = "default_lang_server_cmd_path")]
        lang_server_cmd_path: String,
        #[serde(default)]
        lang_server_function_timeout: Option<usize>,
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
        #[serde(default)]
        pool_size: u32,
        #[serde(default)]
        connect_timeout: u64,
        #[serde(default = "default_create_firecracker_setup_scripts")]
        create_firecracker_setup_scripts: bool,
    },
}

impl CycloneConfig {
    pub fn default_local_http() -> Self {
        Self::LocalHttp {
            cyclone_cmd_path: default_cyclone_cmd_path(),
            lang_server_cmd_path: default_lang_server_cmd_path(),
            lang_server_function_timeout: Default::default(),
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
            lang_server_cmd_path: default_lang_server_cmd_path(),
            lang_server_function_timeout: Default::default(),
            socket_strategy: Default::default(),
            runtime_strategy: default_runtime_strategy(),
            watch_timeout: Default::default(),
            limit_requets: default_limit_requests(),
            ping: default_enable_endpoint(),
            resolver: default_enable_endpoint(),
            action: default_enable_endpoint(),
            pool_size: default_pool_size(),
            connect_timeout: default_connect_timeout(),
            create_firecracker_setup_scripts: default_create_firecracker_setup_scripts(),
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

    pub fn set_pool_size(&mut self, value: u32) {
        if let CycloneConfig::LocalUds { pool_size, .. } = self {
            *pool_size = value
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
                lang_server_cmd_path,
                lang_server_function_timeout,
                socket_strategy,
                runtime_strategy,
                watch_timeout,
                limit_requets,
                ping,
                resolver,
                action,
                pool_size,
                connect_timeout,
                create_firecracker_setup_scripts,
            } => {
                let mut builder = LocalUdsInstance::spec();

                //we only need these if running local process. Maybe the builder should handle
                //this?
                if matches!(runtime_strategy, LocalUdsRuntimeStrategy::LocalProcess) {
                    builder
                        .try_cyclone_cmd_path(cyclone_cmd_path)
                        .map_err(ConfigError::cyclone_spec_build)?;
                    builder
                        .try_lang_server_cmd_path(lang_server_cmd_path)
                        .map_err(ConfigError::cyclone_spec_build)?;
                }
                builder.lang_server_function_timeout(lang_server_function_timeout);

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
                builder.pool_size(pool_size);
                builder.connect_timeout(connect_timeout);
                builder.create_firecracker_setup_scripts(create_firecracker_setup_scripts);

                Ok(Self::LocalUds(
                    builder.build().map_err(ConfigError::cyclone_spec_build)?,
                ))
            }
            CycloneConfig::LocalHttp {
                cyclone_cmd_path,
                lang_server_cmd_path,
                lang_server_function_timeout,
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

                builder
                    .try_lang_server_cmd_path(lang_server_cmd_path)
                    .map_err(ConfigError::cyclone_spec_build)?;
                builder.lang_server_function_timeout(lang_server_function_timeout);

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

fn random_instance_id() -> String {
    Ulid::new().to_string()
}

fn default_cyclone_cmd_path() -> String {
    "/usr/local/bin/cyclone".to_string()
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

fn default_runtime_strategy() -> LocalUdsRuntimeStrategy {
    LocalUdsRuntimeStrategy::default()
}

fn default_pool_size() -> u32 {
    DEFAULT_POOL_SIZE
}

fn default_connect_timeout() -> u64 {
    10
}

fn default_healthcheck_pool() -> bool {
    true
}

fn default_cyclone_client_execution_timeout() -> Duration {
    DEFAULT_CYCLONE_CLIENT_EXECUTION_TIMEOUT
}

fn default_cyclone_client_execution_timeout_secs() -> u64 {
    DEFAULT_CYCLONE_CLIENT_EXECUTION_TIMEOUT_SECS
}

fn default_heartbeat_app() -> bool {
    true
}

fn default_heartbeat_app_sleep_duration() -> Duration {
    DEFAULT_HEARTBEAT_APP_SLEEP_DURATION
}

fn default_heartbeat_app_sleep_secs() -> u64 {
    DEFAULT_HEARTBEAT_APP_SLEEP_SECONDS
}

fn default_heartbeat_app_publish_timeout_duration() -> Duration {
    DEFAULT_HEARTBEAT_APP_PUBLISH_TIMEOUT_DURATION
}

fn default_heartbeat_app_publish_timeout_secs() -> u64 {
    DEFAULT_HEARTBEAT_APP_PUBLISH_TIMEOUT_SECONDS
}

fn default_create_firecracker_setup_scripts() -> bool {
    true
}

fn default_service_endpoints_config() -> ServiceEndpointsConfig {
    ServiceEndpointsConfig::new(0)
}

fn default_pool_get_retry_limit() -> u32 {
    DEFAULT_POOL_GET_RETRY_LIMIT
}

fn default_consumer_max_deliver() -> i64 {
    DEFAULT_CONSUMER_MAX_DELIVER
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
    let decryption_key_path = resources
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
        decryption_key_path = decryption_key_path.as_str(),
        lang_server_cmd_path = lang_server_cmd_path,
        "detected development run",
    );

    config.cyclone.set_cyclone_cmd_path(cyclone_cmd_path);
    config.crypto.decryption_key_file = decryption_key_path.parse().ok();
    config
        .cyclone
        .set_lang_server_cmd_path(lang_server_cmd_path.to_string());

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    let cyclone_cmd_path = Path::new(&dir)
        .join("../../target/debug/cyclone")
        .canonicalize()
        .expect("failed to canonicalize local dev build of <root>/target/debug/cyclone")
        .to_string_lossy()
        .to_string();
    let decryption_key_path = Path::new(&dir)
        .join("../../lib/veritech-server/src/dev.decryption.key")
        .canonicalize()
        .expect(
            "failed to canonicalize local key at <root>/lib/veritech-server/src/dev.decryption.key",
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
        decryption_key_path = decryption_key_path.as_str(),
        lang_server_cmd_path = lang_server_cmd_path.as_str(),
        "detected development run",
    );

    config.cyclone.set_cyclone_cmd_path(cyclone_cmd_path);
    config.crypto.decryption_key_file = decryption_key_path.parse().ok();
    config
        .cyclone
        .set_lang_server_cmd_path(lang_server_cmd_path);

    Ok(())
}
