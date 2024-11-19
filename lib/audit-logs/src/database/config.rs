use serde::{Deserialize, Serialize};
use si_data_pg::PgPoolConfig;

/// The name of the audit database.
pub const DBNAME: &str = "si_audit";
const APPLICATION_NAME: &str = "si-audit";

/// The configuration used for communicating with and setting up the audit database.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditDatabaseConfig {
    /// The configuration for the PostgreSQL pool.
    ///
    /// _Note:_ this is called "pg" for ease of use with layered load configuration files.
    pub pg: PgPoolConfig,
}

impl Default for AuditDatabaseConfig {
    fn default() -> Self {
        Self {
            pg: PgPoolConfig {
                dbname: DBNAME.into(),
                application_name: APPLICATION_NAME.into(),
                ..Default::default()
            },
        }
    }
}
