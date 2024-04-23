use std::path::PathBuf;

use clap::{builder::PossibleValuesParser, ArgAction, Parser};
use sdf_server::{Config, ConfigError, ConfigFile, MigrationMode, StandardConfigFile};
use si_std::SensitiveString;

const NAME: &str = "sdf";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative API service.
///
/// Super Dimension Fortress (SDF) is the central and primary API surface which handles front end
/// calls and dispatches function executions, among other great things.
#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
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

    /// PostgreSQL connection pool dbname [example: myapp]
    #[arg(long)]
    pub(crate) pg_dbname: Option<String>,

    /// PostgreSQL connection pool dbname for layer_db [example: melons]
    #[arg(long)]
    pub(crate) layer_cache_pg_dbname: Option<String>,

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

    /// Cyclone encryption key file location [default: /run/sdf/cyclone_encryption.key]
    #[arg(long)]
    pub(crate) cyclone_encryption_key_path: Option<PathBuf>,

    /// Cyclone encryption key file contents
    #[arg(long)]
    pub(crate) cyclone_encryption_key_base64: Option<SensitiveString>,

    /// Cyclone secret key as base64 string
    #[arg(long)]
    pub(crate) cyclone_secret_key_base64: Option<SensitiveString>,

    /// jwt public signing key as a base64 string
    #[arg(long)]
    pub(crate) jwt_public_signing_key_base64: Option<SensitiveString>,

    /// The path at which the layer db cache is created/used on disk [e.g. /banana/]
    #[arg(long)]
    pub(crate) layer_cache_disk_path: Option<String>,

    /// Generates cyclone secret key file (does not run server)
    ///
    /// Will error if set when `generate_cyclone_public_key_path` is not set
    #[arg(
        long,
        requires = "generate_cyclone_public_key_path",
        conflicts_with = "generate_symmetric_key_path"
    )]
    pub(crate) generate_cyclone_secret_key_path: Option<PathBuf>,

    /// Generates cyclone public key file (does not run server)
    ///
    /// Will error if set when `generate_cyclone_secret_key_path` is not set
    #[arg(
        long,
        requires = "generate_cyclone_secret_key_path",
        conflicts_with = "generate_symmetric_key_path"
    )]
    pub(crate) generate_cyclone_public_key_path: Option<PathBuf>,

    /// Generates symmetric key (does not run server)
    ///
    /// Will error if set when cyclone key generation flags are set
    #[arg(
        long,
        group = "symmetric",
        conflicts_with_all = [
            "generate_cyclone_secret_key_path",
            "generate_cyclone_public_key_path",
        ]
    )]
    pub(crate) generate_symmetric_key_path: Option<PathBuf>,

    /// Location on disk of available packages
    pub(crate) pkgs_path: Option<String>,

    /// The base URL for the module-index API server
    #[arg(long, env = "SI_MODULE_INDEX_URL")]
    pub(crate) module_index_url: Option<String>,
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, |config_map| {
            if let Some(dbname) = args.pg_dbname {
                config_map.set("pg.dbname", dbname);
            }
            if let Some(hostname) = args.pg_hostname {
                config_map.set("pg.hostname", hostname);
            }
            if let Some(pool_max_size) = args.pg_pool_max_size {
                config_map.set("pg.pool_max_size", i64::from(pool_max_size));
            }
            if let Some(port) = args.pg_port {
                config_map.set("pg.port", i64::from(port));
            }
            if let Some(user) = args.pg_user {
                config_map.set("pg.user", user);
            }
            if let Some(cert_path) = args.pg_cert_path {
                config_map.set("pg.certificate_path", cert_path.display().to_string());
            }
            if let Some(cert) = args.pg_cert_base64 {
                config_map.set("pg.certificate_base64", cert.to_string());
            }
            if let Some(layer_cache_pg_dbname) = args.layer_cache_pg_dbname {
                config_map.set("layer_cache_pg_dbname", layer_cache_pg_dbname);
            }
            if let Some(migration_mode) = args.migration_mode {
                config_map.set("migration_mode", migration_mode);
            }
            if let Some(url) = args.nats_url {
                config_map.set("nats.url", url);
            }
            if let Some(creds) = args.nats_creds {
                config_map.set("nats.creds", creds.to_string());
            }
            if let Some(creds_file) = args.nats_creds_path {
                config_map.set("nats.creds_file", creds_file.display().to_string());
            }
            if let Some(cyclone_encryption_key_file) = args.cyclone_encryption_key_path {
                config_map.set(
                    "crypto.encryption_key_file",
                    cyclone_encryption_key_file.display().to_string(),
                );
            }
            if let Some(cyclone_encryption_key_base64) = args.cyclone_encryption_key_base64 {
                config_map.set(
                    "crypto.encryption_key_base64",
                    cyclone_encryption_key_base64.to_string(),
                );
            }
            if let Some(secret_string) = args.cyclone_secret_key_base64 {
                config_map.set(
                    "symmetric_crypto_service.active_key_base64",
                    secret_string.to_string(),
                );
            }
            if let Some(jwt) = args.jwt_public_signing_key_base64 {
                config_map.set("jwt_signing_public_key.key_base64", jwt.to_string());
            }
            if let Some(layer_cache_disk_path) = args.layer_cache_disk_path {
                config_map.set("layer_cache_disk_path", layer_cache_disk_path);
            }
            if let Some(pkgs_path) = args.pkgs_path {
                config_map.set("pkgs_path", pkgs_path);
            }
            if let Some(module_index_url) = args.module_index_url {
                config_map.set("module_index_url", module_index_url);
            }

            config_map.set("nats.connection_name", NAME);
            config_map.set("pg.application_name", NAME);
        })?
        .try_into()
    }
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
