use std::path::PathBuf;

use clap::{
    ArgAction,
    Parser,
};
use rebaser_server::{
    Config,
    ConfigError,
    ConfigFile,
    ConfigMap,
    ParameterProvider,
    StandardConfigFile,
};
use si_service::prelude::*;

pub const NAME: &str = "rebaser";

include!(concat!(env!("OUT_DIR"), "/git_metadata.rs"));

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

#[derive(Parser, Debug)]
#[command(name = NAME, version = VERSION, max_term_width = 100)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 4.
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

    /// Disable OpenTelemetry on startup
    #[arg(long)]
    pub(crate) disable_opentelemetry: bool,

    /// Veritech encryption key file location [default: /run/rebaser/veritech_encryption.key]
    #[arg(long)]
    pub(crate) veritech_encryption_key_path: Option<String>,

    /// Cyclone encryption key file contents as a base64 encoded string
    #[arg(long)]
    pub(crate) veritech_encryption_key_base64: Option<SensitiveString>,

    /// Symmetric crypto active key as base64 string
    #[arg(long)]
    pub(crate) symmetric_crypto_active_key_base64: Option<SensitiveString>,

    /// The number of concurrent jobs that can be processed [default: 10]
    #[arg(long)]
    pub(crate) concurrency: Option<u32>,

    /// The path at which the layer db cache is created/used on disk [e.g. /banana/]
    #[arg(long)]
    pub(crate) layer_db_disk_path: Option<String>,

    /// The time to idle for items in the layercache
    #[arg(long)]
    pub(crate) layer_db_seconds_to_idle: Option<u64>,

    /// Instance ID [example: 01GWEAANW5BVFK5KDRVS6DEY0F"]
    ///
    /// And instance ID is used when tracking the execution of jobs in a way that can be traced
    /// back to an instance of a Pinga service.
    #[arg(long)]
    pub(crate) instance_id: Option<String>,
}

fn build_config_map(args: Args, config_map: &mut ConfigMap) -> &ConfigMap {
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
    if let Some(cert) = args.pg_cert_base64 {
        config_map.set("pg.certificate_base64", cert.to_string());
        config_map.set(
            "layer_db_config.pg_pool_config.certificate_base64",
            cert.to_string(),
        );
    }
    if let Some(recycling_method) = args.pg_recycling_method {
        config_map.set("pg.recycling_method", recycling_method.clone());
        config_map.set(
            "layer_db_config.pg_pool_config.recycling_method",
            recycling_method,
        );
    }
    if let Some(url) = args.nats_url {
        config_map.set("nats.url", url.clone());
        config_map.set("layer_db_config.nats_config.url", url);
    }
    if let Some(creds) = args.nats_creds {
        config_map.set("nats.creds", creds.to_string());
        config_map.set("layer_db_config.nats_config.creds", creds.to_string());
    }
    if let Some(creds_path) = args.nats_creds_path {
        config_map.set("nats.creds_file", creds_path.display().to_string());
        config_map.set(
            "layer_db_config.nats_config.creds_file",
            creds_path.display().to_string(),
        );
    }
    if let Some(veritech_encryption_key_file) = args.veritech_encryption_key_path {
        config_map.set(
            "crypto.encryption_key_file",
            veritech_encryption_key_file.to_string(),
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
    if let Some(concurrency) = args.concurrency {
        config_map.set("concurrency_limit", i64::from(concurrency));
    }
    if let Some(layer_cache_disk_path) = args.layer_db_disk_path {
        config_map.set("layer_db_config.disk_path", layer_cache_disk_path);
    }
    if let Some(layer_cache_seconds_to_idle) = args.layer_db_seconds_to_idle {
        config_map.set(
            "layer_db_config.memory_cache_config.seconds_to_idle",
            layer_cache_seconds_to_idle,
        );
    }
    if let Some(instance_id) = args.instance_id {
        config_map.set("instance_id", instance_id);
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
