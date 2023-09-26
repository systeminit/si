//! This crate provides tools for using the content store in integration tests.

#![warn(
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    bad_style,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    clippy::missing_panics_doc
)]

use color_eyre::eyre::{Result, WrapErr};
use content_store::{PgMigrationHelpers, PgStore};
use si_data_pg::{PgPool, PgPoolConfig};
use telemetry::prelude::*;
use uuid::Uuid;

const TEST_DBNAME: &str = "si_test_content_store";
const TEST_APPLICATION_NAME: &str = "si-test-content-store";

/// A client for preparing the global content store test database.
#[derive(Debug)]
pub struct PgTestMigrationClient {
    pg_pool: PgPool,
}

impl PgTestMigrationClient {
    /// Create a new [`test migration client`](Self).
    pub async fn new() -> Result<Self> {
        let pg_pool_config = PgPoolConfig {
            dbname: TEST_DBNAME.to_string(),
            application_name: TEST_APPLICATION_NAME.to_string(),
            ..Default::default()
        };
        let pg_pool = PgPool::new(&pg_pool_config).await?;
        Ok(Self { pg_pool })
    }

    /// Test the connection to the global content store test database.
    pub async fn test_connection(&self) -> Result<()> {
        Ok(self.pg_pool.test_connection().await?)
    }

    /// Drop old test databases using the global content store test database as the prefix.
    pub async fn drop_old_test_databases(&self) -> Result<()> {
        let name_prefix = format!("{}_%", &self.pg_pool.db_name());
        let pg_conn = self.pg_pool.get().await?;

        let rows = pg_conn
            .query(
                "SELECT datname FROM pg_database WHERE datname LIKE $1",
                &[&name_prefix.as_str()],
            )
            .await?;

        for row in rows {
            let dbname: String = row.try_get("datname")?;
            debug!(db_name = %dbname, "dropping database");
            pg_conn
                .execute(&format!("DROP DATABASE IF EXISTS {dbname}"), &[])
                .await?;
        }

        Ok(())
    }

    /// Drop and create the public schema for the global content store test database.
    pub async fn drop_and_create_public_schema(&self) -> Result<()> {
        Ok(self.pg_pool.drop_and_create_public_schema().await?)
    }

    /// Perform migrations for the global content store test database.
    pub async fn migrate(&self) -> Result<()> {
        Ok(PgMigrationHelpers::migrate(&self.pg_pool).await?)
    }
}

/// This unit struct provides method(s) for creating [`PgStores`](PgStore) in `dal` integration
/// tests.
#[allow(missing_debug_implementations)]
pub struct DalTestPgStore;

impl DalTestPgStore {
    /// Creates a test-specific database using the global content store test database. Then, a
    /// [`PgPool`] is created for the new database. Finally, a [`PgStore`] is created from that
    /// pool.
    ///
    /// This should be used over [`PgStore::new`] for `dal` integration tests until `dal-test` is
    /// able to perform this functionality on its own.
    #[allow(clippy::new_ret_no_self)]
    pub async fn new() -> Result<PgStore> {
        let global_test_dbname = TEST_DBNAME.to_string();
        let global_application_name = TEST_APPLICATION_NAME.to_string();

        // Connect to the 'postgres' database so we can copy our migrated template test database
        let pg = PgPoolConfig {
            dbname: global_test_dbname,
            application_name: global_application_name,
            ..Default::default()
        };
        let mut new_pg_pool_config = pg.clone();
        new_pg_pool_config.dbname = "postgres".to_string();
        let new_pg_pool = PgPool::new(&new_pg_pool_config)
            .await
            .wrap_err("failed to create PgPool to db 'postgres'")?;
        let db_conn = new_pg_pool
            .get()
            .await
            .wrap_err("failed to connect to db 'postgres'")?;

        // Create new database from template
        let db_name_suffix = Uuid::new_v4().as_simple().to_string();
        let dbname = format!("{}_{}", pg.dbname, db_name_suffix);
        let query = format!(
            "CREATE DATABASE {dbname} WITH TEMPLATE {} OWNER {};",
            pg.dbname, pg.user,
        );
        let db_exists_check = db_conn
            .query_opt(
                "SELECT datname FROM pg_database WHERE datname = $1",
                &[&dbname],
            )
            .await?;
        if db_exists_check.is_none() {
            info!(dbname = %dbname, "creating test-specific database");
            db_conn
                .execute(&query, &[])
                .instrument(debug_span!("creating test database from template"))
                .await
                .wrap_err("failed to create test specific database")?;
        } else {
            info!(dbname = %dbname, "test-specific database already exists");
        }
        // This is ugly, but we pretty much always want to know which test DB is used for
        // any given test when it fails, and the logging/tracing macros aren't captured
        // (or displayed) during tests, while `println!(...)` will be captured the same as
        // "normal" test output, meaning it respects --nocapture and being displayed for
        // failing tests.
        println!("Content store test database: {}", &dbname);

        // Create the pg pool for the new database.
        new_pg_pool_config.dbname = dbname;
        let test_specific_pg_pool = PgPool::new(&new_pg_pool_config)
            .await
            .wrap_err("failed to create PgPool to db 'postgres'")?;

        // Before returning the new store, test the connection.
        test_specific_pg_pool
            .test_connection()
            .await
            .wrap_err("failed to connect to the database")?;

        // Return the pg store using the new pool.
        PgStore::new(test_specific_pg_pool)
            .await
            .wrap_err("failed to create PgStore for new, test-specific database")
    }
}
