//! Contains functionality for setting up and communicating with the audit database.

use serde::{Deserialize, Serialize};
use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;

mod migrate;

pub use migrate::{migrate, AuditDatabaseMigrationError};

/// The name of the audit database.
pub const DBNAME: &str = "si_audit";
const APPLICATION_NAME: &str = "si-audit";

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum AuditDatabaseContextError {
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
}

type Result<T> = std::result::Result<T, AuditDatabaseContextError>;

/// The context used for communicating with and setting up the audit database.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct AuditDatabaseContext {
    pg_pool: PgPool,
}

impl AuditDatabaseContext {
    /// Creates an [`AuditDatabaseContext`] from an [`AuditDatabaseConfig`].
    #[instrument(level = "info", name = "audit.context.from_config", skip_all)]
    pub async fn from_config(config: &AuditDatabaseConfig) -> Result<Self> {
        Ok(Self {
            pg_pool: PgPool::new(&config.pg).await?,
        })
    }

    /// Returns a reference to the [`PgPool`].
    pub fn pg_pool(&self) -> &PgPool {
        &self.pg_pool
    }
}

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
