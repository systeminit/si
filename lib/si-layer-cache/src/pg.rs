use std::{
    collections::HashMap,
    sync::Arc,
};

use si_data_pg::{
    PgPool,
    PgPoolConfig,
    PgRow,
    postgres_types::ToSql,
};
use telemetry::tracing::info;
use telemetry_utils::monotonic;

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
pub struct PgLayer {
    pool: Arc<PgPool>,
    pub table_name: String,
    delete_query: String,
    get_value_query: String,
    get_value_by_prefix_query: String,
    get_value_many_query: String,
    get_most_recent_query: String,
    insert_value_query: String,
    contains_key_query: String,
    search_query: String,
}

impl PgLayer {
    pub fn new(pg_pool: PgPool, table_name: impl Into<String>) -> Self {
        let table_name = table_name.into();
        Self {
            pool: Arc::new(pg_pool),
            delete_query: format!("DELETE FROM {table_name} WHERE key = $1"),
            get_value_query: format!("SELECT value FROM {table_name} WHERE key = $1 LIMIT 1"),
            get_value_by_prefix_query: format!(
                "SELECT key, value FROM {table_name} WHERE key like $1"
            ),
            get_value_many_query: format!(
                "SELECT key, value FROM {table_name} WHERE key = any($1)"
            ),
            get_most_recent_query: format!(
                "SELECT key, value FROM {table_name} ORDER BY created_at LIMIT $1"
            ),
            insert_value_query: format!(
                "INSERT INTO {table_name} (key, sort_key, value) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
            ),
            contains_key_query: format!("SELECT key FROM {table_name} WHERE key = $1 LIMIT 1"),
            search_query: format!("SELECT value FROM {table_name} WHERE sort_key LIKE $1"),
            table_name,
        }
    }

    pub async fn migrate(&self) -> LayerDbResult<()> {
        self.pool.migrate(embedded::migrations::runner()).await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> LayerDbResult<Option<Vec<u8>>> {
        let key: String = key.into();
        let client = self.pool.get().await?;
        let maybe_row = client.query_opt(&self.get_value_query, &[&key]).await?;

        match maybe_row {
            Some(row) => {
                monotonic!(layer_cache.hit.pg = 1);
                Ok(Some(row.get("value")))
            }
            None => Ok(None),
        }
    }

    pub async fn get_raw(
        &self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> LayerDbResult<Option<PgRow>> {
        let client = self.pool.get().await?;
        match client.query_opt(query, params).await? {
            Some(row) => Ok(Some(row)),
            None => Ok(None),
        }
    }

    pub async fn get_many(
        &self,
        keys: &[Arc<str>],
    ) -> LayerDbResult<Option<HashMap<String, Vec<u8>>>> {
        let mut result = HashMap::new();
        let client = self.pool.get().await?;

        let key_refs: Vec<&str> = keys.iter().map(|key_arc| key_arc.as_ref()).collect();

        for row in client
            .query(&self.get_value_many_query, &[&key_refs])
            .await?
        {
            monotonic!(layer_cache.hit.pg = 1);
            result.insert(
                row.get::<&str, String>("key").to_owned(),
                row.get::<&str, Vec<u8>>("value"),
            );
        }

        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    pub async fn query(
        &self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> LayerDbResult<Option<Vec<PgRow>>> {
        let client = self.pool.get().await?;
        Ok(Some(client.query(query, params).await?))
    }

    pub async fn query_opt(
        &self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> LayerDbResult<Option<PgRow>> {
        let client = self.pool.get().await?;
        Ok(client.query_opt(query, params).await?)
    }

    pub async fn get_many_by_prefix(
        &self,
        key: &str,
    ) -> LayerDbResult<Option<HashMap<String, Vec<u8>>>> {
        let mut result = HashMap::new();
        let client = self.pool.get().await?;

        for row in client
            .query(&self.get_value_by_prefix_query, &[&format!("{}%", &key)])
            .await?
        {
            result.insert(
                row.get::<&str, String>("key").to_owned(),
                row.get::<&str, Vec<u8>>("value"),
            );
        }

        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    pub async fn get_most_recent(
        &self,
        limit: i64,
    ) -> LayerDbResult<Option<HashMap<String, Vec<u8>>>> {
        let mut result = HashMap::new();
        let client = self.pool.get().await?;

        for row in client.query(&self.get_most_recent_query, &[&limit]).await? {
            result.insert(
                row.get::<&str, String>("key").to_owned(),
                row.get::<&str, Vec<u8>>("value"),
            );
        }

        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))
    }

    pub async fn search(&self, sort_key_like: impl AsRef<str>) -> LayerDbResult<Vec<Vec<u8>>> {
        let sort_key_like = sort_key_like.as_ref();
        let client = self.pool.get().await?;
        let rows = client.query(&self.search_query, &[&sort_key_like]).await?;

        Ok(rows.into_iter().map(|r| r.get("value")).collect())
    }

    pub async fn insert(
        &self,
        key: &str,
        sort_key: impl AsRef<str>,
        value: &[u8],
    ) -> LayerDbResult<()> {
        let client = self.pool.get().await?;
        let sort_key = sort_key.as_ref();
        client
            .query(&self.insert_value_query, &[&key, &sort_key, &value])
            .await?;
        Ok(())
    }

    pub async fn insert_raw(
        &self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> LayerDbResult<()> {
        let client = self.pool.get().await?;
        client.query(query, params).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> LayerDbResult<()> {
        let client = self.pool.get().await?;
        client.query(&self.delete_query, &[&key]).await?;
        Ok(())
    }

    pub async fn contains_key(&self, key: &str) -> LayerDbResult<bool> {
        let client = self.pool.get().await?;
        let maybe_row = client.query_opt(&self.contains_key_query, &[&key]).await?;

        Ok(maybe_row.is_some())
    }
}
