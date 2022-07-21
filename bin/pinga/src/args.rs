use std::path::PathBuf;

use clap::Parser;

use crate::config::{Config, ConfigError, ConfigFile, StandardConfigFile};
use dal::MigrationMode;

const NAME: &str = "pinga";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative API service.
///
/// Pinga queue executor system that handles whatever job comes from Faktory.
/// It means "drip" in portuguese and also is a name for Cachaça
#[derive(Parser, Debug)]
#[clap(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 4.
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub(crate) verbose: usize,

    /// PostgreSQL connection pool dbname [example: myapp]
    #[clap(long)]
    pub(crate) pg_dbname: Option<String>,

    /// PostgreSQL connection pool hostname [example: prod.db.example.com]
    #[clap(long)]
    pub(crate) pg_hostname: Option<String>,

    /// PostgreSQL connection pool max size [example: 8]
    #[clap(long)]
    pub(crate) pg_pool_max_size: Option<u32>,

    /// PostgreSQL connection pool port [example: 5432]
    #[clap(long)]
    pub(crate) pg_port: Option<u16>,

    /// PostgreSQL connection pool user [example: dbuser]
    #[clap(long)]
    pub(crate) pg_user: Option<String>,

    /// NATS connection URL [example: demo.nats.io]
    #[clap(long)]
    pub(crate) nats_url: Option<String>,

    /// Database migration mode on startup
    #[clap(long, possible_values = MigrationMode::variants())]
    pub(crate) migration_mode: Option<MigrationMode>,

    /// Disable OpenTelemetry on startup
    #[clap(long)]
    pub(crate) disable_opentelemetry: bool,

    /// Cyclone encryption key file location [default: /run/pinga/cyclone_encryption.key]
    #[clap(long)]
    pub(crate) cyclone_encryption_key_path: Option<String>,

    /// Generates cyclone secret key file (does not run server)
    /// Will error if set when `generate_cyclone_public_key_path` is not set
    #[clap(long, requires = "generate-cyclone-public-key-path")]
    pub(crate) generate_cyclone_secret_key_path: Option<PathBuf>,

    /// Generates cyclone public key file (does not run server)
    /// Will error if set when `generate_cyclone_secret_key_path` is not set
    #[clap(long, requires = "generate-cyclone-secret-key-path")]
    pub(crate) generate_cyclone_public_key_path: Option<PathBuf>,
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
                config_map.set("migration_mode", migration_mode.to_string());
            }
            if let Some(url) = args.nats_url {
                config_map.set("nats.url", url);
            }
            if let Some(cyclone_encyption_key_path) = args.cyclone_encryption_key_path {
                config_map.set("cyclone_encryption_key_path", cyclone_encyption_key_path);
            }
        })?
        .try_into()
    }
}
