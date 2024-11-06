use asset_sprayer::config::{AssetSprayerConfig, SIOpenAIConfig};
use dal::jwt_key::JwtConfig;
use si_crypto::VeritechCryptoConfig;
use si_data_spicedb::SpiceDbConfig;
use si_layer_cache::{db::LayerDbConfig, error::LayerDbError};
use std::collections::HashSet;
use std::{
    env,
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
};
use ulid::Ulid;

use buck2_resources::Buck2Resources;
use dal::feature_flags::FeatureFlag;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_crypto::{SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_posthog::PosthogConfig;
use si_std::{CanonicalFile, CanonicalFileError};
use telemetry::prelude::*;
use thiserror::Error;

pub use axum_util::config::*;
pub use dal::MigrationMode;
pub use si_settings::{StandardConfig, StandardConfigFile};

const DEFAULT_MODULE_INDEX_URL: &str = "https://module-index.systeminit.com";
const DEFAULT_AUTH_API_URL: &str = "https://auth-api.systeminit.com";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error(transparent)]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error(transparent)]
    LayerCache(#[from] LayerDbError),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
}

impl ConfigError {
    fn development(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::Development(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default)]
    incoming_stream: IncomingStream,

    #[builder(default)]
    pg_pool: PgPoolConfig,

    #[builder(default = "default_module_index_url()")]
    module_index_url: String,

    #[builder(default = "default_auth_api_url()")]
    auth_api_url: String,

    #[builder(default)]
    openai: SIOpenAIConfig,

    #[builder(default)]
    asset_sprayer: AssetSprayerConfig,

    #[builder(default)]
    nats: NatsConfig,

    #[builder(default)]
    posthog: PosthogConfig,

    #[builder(default)]
    symmetric_crypto_service: SymmetricCryptoServiceConfig,

    #[builder(default)]
    migration_mode: MigrationMode,

    #[builder(default)]
    crypto: VeritechCryptoConfig,

    #[builder(default)]
    jwt_signing_public_key: JwtConfig,

    #[builder(default = "default_layer_db_config()")]
    layer_db_config: LayerDbConfig,

    #[builder(default)]
    spicedb: SpiceDbConfig,

    pkgs_path: CanonicalFile,

    boot_feature_flags: HashSet<FeatureFlag>,

    create_workspace_permissions: WorkspacePermissionsMode,

    create_workspace_allowlist: Vec<WorkspacePermissions>,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }

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

    /// Gets a reference to the config's jwt signing public config.
    #[must_use]
    pub fn jwt_signing_public_key(&self) -> &JwtConfig {
        &self.jwt_signing_public_key
    }

    /// Gets a reference to the config's cyclone public key path.
    #[must_use]
    pub fn crypto(&self) -> &VeritechCryptoConfig {
        &self.crypto
    }

    /// Gets a reference to the config's pkg path.
    #[must_use]
    pub fn pkgs_path(&self) -> &Path {
        self.pkgs_path.as_path()
    }

    /// Gets a reference to the config's posthog config.
    #[must_use]
    pub fn posthog(&self) -> &PosthogConfig {
        &self.posthog
    }

    pub fn symmetric_crypto_service(&self) -> &SymmetricCryptoServiceConfig {
        &self.symmetric_crypto_service
    }

    /// URL to the module index service
    #[must_use]
    pub fn module_index_url(&self) -> &str {
        &self.module_index_url
    }

    /// URL to the auth API
    #[must_use]
    pub fn auth_api_url(&self) -> &str {
        &self.auth_api_url
    }

    /// OpenAI API configuration
    pub fn openai(&self) -> &SIOpenAIConfig {
        &self.openai
    }

    /// Prompts directory for the asset sprayer
    pub fn asset_sprayer(&self) -> &AssetSprayerConfig {
        &self.asset_sprayer
    }

    /// Feature flags defined at boot time, via config files or the FEATURES env variable
    #[must_use]
    pub fn boot_feature_flags(&self) -> &HashSet<FeatureFlag> {
        &self.boot_feature_flags
    }

    #[must_use]
    pub fn layer_db_config(&self) -> &LayerDbConfig {
        &self.layer_db_config
    }

    // The Create Workspace Permissions Mode should be set via an env variable or it will default to Closed
    pub fn create_workspace_permissions(&self) -> &WorkspacePermissionsMode {
        &self.create_workspace_permissions
    }

    // This Allowlist is a list of email addresses only used in WorkspacePermissionsMode::Allowlist
    pub fn create_workspace_allowlist(&self) -> &Vec<WorkspacePermissions> {
        &self.create_workspace_allowlist
    }

    /// Gets a referece to the config's spicedb config
    #[must_use]
    pub fn spicedb(&self) -> &SpiceDbConfig {
        &self.spicedb
    }
}

impl ConfigBuilder {
    pub fn http_socket(&mut self, socket_addrs: impl ToSocketAddrs) -> Result<&mut Self> {
        Ok(self.incoming_stream(IncomingStream::tcp_socket(socket_addrs)?))
    }

    pub fn unix_domain_socket(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.incoming_stream(IncomingStream::unix_domain_socket(path))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default = "random_instance_id")]
    instance_id: String,
    #[serde(default)]
    pub pg: PgPoolConfig,
    #[serde(default)]
    pub nats: NatsConfig,
    #[serde(default)]
    pub migration_mode: MigrationMode,
    #[serde(default)]
    pub jwt_signing_public_key: JwtConfig,
    #[serde(default)]
    pub crypto: VeritechCryptoConfig,
    #[serde(default = "default_pkgs_path")]
    pub pkgs_path: String,
    #[serde(default)]
    pub posthog: PosthogConfig,
    #[serde(default = "default_layer_db_config")]
    layer_db_config: LayerDbConfig,
    #[serde(default)]
    pub module_index_url: String,
    #[serde(default = "default_auth_api_url")]
    pub auth_api_url: String,
    #[serde(default)]
    pub openai: SIOpenAIConfig,
    #[serde(default)]
    pub asset_sprayer: AssetSprayerConfig,
    #[serde(default = "default_symmetric_crypto_config")]
    symmetric_crypto_service: SymmetricCryptoServiceConfigFile,
    #[serde(default)]
    boot_feature_flags: Vec<FeatureFlag>,
    #[serde(default)]
    create_workspace_permissions: WorkspacePermissionsMode,
    #[serde(default)]
    create_workspace_allowlist: Vec<WorkspacePermissions>,
    #[serde(default)]
    spicedb: SpiceDbConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            instance_id: random_instance_id(),
            pg: Default::default(),
            nats: Default::default(),
            migration_mode: Default::default(),
            jwt_signing_public_key: Default::default(),
            crypto: Default::default(),
            pkgs_path: default_pkgs_path(),
            posthog: Default::default(),
            layer_db_config: default_layer_db_config(),
            module_index_url: default_module_index_url(),
            auth_api_url: default_auth_api_url(),
            openai: Default::default(),
            asset_sprayer: Default::default(),
            symmetric_crypto_service: default_symmetric_crypto_config(),
            boot_feature_flags: Default::default(),
            create_workspace_permissions: Default::default(),
            create_workspace_allowlist: Default::default(),
            spicedb: Default::default(),
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

        Ok(Config {
            instance_id: value.instance_id,
            pg_pool: value.pg,
            nats: value.nats,
            incoming_stream: IncomingStream::default(), // TODO this OK?
            migration_mode: value.migration_mode,
            jwt_signing_public_key: value.jwt_signing_public_key,
            crypto: value.crypto,
            pkgs_path: value.pkgs_path.try_into()?,
            posthog: value.posthog,
            module_index_url: value.module_index_url,
            auth_api_url: value.auth_api_url,
            openai: value.openai,
            asset_sprayer: value.asset_sprayer,
            symmetric_crypto_service: value.symmetric_crypto_service.try_into()?,
            layer_db_config: value.layer_db_config,
            boot_feature_flags: value.boot_feature_flags.into_iter().collect::<HashSet<_>>(),
            create_workspace_permissions: value.create_workspace_permissions,
            create_workspace_allowlist: value.create_workspace_allowlist,
            spicedb: value.spicedb,
        })
    }
}

#[remain::sorted]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingStream {
    TcpSocket(SocketAddr),
    UnixDomainSocket(PathBuf),
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self::TcpSocket(SocketAddr::from(([0, 0, 0, 0], 5156)))
    }
}

impl IncomingStream {
    pub fn tcp_socket(socket_addrs: impl ToSocketAddrs) -> Result<Self> {
        let socket_addr = socket_addrs
            .to_socket_addrs()
            .map_err(ConfigError::SocketAddrResolve)?
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::TcpSocket(socket_addr))
    }

    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}

fn default_pkgs_path() -> String {
    "/run/sdf/pkgs/".to_string()
}

fn default_symmetric_crypto_config() -> SymmetricCryptoServiceConfigFile {
    SymmetricCryptoServiceConfigFile {
        active_key: None,
        active_key_base64: None,
        extra_keys: vec![],
    }
}

fn default_module_index_url() -> String {
    DEFAULT_MODULE_INDEX_URL.into()
}

fn default_auth_api_url() -> String {
    DEFAULT_AUTH_API_URL.into()
}

fn default_layer_db_config() -> LayerDbConfig {
    LayerDbConfig::default()
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

    #[allow(clippy::disallowed_methods)] // Used in development with a local auth services
    // Note(victor): If the user has set a custom auth ip url via env variable we assume dev mode
    let jwt_signing_public_key_path = if env::var("SI_AUTH_API_URL").is_ok() {
        resources
            .get_ends_with("dev.jwt_signing_public_key.pem")
            .map_err(ConfigError::development)?
            .to_string_lossy()
            .to_string()
    } else {
        resources
            .get_ends_with("prod.jwt_signing_public_key.pem")
            .map_err(ConfigError::development)?
            .to_string_lossy()
            .to_string()
    };
    let veritech_encryption_key_path = resources
        .get_ends_with("dev.encryption.key")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();
    let symmetric_crypto_service_key = resources
        .get_ends_with("dev.donkey.key")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();
    let postgres_cert = resources
        .get_ends_with("dev.postgres.root.crt")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();
    let pkgs_path = resources
        .get_ends_with("pkgs_path")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        veritech_encryption_key_path = veritech_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_cert = postgres_cert.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key = JwtConfig {
        key_file: Some(jwt_signing_public_key_path.try_into()?),
        key_base64: None,
    };
    config.crypto.encryption_key_file = veritech_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    config.layer_db_config.pg_pool_config.certificate_path =
        Some(postgres_cert.clone().try_into()?);
    config.pkgs_path = pkgs_path;
    config.layer_db_config.pg_pool_config.dbname = "si_layer_db".to_string();
    config.spicedb.enabled = true;

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    #[allow(clippy::disallowed_methods)] // Used in development with a local auth services
    // Note(victor): If the user has set a custom auth ip url via env variable we assume dev mode
    let jwt_signing_public_key_path = if env::var("SI_AUTH_API_URL").is_ok() {
        Path::new(&dir)
            .join("../../config/keys/dev.jwt_signing_public_key.pem")
            .to_string_lossy()
            .to_string()
    } else {
        Path::new(&dir)
            .join("../../config/keys/prod.jwt_signing_public_key.pem")
            .to_string_lossy()
            .to_string()
    };
    let veritech_encryption_key_path = Path::new(&dir)
        .join("../../lib/veritech-server/src/dev.encryption.key")
        .to_string_lossy()
        .to_string();
    let symmetric_crypto_service_key = Path::new(&dir)
        .join("../../lib/dal/dev.donkey.key")
        .to_string_lossy()
        .to_string();
    let postgres_cert = Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string();
    let pkgs_path = Path::new(&dir)
        .join("../../pkgs/")
        .to_string_lossy()
        .to_string();

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        veritech_encryption_key_path = veritech_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_cert = postgres_cert.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key = JwtConfig {
        key_file: Some(jwt_signing_public_key_path.try_into()?),
        key_base64: None,
    };
    config.crypto.encryption_key_file = veritech_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    config.layer_db_config.pg_pool_config.certificate_path =
        Some(postgres_cert.clone().try_into()?);
    config.layer_db_config.pg_pool_config.dbname = "si_layer_db".to_string();
    config.pkgs_path = pkgs_path;
    config.spicedb.enabled = true;

    Ok(())
}
