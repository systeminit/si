use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/store/pg/migrations");
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PgMigrationHelpersError {
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
}

pub(crate) type PgMigrationHelpersResult<T> = Result<T, PgMigrationHelpersError>;

const DBNAME: &str = "si_content_store";
const APPLICATION_NAME: &str = "si_test_content_store";

/// A unit struct that provides helpers for performing [`PgStore`] migrations.
#[allow(missing_debug_implementations)]
pub struct PgMigrationHelpers;

impl PgMigrationHelpers {
    /// Create a new [`PgPool`] for a production [`PgStore`].
    pub async fn new_production_pg_pool() -> PgMigrationHelpersResult<PgPool> {
        let pg_pool_config = PgPoolConfig {
            dbname: DBNAME.to_string(),
            application_name: APPLICATION_NAME.to_string(),
            ..Default::default()
        };
        let pg_pool = PgPool::new(&pg_pool_config).await?;
        Ok(pg_pool)
    }

    /// Perform migrations for the database.
    #[instrument(skip_all)]
    pub async fn migrate(pg_pool: &PgPool) -> PgMigrationHelpersResult<()> {
        Ok(pg_pool.migrate(embedded::migrations::runner()).await?)
    }
}
