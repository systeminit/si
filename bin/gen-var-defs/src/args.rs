use crate::config::{Config, ConfigError, ConfigFile, StandardConfigFile};
use clap::Parser;

const NAME: &str = "gen-var-defs";

pub(crate) fn parse() -> Args {
    Args::parse()
}

#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
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

    #[arg(long, default_value = "all")]
    pub(crate) schema_name: String,

    #[arg(long, default_value = "./")]
    pub(crate) output_dir: String,
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
        })?
        .try_into()
    }
}
