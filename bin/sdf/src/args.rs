use std::path::PathBuf;

use clap::{builder::PossibleValuesParser, ArgAction, Parser};
use sdf_server::{Config, ConfigError, ConfigFile, MigrationMode, StandardConfigFile};

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
    /// Multiple -v options increase verbosity. The maximum is 4.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// PostgreSQL connection pool dbname [example: myapp]
    #[arg(long)]
    pub(crate) pg_dbname: Option<String>,

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

    /// NATS connection URL [example: demo.nats.io]
    #[arg(long)]
    pub(crate) nats_url: Option<String>,

    /// NATS credentials string
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) nats_creds: Option<String>,

    /// NATS credentials file
    #[arg(long)]
    pub(crate) nats_creds_file: Option<String>,

    /// Database migration mode on startup
    #[arg(long, value_parser = PossibleValuesParser::new(MigrationMode::variants()))]
    pub(crate) migration_mode: Option<String>,

    /// Disable OpenTelemetry on startup
    #[arg(long)]
    pub(crate) disable_opentelemetry: bool,

    /// Cyclone encryption key file location [default: /run/sdf/cyclone_encryption.key]
    #[arg(long)]
    pub(crate) cyclone_encryption_key_path: Option<String>,

    /// Generates cyclone secret key file (does not run server)
    ///
    /// Will error if set when `generate_cyclone_public_key_path` is not set
    #[arg(long, requires = "generate_cyclone_public_key_path")]
    pub(crate) generate_cyclone_secret_key_path: Option<PathBuf>,

    /// Generates cyclone public key file (does not run server)
    ///
    /// Will error if set when `generate_cyclone_secret_key_path` is not set
    #[arg(long, requires = "generate_cyclone_secret_key_path")]
    pub(crate) generate_cyclone_public_key_path: Option<PathBuf>,

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
            if let Some(migration_mode) = args.migration_mode {
                config_map.set("migration_mode", migration_mode);
            }
            if let Some(url) = args.nats_url {
                config_map.set("nats.url", url);
            }
            if let Some(creds) = args.nats_creds {
                config_map.set("nats.creds", creds);
            }
            if let Some(creds_file) = args.nats_creds_file {
                config_map.set("nats.creds_file", creds_file);
            }
            if let Some(cyclone_encyption_key_path) = args.cyclone_encryption_key_path {
                config_map.set("cyclone_encryption_key_path", cyclone_encyption_key_path);
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
