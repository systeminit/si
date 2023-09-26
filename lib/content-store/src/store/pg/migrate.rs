use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use si_data_pg::{PgPool, PgPoolConfig};
use telemetry::prelude::*;
use uuid::Uuid;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/store/pg/migrations");
}

#[instrument(skip_all)]
pub async fn migrate(pg_pool: &PgPool) -> Result<()> {
    Ok(pg_pool.migrate(embedded::migrations::runner()).await?)
}

fn random_identifier_string() -> String {
    Uuid::new_v4().as_simple().to_string()
}

// async fn create_db_with_pg_pool() -> Result<PgPool> {
//     let pg = PgPoolConfig::default();
// }

async fn create_test_specific_db_with_pg_pool() -> Result<PgPool> {
    // Connect to the 'postgres' database so we can copy our migrated template test database
    let pg = PgPoolConfig::default();
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
    let db_name_suffix = random_identifier_string();
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
    println!("Test database: {}", &dbname);

    // Return new PG pool that uess the new datatbase
    new_pg_pool_config.dbname = dbname;
    PgPool::new(&new_pg_pool_config)
        .await
        .wrap_err("failed to create PgPool to db 'postgres'")
}
