use std::path::PathBuf;

use clap::{
    ArgAction,
    Parser,
    builder::{
        EnumValueParser,
        PossibleValuesParser,
    },
};
use sdf_server::{
    Config,
    ConfigError,
    ConfigFile,
    ConfigMap,
    FeatureFlag,
    MigrationMode,
    ParameterProvider,
    StandardConfigFile,
    WorkspacePermissions,
    WorkspacePermissionsMode,
};
use si_service::prelude::*;

pub const NAME: &str = "sdf";

include!(concat!(env!("OUT_DIR"), "/git_metadata.rs"));

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative API service.
///
/// Super Dimension Fortress (SDF) is the central and primary API surface which handles front end
/// calls and dispatches function executions, among other great things.
#[derive(Parser, Debug)]
#[command(name = NAME, version = VERSION, max_term_width = 100)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 6.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Disables ANSI coloring in log output, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long = "no-color",
        default_value = "false",
        env = "SI_NO_COLOR",
        hide_env_values = true,
        conflicts_with = "force_color"
    )]
    pub(crate) no_color: bool,

    /// Forces ANSI coloring, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long = "force-color",
        default_value = "false",
        env = "SI_FORCE_COLOR",
        hide_env_values = true,
        conflicts_with = "no_color"
    )]
    pub(crate) force_color: bool,

    /// Prints telemetry logging as JSON lines.
    ///
    /// For more details, visit: <https://jsonlines.org/>.
    #[arg(
        long = "log-json",
        default_value = "false",
        env = "SI_LOG_JSON",
        hide_env_values = true
    )]
    pub(crate) log_json: bool,

    /// Additionally appends logging to rolling files under the given directory.
    #[arg(
        long = "log-file-directory",
        env = "SI_LOG_FILE_DIRECTORY",
        hide_env_values = true
    )]
    pub(crate) log_file_directory: Option<PathBuf>,

    /// Enables support for emitting async runtime data to `tokio-console`.
    ///
    /// For more details, visit: <https://github.com/tokio-rs/console>.
    #[arg(
        long = "tokio-console",
        default_value = "false",
        env = "SI_TOKIO_CONSOLE",
        hide_env_values = true
    )]
    pub(crate) tokio_console: bool,

    /// PostgreSQL connection pool dbname [example: myapp]
    #[arg(long)]
    pub(crate) pg_dbname: Option<String>,

    /// PostgreSQL connection pool dbname for layer_db [example: melons]
    #[arg(long)]
    pub(crate) layer_db_pg_dbname: Option<String>,

    /// PostgreSQL connection pool hostname [example: prod.db.example.com]
    #[arg(long)]
    pub(crate) pg_hostname: Option<String>,

    /// PostgreSQL connection pool max size [example: 8]
    #[arg(long)]
    pub(crate) pg_pool_max_size: Option<u32>,

    /// PostgreSQL connection pool port [example: 5432]
    #[arg(long)]
    pub(crate) pg_port: Option<u16>,

    /// PostgreSQL connection pool user [example: dbuser]
    #[arg(long)]
    pub(crate) pg_user: Option<String>,

    /// PostgreSQL connection certification path
    #[arg(long)]
    pub(crate) pg_cert_path: Option<PathBuf>,

    /// PostgreSQL connection certification base64 string
    #[arg(long)]
    pub(crate) pg_cert_base64: Option<SensitiveString>,

    /// PostgreSQL connection recycling method [default: Fast]
    #[arg(long)]
    pub(crate) pg_recycling_method: Option<String>,

    /// NATS connection URL [example: demo.nats.io]
    #[arg(long)]
    pub(crate) nats_url: Option<String>,

    /// NATS credentials string
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) nats_creds: Option<SensitiveString>,

    /// NATS credentials file
    #[arg(long)]
    pub(crate) nats_creds_path: Option<PathBuf>,

    /// Database migration mode on startup
    #[arg(long, value_parser = PossibleValuesParser::new(MigrationMode::variants()))]
    pub(crate) migration_mode: Option<String>,

    /// Timestamp cutoff for layer cache backfill (RFC 3339 format, e.g., 2025-01-14T15:30:45.123456Z)
    /// Only backfill data created before this timestamp. Copy from checkpoint logs.
    #[arg(long, env = "SI_BACKFILL_CUTOFF_TIMESTAMP")]
    pub(crate) backfill_cutoff_timestamp: Option<String>,

    /// Comma-separated cache types to backfill (REQUIRED when using BackfillLayerCache mode)
    /// Specify only caches that need historical data migrated to S3.
    /// Valid types: cas,workspace_snapshot,encrypted_secret,rebase_batch,
    ///              change_batch,split_snapshot_subgraph,split_snapshot_supergraph,
    ///              split_snapshot_rebase_batch
    /// Example: --backfill-cache-types="cas,workspace_snapshot,encrypted_secret"
    #[arg(long, env = "SI_BACKFILL_CACHE_TYPES")]
    pub(crate) backfill_cache_types: Option<String>,

    /// Batch size for PostgreSQL queries during backfill (layer cache and func runs)
    #[arg(long, default_value = "1000", env = "SI_BACKFILL_KEY_BATCH_SIZE")]
    pub(crate) backfill_key_batch_size: usize,

    /// Checkpoint interval in seconds for progress logging
    #[arg(
        long,
        default_value = "30",
        env = "SI_BACKFILL_CHECKPOINT_INTERVAL_SECS"
    )]
    pub(crate) backfill_checkpoint_interval_secs: u64,

    /// Maximum concurrent uploads per cache type during backfill
    #[arg(long, default_value = "5", env = "SI_BACKFILL_MAX_CONCURRENT_UPLOADS")]
    pub(crate) backfill_max_concurrent_uploads: usize,

    /// Cutoff ID for func runs backfill (optional, start from a specific func run ID)
    #[arg(long, env = "SI_BACKFILL_FUNC_RUNS_CUTOFF_ID")]
    pub(crate) backfill_func_runs_cutoff_id: Option<String>,

    /// Cutoff ID for func run logs backfill (optional, start from a specific func run log ID)
    #[arg(long, env = "SI_BACKFILL_FUNC_RUN_LOGS_CUTOFF_ID")]
    pub(crate) backfill_func_run_logs_cutoff_id: Option<String>,

    /// Veritech encryption key file location [default: /run/sdf/veritech_encryption.key]
    #[arg(long)]
    pub(crate) veritech_encryption_key_path: Option<PathBuf>,

    /// Veritech encryption key file contents
    #[arg(long)]
    pub(crate) veritech_encryption_key_base64: Option<SensitiveString>,

    /// Symmetric crypto active key as base64 string
    #[arg(long)]
    pub(crate) symmetric_crypto_active_key_base64: Option<SensitiveString>,

    /// jwt public signing key as a base64 string
    #[arg(long)]
    pub(crate) jwt_public_signing_key_base64: Option<String>,

    /// jwt public signing key algorithm (ES256 or RS256)
    #[arg(long)]
    pub(crate) jwt_public_signing_key_algo: Option<String>,

    /// jwt secondary public signing key as a base64 string
    #[arg(long)]
    pub(crate) jwt_secondary_public_signing_key_base64: Option<String>,

    /// jwt secondary public signing key algorithm (ES256 or RS256)
    #[arg(long)]
    pub(crate) jwt_secondary_public_signing_key_algo: Option<String>,

    /// The path at which the layer db cache is created/used on disk [e.g. /banana/]
    #[arg(long)]
    pub(crate) layer_db_disk_path: Option<String>,

    /// The time to idle for items in the layercache
    #[arg(long)]
    pub(crate) layer_db_seconds_to_idle: Option<u64>,

    /// Generates Veritech secret key file (does not run server)
    ///
    /// Will error if set when `generate_veritech_public_key_path` is not set
    #[arg(
        long,
        requires = "generate_veritech_public_key_path",
        conflicts_with = "generate_symmetric_key_path"
    )]
    pub(crate) generate_veritech_secret_key_path: Option<PathBuf>,

    /// Generates Veritech public key file (does not run server)
    ///
    /// Will error if set when `generate_veritech_secret_key_path` is not set
    #[arg(
        long,
        requires = "generate_veritech_secret_key_path",
        conflicts_with = "generate_symmetric_key_path"
    )]
    pub(crate) generate_veritech_public_key_path: Option<PathBuf>,

    /// Generates symmetric key (does not run server)
    ///
    /// Will error if set when Veritech key generation flags are set
    #[arg(
        long,
        group = "symmetric",
        conflicts_with_all = [
            "generate_veritech_secret_key_path",
            "generate_veritech_public_key_path",
        ]
    )]
    pub(crate) generate_symmetric_key_path: Option<PathBuf>,

    /// Location on disk of available packages
    pub(crate) pkgs_path: Option<String>,

    /// The base URL for the module-index API server
    #[arg(long, env = "SI_MODULE_INDEX_URL")]
    pub(crate) module_index_url: Option<String>,

    /// Allow for Posthog feature flags in SDF
    #[arg(
        long,
        env = "SI_FEATURES",
        value_parser = EnumValueParser::<FeatureFlag>::new(),
        value_delimiter = ',',
        rename_all = "snake_case",
    )]
    pub(crate) features: Vec<FeatureFlag>,

    /// Create Workspace Permissions Mode [default: closed]
    #[arg(long, env = "SI_CREATE_WORKSPACE_PERMISSIONS", value_parser = PossibleValuesParser::new(WorkspacePermissionsMode::variants()))]
    pub(crate) create_workspace_permissions: Option<String>,

    /// List of emails that can create workspaces
    #[arg(
        long,
        env = "SI_CREATE_WORKSPACE_ALLOWLIST",
        value_delimiter = ',',
        rename_all = "snake_case"
    )]
    pub(crate) create_workspace_allowlist: Vec<WorkspacePermissions>,

    /// Override for the auth api url
    #[arg(long, env = "SI_AUTH_API_URL")]
    pub(crate) auth_api_url: Option<String>,

    /// Override for the openai API key
    #[arg(long, env = "SI_OPENAI_API_KEY")]
    // TODO This should be a SensitiveString but if it is, it gets passed as "..."
    // pub(crate) openai_api_key: Option<SensitiveString>,
    pub(crate) openai_api_key: Option<String>,

    /// Override for the openai API base URL
    #[arg(long, env = "SI_OPENAI_API_BASE")]
    pub(crate) openai_api_base: Option<String>,

    /// Override for the openai org id
    #[arg(long, env = "SI_OPENAI_ORG_ID")]
    pub(crate) openai_org_id: Option<String>,

    /// Override for the openai project id
    #[arg(long, env = "SI_OPENAI_PROJECT_ID")]
    pub(crate) openai_project_id: Option<String>,

    /// Directory to read prompts from (localdev)
    #[arg(long, env = "SI_ASSET_SPRAYER_PROMPTS_DIR")]
    pub(crate) asset_sprayer_prompts_dir: Option<String>,

    /// Instance ID [example: 01GWEAANW5BVFK5KDRVS6DEY0F"]
    ///
    /// And instance ID is used when tracking the execution of jobs in a way that can be traced
    /// back to an instance of this service.
    #[arg(long)]
    pub(crate) instance_id: Option<String>,
}

impl Args {
    pub fn generating_veritech_key_pair(&self) -> Option<(PathBuf, PathBuf)> {
        match (
            self.generate_veritech_secret_key_path.as_ref(),
            self.generate_veritech_public_key_path.as_ref(),
        ) {
            (Some(secret_key_path), Some(public_key_path)) => {
                Some((secret_key_path.clone(), public_key_path.clone()))
            }
            _ => None,
        }
    }

    pub fn generating_symmetric_key(&self) -> Option<PathBuf> {
        self.generate_symmetric_key_path.clone()
    }
}
fn build_config_map(args: Args, config_map: &mut ConfigMap) -> &ConfigMap {
    if let Some(instance_id) = args.instance_id {
        config_map.set("instance_id", instance_id);
    }

    if let Some(dbname) = args.pg_dbname {
        config_map.set("pg.dbname", dbname);
    }

    if let Some(layer_cache_pg_dbname) = args.layer_db_pg_dbname {
        config_map.set(
            "layer_db_config.pg_pool_config.dbname",
            layer_cache_pg_dbname,
        );
    }
    if let Some(hostname) = args.pg_hostname {
        config_map.set("pg.hostname", hostname.clone());
        config_map.set("layer_db_config.pg_pool_config.hostname", hostname);
    }
    if let Some(pool_max_size) = args.pg_pool_max_size {
        config_map.set("pg.pool_max_size", i64::from(pool_max_size));
        config_map.set(
            "layer_db_config.pg_pool_config.pool_max_size",
            i64::from(pool_max_size),
        );
    }
    if let Some(port) = args.pg_port {
        config_map.set("pg.port", i64::from(port));
        config_map.set("layer_db_config.pg_pool_config.port", i64::from(port));
    }
    if let Some(user) = args.pg_user {
        config_map.set("pg.user", user.clone());
        config_map.set("layer_db_config.pg_pool_config.user", user);
    }
    if let Some(cert_path) = args.pg_cert_path {
        config_map.set("pg.certificate_path", cert_path.display().to_string());
        config_map.set(
            "layer_db_config.pg_pool_config.certificate_path",
            cert_path.display().to_string(),
        );
    }
    if let Some(recycling_method) = args.pg_recycling_method {
        config_map.set("pg.recycling_method", recycling_method.clone());
        config_map.set(
            "layer_db_config.pg_pool_config.recycling_method",
            recycling_method,
        );
    }
    if let Some(cert) = args.pg_cert_base64 {
        config_map.set("pg.certificate_base64", cert.to_string());
        config_map.set(
            "layer_db_config.pg_pool_config.certificate_base64",
            cert.to_string(),
        );
    }
    if let Some(migration_mode) = args.migration_mode {
        config_map.set("migration_mode", migration_mode);
    }
    if let Some(url) = args.nats_url {
        config_map.set("nats.url", url.clone());
        config_map.set("layer_db_config.nats_config.url", url);
    }
    if let Some(creds) = args.nats_creds {
        config_map.set("nats.creds", creds.to_string());
        config_map.set("layer_db_config.nats_config.creds", creds.to_string());
    }
    if let Some(creds_file) = args.nats_creds_path {
        config_map.set("nats.creds_file", creds_file.display().to_string());
        config_map.set(
            "layer_db_config.nats_config.creds_file",
            creds_file.display().to_string(),
        );
    }
    if let Some(veritech_encryption_key_file) = args.veritech_encryption_key_path {
        config_map.set(
            "crypto.encryption_key_file",
            veritech_encryption_key_file.display().to_string(),
        );
    }
    if let Some(veritech_encryption_key_base64) = args.veritech_encryption_key_base64 {
        config_map.set(
            "crypto.encryption_key_base64",
            veritech_encryption_key_base64.to_string(),
        );
    }
    if let Some(base64) = args.symmetric_crypto_active_key_base64 {
        config_map.set(
            "symmetric_crypto_service.active_key_base64",
            base64.to_string(),
        );
    }

    if let Some(jwt) = args.jwt_public_signing_key_base64 {
        config_map.set("jwt_signing_public_key.key_base64", jwt);
    }
    if let Some(algo) = args.jwt_public_signing_key_algo {
        config_map.set("jwt_signing_public_key.algo", algo);
    }

    if let Some(jwt) = args.jwt_secondary_public_signing_key_base64 {
        config_map.set("jwt_secondary_signing_public_key.key_base64", jwt);
    }
    if let Some(algo) = args.jwt_secondary_public_signing_key_algo {
        config_map.set("jwt_secondary_signing_public_key.algo", algo);
    }

    if let Some(layer_cache_disk_path) = args.layer_db_disk_path {
        config_map.set("layer_db_config.disk_path", layer_cache_disk_path);
    }
    if let Some(pkgs_path) = args.pkgs_path {
        config_map.set("pkgs_path", pkgs_path);
    }
    if let Some(module_index_url) = args.module_index_url {
        config_map.set("module_index_url", module_index_url);
    }

    if let Some(auth_api_url) = args.auth_api_url {
        config_map.set("auth_api_url", auth_api_url);
    }

    if let Some(openai_api_key) = args.openai_api_key {
        config_map.set("openai.api_key", openai_api_key.to_string());
    }
    if let Some(openai_api_base) = args.openai_api_base {
        config_map.set("openai.api_base", openai_api_base);
    }
    if let Some(openai_org_id) = args.openai_org_id {
        config_map.set("openai.org_id", openai_org_id);
    }
    if let Some(openai_project_id) = args.openai_project_id {
        config_map.set("openai.project_id", openai_project_id);
    }

    if let Some(asset_sprayer_prompts_dir) = args.asset_sprayer_prompts_dir {
        config_map.set(
            "asset_sprayer.prompts_dir",
            asset_sprayer_prompts_dir.to_string(),
        );
    }

    config_map.set("boot_feature_flags", args.features);

    if let Some(create_workspace_permissions) = args.create_workspace_permissions {
        config_map.set("create_workspace_permissions", create_workspace_permissions);
    }

    if !args.create_workspace_allowlist.is_empty() {
        config_map.set(
            "create_workspace_allowlist",
            args.create_workspace_allowlist,
        );
    }

    if let Some(backfill_cutoff_timestamp) = args.backfill_cutoff_timestamp {
        config_map.set("backfill_cutoff_timestamp", backfill_cutoff_timestamp);
    }
    if let Some(backfill_cache_types) = args.backfill_cache_types {
        config_map.set("backfill_cache_types", backfill_cache_types);
    }
    config_map.set(
        "backfill_key_batch_size",
        i64::try_from(args.backfill_key_batch_size).unwrap_or(1000),
    );
    config_map.set(
        "backfill_checkpoint_interval_secs",
        i64::try_from(args.backfill_checkpoint_interval_secs).unwrap_or(30),
    );
    config_map.set(
        "backfill_max_concurrent_uploads",
        i64::try_from(args.backfill_max_concurrent_uploads).unwrap_or(5),
    );

    if let Some(backfill_func_runs_cutoff_id) = args.backfill_func_runs_cutoff_id {
        config_map.set("backfill_func_runs_cutoff_id", backfill_func_runs_cutoff_id);
    }

    if let Some(backfill_func_run_logs_cutoff_id) = args.backfill_func_run_logs_cutoff_id {
        config_map.set(
            "backfill_func_run_logs_cutoff_id",
            backfill_func_run_logs_cutoff_id,
        );
    }

    config_map.set("nats.connection_name", NAME);
    config_map.set("pg.application_name", NAME);
    config_map.set("layer_db_config.pg_pool_config.application_name", NAME);
    config_map.set("layer_db_config.nats_config.connection_name", NAME);
    config_map
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, |config_map| {
            build_config_map(args, config_map);
        })?
        .try_into()
    }
}

pub async fn load_config_with_provider<P>(
    args: Args,
    provider: Option<P>,
) -> Result<Config, ConfigError>
where
    P: ParameterProvider + 'static,
{
    ConfigFile::layered_load_with_provider::<_, P>(NAME, provider, move |config_map| {
        build_config_map(args, config_map);
    })
    .await?
    .try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_command() {
        use clap::CommandFactory;
        Args::command().debug_assert()
    }
}
