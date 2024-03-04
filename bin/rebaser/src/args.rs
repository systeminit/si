use clap::{ArgAction, Parser};
use rebaser_server::{Config, ConfigError, ConfigFile, StandardConfigFile};

const NAME: &str = "rebaser";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

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

    /// Disable OpenTelemetry on startup
    #[arg(long)]
    pub(crate) disable_opentelemetry: bool,

    /// Cyclone encryption key file location [default: /run/rebaser/cyclone_encryption.key]
    #[arg(long)]
    pub(crate) cyclone_encryption_key_path: Option<String>,

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
            if let Some(url) = args.nats_url {
                config_map.set("nats.url", url);
            }
            if let Some(cyclone_encyption_key_path) = args.cyclone_encryption_key_path {
                config_map.set("cyclone_encryption_key_path", cyclone_encyption_key_path);
            }
            if let Some(concurrency) = args.concurrency {
                config_map.set("concurrency_limit", i64::from(concurrency));
            }
            if let Some(instance_id) = args.instance_id {
                config_map.set("instance_id", instance_id);
            }

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
