use std::{marker::PhantomData, sync::Arc};

use si_data_pg::PgPool;

use crate::error::LayerCacheResult;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

pub const DBNAME: &str = "si_key_value_pairs";
pub const APPLICATION_NAME: &str = "si-key-value-pairs";

#[derive(Clone, Debug)]
pub struct PgLayer<K>
where
    K: AsRef<[u8]> + Copy + Send + Sync,
{
    pool: Arc<PgPool>,
    _phantom_k: PhantomData<K>,
}

impl<K> PgLayer<K>
where
    K: AsRef<[u8]> + Copy + Send + Sync,
{
    pub fn new(pg_pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pg_pool),
            _phantom_k: PhantomData,
        }
    }

    pub async fn migrate(&self) -> LayerCacheResult<()> {
        self.pool.migrate(embedded::migrations::runner()).await?;
        Ok(())
    }

    pub async fn get(&self, key: &K) -> LayerCacheResult<Option<Vec<u8>>> {
        let client = self.pool.get().await?;
        let maybe_row = client
            .query_opt(
                "SELECT * FROM key_value_pairs WHERE key = $1 LIMIT 1",
                &[&key.as_ref()],
            )
            .await?;

        match maybe_row {
            Some(row) => Ok(Some(row.get("value"))),
            None => Ok(None),
        }
    }

    pub async fn insert(&self, key: K, value: &[u8]) -> LayerCacheResult<()> {
        let client = self.pool.get().await?;
        client
            .query(
                "INSERT INTO key_value_pairs (key, value) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                &[&key.as_ref(), &value],
            )
            .await?;
        Ok(())
    }

    pub async fn contains_key(&self, key: &K) -> LayerCacheResult<bool> {
        let client = self.pool.get().await?;
        let maybe_row = client
            .query_opt(
                "SELECT key FROM key_value_pairs WHERE key = $1 LIMIT 1",
                &[&key.as_ref()],
            )
            .await?;

        Ok(maybe_row.is_some())
    }
}
