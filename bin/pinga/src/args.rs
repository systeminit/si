use clap::{ArgAction, Parser};
use pinga_server::{Config, ConfigError, ConfigFile, StandardConfigFile};

const NAME: &str = "pinga";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative API service.
///
/// Pinga queue executor system that handles whatever job comes from Nats.
/// It means "drip" in portuguese and also is a name for Cachaça
#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
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
        long,
        env = "SI_NO_COLOR",
        hide_env_values = true,
        conflicts_with = "force_color"
    )]
    pub(crate) no_color: Option<bool>,

    /// Forces ANSI coloring, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long,
        env = "SI_FORCE_COLOR",
        hide_env_values = true,
        conflicts_with = "no_color"
    )]
    pub(crate) force_color: Option<bool>,

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

    /// PostgreSQL connection certification path
    #[arg(long)]
    pub(crate) pg_cert_path: Option<String>,

    /// PostgreSQL connection certification base64 string
    #[arg(long)]
    pub(crate) pg_cert_base64: Option<String>,

    /// NATS connection URL [example: demo.nats.io]
    #[arg(long)]
    pub(crate) nats_url: Option<String>,

    /// NATS credentials string
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) nats_creds: Option<String>,

    /// NATS credentials file
    #[arg(long)]
    pub(crate) nats_creds_path: Option<String>,

    /// Cyclone encryption key file location [default: /run/pinga/cyclone_encryption.key]
    #[arg(long)]
    pub(crate) cyclone_encryption_key_path: Option<String>,

    /// Cyclone encryption key file contents as a base64 encoded string
    #[arg(long)]
    pub(crate) cyclone_encryption_key_base64: Option<String>,

    /// Cyclone secret key as base64 string
    #[arg(long)]
    pub(crate) cyclone_secret_key_base64: Option<String>,

    /// The number of concurrent jobs that can be processed [default: 10]
    #[arg(long)]
    pub(crate) concurrency: Option<u32>,

    /// Instance ID [example: 01GWEAANW5BVFK5KDRVS6DEY0F"]
    ///
    /// And instance ID is used when tracking the execution of jobs in a way that can be traced
    /// back to an instance of a Pinga service.
    #[arg(long)]
    pub(crate) instance_id: Option<String>,
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
            if let Some(cert) = args.pg_cert_path {
                config_map.set("pg.certificate_path", cert);
            }
            if let Some(cert) = args.pg_cert_base64 {
                config_map.set("pg.certificate_base64", cert);
            }
            if let Some(url) = args.nats_url {
                config_map.set("nats.url", url);
            }
            if let Some(creds) = args.nats_creds {
                config_map.set("nats.creds", creds);
            }
            if let Some(creds_file) = args.nats_creds_path {
                config_map.set("nats.creds_file", creds_file);
            }
            if let Some(cyclone_encryption_key_file) = args.cyclone_encryption_key_path {
                config_map.set("crypto.encryption_key_file", cyclone_encryption_key_file);
            }
            if let Some(cyclone_encryption_key_base64) = args.cyclone_encryption_key_base64 {
                config_map.set(
                    "crypto.encryption_key_base64",
                    cyclone_encryption_key_base64,
                );
            }
            if let Some(secret_string) = args.cyclone_secret_key_base64 {
                config_map.set("symmetric_crypto_service.active_key_base64", secret_string);
            }
            if let Some(concurrency) = args.concurrency {
                config_map.set("concurrency_limit", i64::from(concurrency));
            }
            if let Some(instance_id) = args.instance_id {
                config_map.set("instance_id", instance_id);
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
