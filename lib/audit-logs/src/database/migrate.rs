//! Contains functionality for migrating the audit database.

use si_data_pg::{PgPool, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;

use super::AuditDatabaseContext;

#[allow(missing_docs)]
#[derive(Error, Debug)]
pub enum AuditDatabaseMigrationError {
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
}

type Result<T> = std::result::Result<T, AuditDatabaseMigrationError>;

/// Performs migrations for the audit database.
#[instrument(level = "info", name = "audit.init.migrate", skip_all)]
pub async fn migrate(context: &AuditDatabaseContext) -> Result<()> {
    migrate_inner(context.pg_pool()).await
}

#[instrument(level = "info", name = "audit.init.migrate.inner", skip_all)]
async fn migrate_inner(pg: &PgPool) -> Result<()> {
    pg.migrate(embedded::migrations::runner()).await?;
    Ok(())
}

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}
