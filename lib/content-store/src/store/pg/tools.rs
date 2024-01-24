use si_data_pg::{PgPool, PgPoolConfig, PgPoolError};
use telemetry::prelude::*;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/store/pg/migrations");
}

const DBNAME: &str = "si_content_store";
const APPLICATION_NAME: &str = "si-content-store";

/// A unit struct that provides helpers for performing [`PgStore`] migrations.
#[allow(missing_debug_implementations)]
pub struct PgStoreTools;

impl PgStoreTools {
    /// Create a new [`PgPool`] for a production [`PgStore`].
    pub async fn new_production_pg_pool() -> Result<PgPool, PgPoolError> {
        let pg_pool_config = Self::default_pool_config();
        let pg_pool = PgPool::new(&pg_pool_config).await?;
        Ok(pg_pool)
    }

    /// The default pool configuration for the PgStore
    pub fn default_pool_config() -> PgPoolConfig {
        PgPoolConfig {
            dbname: DBNAME.to_string(),
            application_name: APPLICATION_NAME.to_string(),
            ..Default::default()
        }
    }

    /// Perform migrations for the database.
    #[instrument(skip_all)]
    pub async fn migrate(pg_pool: &PgPool) -> Result<(), PgPoolError> {
        pg_pool.migrate(embedded::migrations::runner()).await
    }
}
