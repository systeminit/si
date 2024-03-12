use std::{marker::PhantomData, sync::Arc};

use si_data_pg::{PgPool, PgPoolConfig};

use crate::error::LayerDbResult;

mod embedded {
    use refinery::embed_migrations;

    embed_migrations!("./src/migrations");
}

pub const DBNAME: &str = "si_layer_db";
pub const APPLICATION_NAME: &str = "si-layer-db";

pub fn default_pg_pool_config() -> PgPoolConfig {
    PgPoolConfig {
        dbname: DBNAME.into(),
        application_name: APPLICATION_NAME.into(),
        ..Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct PgLayer<K>
where
    K: AsRef<[u8]> + Copy + Send + Sync,
{
    pool: Arc<PgPool>,
    pub table_name: String,
    get_value_query: String,
    insert_value_query: String,
    contains_key_query: String,
    search_query: String,
    _phantom_k: PhantomData<K>,
}

impl<K> PgLayer<K>
where
    K: AsRef<[u8]> + Copy + Send + Sync,
{
    pub fn new(pg_pool: PgPool, table_name: impl Into<String>) -> Self {
        let table_name = table_name.into();
        Self {
            pool: Arc::new(pg_pool),
            get_value_query: format!("SELECT value FROM {table_name} WHERE key = $1 LIMIT 1"),
            insert_value_query: format!("INSERT INTO {table_name} (key, sort_key, value) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"),
            contains_key_query: format!("SELECT key FROM {table_name} WHERE key = $1 LIMIT 1"),
            search_query: format!("SELECT value FROM {table_name} WHERE sort_key LIKE $1"),
            table_name,
            _phantom_k: PhantomData,
        }
    }

    pub async fn migrate(&self) -> LayerDbResult<()> {
        self.pool.migrate(embedded::migrations::runner()).await?;
        Ok(())
    }

    pub async fn get(&self, key: &K) -> LayerDbResult<Option<Vec<u8>>> {
        let client = self.pool.get().await?;
        let maybe_row = client
            .query_opt(&self.get_value_query, &[&key.as_ref()])
            .await?;

        match maybe_row {
            Some(row) => Ok(Some(row.get("value"))),
            None => Ok(None),
        }
    }

    pub async fn search(&self, sort_key_like: impl AsRef<str>) -> LayerDbResult<Vec<Vec<u8>>> {
        let sort_key_like = sort_key_like.as_ref();
        let client = self.pool.get().await?;
        let rows = client.query(&self.search_query, &[&sort_key_like]).await?;

        Ok(rows.into_iter().map(|r| r.get("value")).collect())
    }

    pub async fn insert(
        &self,
        key: K,
        sort_key: impl AsRef<str>,
        value: &[u8],
    ) -> LayerDbResult<()> {
        let client = self.pool.get().await?;
        let sort_key = sort_key.as_ref();
        client
            .query(
                &self.insert_value_query,
                &[&key.as_ref(), &sort_key, &value],
            )
            .await?;
        Ok(())
    }

    pub async fn contains_key(&self, key: &K) -> LayerDbResult<bool> {
        let client = self.pool.get().await?;
        let maybe_row = client
            .query_opt(&self.contains_key_query, &[&key.as_ref()])
            .await?;

        Ok(maybe_row.is_some())
    }
}
