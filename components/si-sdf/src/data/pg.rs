use deadpool_postgres::{
    config::ConfigError, Config, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod,
};
use thiserror::Error;
use tokio_postgres::NoTls;

use std::ops::DerefMut;

use si_settings::Settings;

const MIGRATION_LOCK_NUMBER: i64 = 42;

#[derive(Error, Debug)]
pub enum PgError {
    #[error("pg pool config error: {0}")]
    DeadpoolConfig(#[from] ConfigError),
    #[error("pg pool error: {0}")]
    PoolError(#[from] PoolError),
    #[error("migration error: {0}")]
    Refinery(#[from] refinery::Error),
    #[error("tokio pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
}

pub type PgResult<T> = Result<T, PgError>;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/data/migrations");
}

#[derive(Clone)]
pub struct PgPool {
    pub pool: Pool,
}

impl std::fmt::Debug for PgPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgPool").finish()
    }
}

impl PgPool {
    pub async fn new(settings: &Settings) -> PgResult<PgPool> {
        let mut cfg = Config::new();
        cfg.hosts = Some(vec![settings.pg.hostname.clone()]);
        cfg.port = Some(settings.pg.port.clone());
        cfg.user = Some(settings.pg.user.clone());
        cfg.password = Some(settings.pg.password.clone());
        cfg.dbname = Some(settings.pg.dbname.clone());
        cfg.application_name = Some(settings.pg.application_name.clone());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        let pool = cfg.create_pool(NoTls)?;
        pool.get().await?;
        Ok(PgPool { pool })
    }

    pub async fn migrate(&self) -> PgResult<()> {
        let mut conn = self.pool.get().await?;
        conn.query_one("SELECT pg_advisory_lock($1)", &[&MIGRATION_LOCK_NUMBER])
            .await?;
        let client = conn.deref_mut().deref_mut();
        match embedded::migrations::runner().run_async(client).await {
            Ok(_) => {
                conn.query_one("SELECT pg_advisory_unlock($1)", &[&MIGRATION_LOCK_NUMBER])
                    .await?;
                Ok(())
            }
            Err(e) => {
                conn.query_one("SELECT pg_advisory_unlock($1)", &[&MIGRATION_LOCK_NUMBER])
                    .await?;
                Err(e.into())
            }
        }
    }

    pub async fn drop_and_create_public_schema(&self) -> PgResult<()> {
        let conn = self.pool.get().await?;
        conn.execute("DROP SCHEMA IF EXISTS public CASCADE", &[])
            .await?;
        conn.execute("CREATE SCHEMA public", &[]).await?;
        Ok(())
    }
}
