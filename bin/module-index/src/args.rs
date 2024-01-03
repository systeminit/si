use clap::{ArgAction, Parser};
use module_index_server::{Config, ConfigError, ConfigFile, StandardConfigFile};

const NAME: &str = "module_index";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative Module Index API service
///
/// This is the centralized store of "modules" - ie how users will back up and share
/// their customizations of our system
#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 4.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// PostgreSQL connection pool dbname [example: myapp]
    #[arg(long, env)]
    pub(crate) pg_dbname: Option<String>,

    /// PostgreSQL connection pool hostname [example: prod.db.example.com]
    #[arg(long, env)]
    pub(crate) pg_hostname: Option<String>,

    /// PostgreSQL connection pool max size [example: 8]
    #[arg(long, env)]
    pub(crate) pg_pool_max_size: Option<u32>,

    /// PostgreSQL connection pool port [example: 5432]
    #[arg(long, env)]
    pub(crate) pg_port: Option<u16>,

    /// PostgreSQL connection pool user [example: dbuser]
    #[arg(long, env)]
    pub(crate) pg_user: Option<String>,

    /// PostgreSQL connection pool password [example: dbuser]
    #[arg(long, env)]
    pub(crate) pg_password: Option<String>,

    /// PostgreSQL connection certification path
    #[arg(long)]
    pub(crate) pg_cert_path: Option<String>,

    /// PostgreSQL connection certification base64 string
    #[arg(long)]
    pub(crate) pg_cert_base64: Option<String>,

    /// The address and port to bind the HTTP server to [example: 0.0.0.0:80]
    #[arg(long, env)]
    pub(crate) socket_addr: Option<String>,

    /// The s3 bucket access key id
    #[arg(long, env)]
    pub(crate) s3_access_key_id: Option<String>,

    /// The s3 bucket
    #[arg(long, env)]
    pub(crate) s3_bucket: Option<String>,

    /// The s3 bucket region
    #[arg(long, env)]
    pub(crate) s3_region: Option<String>,

    /// The s3 bucket secret access key
    #[arg(long, env)]
    pub(crate) s3_secret_access_key: Option<String>,

    /// The s3 bucket path prefix
    #[arg(long, env)]
    pub(crate) s3_path_prefix: Option<String>,

    /// The path to the JWT public signing key
    #[arg(long, env)]
    pub(crate) jwt_public_key: Option<String>,

    // /// Database migration mode on startup
    // #[arg(long, value_parser = PossibleValuesParser::new(MigrationMode::variants()))]

    // pub(crate) migration_mode: Option<String>,
    /// Disable OpenTelemetry on startup
    #[arg(long)]
    pub(crate) disable_opentelemetry: bool,
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
            if let Some(password) = args.pg_password {
                config_map.set("pg.password", password);
            }
            if let Some(cert) = args.pg_cert_path {
                config_map.set("pg.certificate_path", cert);
            }
            if let Some(cert) = args.pg_cert_base64 {
                config_map.set("pg.certificate_base64", cert);
            }
            if let Some(socket_addr) = args.socket_addr {
                config_map.set("socket_addr", socket_addr);
            }

            if let Some(s3_access_key_id) = args.s3_access_key_id {
                config_map.set("s3.access_key_id", s3_access_key_id);
            }
            if let Some(s3_secret_access_key) = args.s3_secret_access_key {
                config_map.set("s3.secret_access_key", s3_secret_access_key);
            }
            if let Some(s3_bucket) = args.s3_bucket {
                config_map.set("s3.bucket", s3_bucket);
            }
            if let Some(s3_region) = args.s3_region {
                config_map.set("s3.region", s3_region);
            }
            if let Some(s3_path_prefix) = args.s3_path_prefix {
                config_map.set("s3.path_prefix", s3_path_prefix);
            }
            if let Some(jwt_public_key) = args.jwt_public_key {
                config_map.set("jwt_signing_public_key_path", jwt_public_key);
            }

            // if let Some(migration_mode) = args.migration_mode {
            //     config_map.set("migration_mode", migration_mode);
            // }

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
