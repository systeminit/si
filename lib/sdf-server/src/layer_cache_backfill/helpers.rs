use chrono::{
    DateTime,
    Utc,
};
use dal::ServicesContext;
use si_layer_cache::db::{
    cas,
    change_batch,
    encrypted_secret,
    rebase_batch,
    split_snapshot_rebase_batch,
    split_snapshot_subgraph,
    split_snapshot_supergraph,
    workspace_snapshot,
};

use super::error::{
    BackfillError,
    BackfillResult,
};

/// Dispatches an operation to the appropriate cache based on cache_type string.
/// Each arm calls $ctx.layer_db().$accessor().cache.$method($($args),*).await?
macro_rules! dispatch_to_cache {
    ($ctx:expr, $cache_type:expr, $method:ident $(, $arg:expr)*) => {
        match $cache_type {
            "cas" => $ctx.layer_db().cas().cache.$method($($arg),*).await?,
            "workspace_snapshot" => $ctx.layer_db().workspace_snapshot().cache.$method($($arg),*).await?,
            "encrypted_secret" => $ctx.layer_db().encrypted_secret().cache.$method($($arg),*).await?,
            "rebase_batch" => $ctx.layer_db().rebase_batch().cache.$method($($arg),*).await?,
            "change_batch" => $ctx.layer_db().change_batch().cache.$method($($arg),*).await?,
            "split_snapshot_subgraph" => $ctx.layer_db().split_snapshot_subgraph().cache.$method($($arg),*).await?,
            "split_snapshot_supergraph" => $ctx.layer_db().split_snapshot_supergraph().cache.$method($($arg),*).await?,
            "split_snapshot_rebase_batch" => $ctx.layer_db().split_snapshot_rebase_batch().cache.$method($($arg),*).await?,
            _ => return Err(BackfillError::CacheTypeNotBackfillable {
                cache_type: $cache_type.to_string(),
            }),
        }
    };
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub key: String,
    pub created_at: DateTime<Utc>,
}

fn get_table_name(cache_type: &str) -> Result<&'static str, BackfillError> {
    match cache_type {
        "cas" => Ok(cas::DBNAME),
        "workspace_snapshot" => Ok(workspace_snapshot::DBNAME),
        "encrypted_secret" => Ok(encrypted_secret::DBNAME),
        "rebase_batch" => Ok(rebase_batch::DBNAME),
        "change_batch" => Ok(change_batch::DBNAME),
        "split_snapshot_subgraph" => Ok(split_snapshot_subgraph::DBNAME),
        "split_snapshot_supergraph" => Ok(split_snapshot_supergraph::DBNAME),
        "split_snapshot_rebase_batch" => Ok(split_snapshot_rebase_batch::DBNAME),
        _ => Err(BackfillError::CacheTypeNotBackfillable {
            cache_type: cache_type.to_string(),
        }),
    }
}

pub async fn fetch_key_batch(
    ctx: &ServicesContext,
    cache_type: &str,
    cutoff_timestamp: DateTime<Utc>,
    batch_size: usize,
) -> BackfillResult<Vec<KeyInfo>> {
    let table_name = get_table_name(cache_type)?;
    let query = format!(
        "SELECT key, created_at
         FROM {table_name}
         WHERE created_at < $1
         ORDER BY created_at DESC
         LIMIT $2"
    );

    let rows = ctx
        .layer_db()
        .pg_pool()
        .get()
        .await?
        .query(&query, &[&cutoff_timestamp, &(batch_size as i64)])
        .await?;

    let keys = rows
        .into_iter()
        .map(|row| KeyInfo {
            key: row.get("key"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(keys)
}

pub async fn check_s3_exists(
    ctx: &ServicesContext,
    cache_type: &str,
    key: &str,
) -> BackfillResult<bool> {
    Ok(dispatch_to_cache!(ctx, cache_type, s3_head, key))
}

pub async fn fetch_pg_value(
    ctx: &ServicesContext,
    cache_type: &str,
    key: &str,
) -> BackfillResult<Vec<u8>> {
    let table_name = get_table_name(cache_type)?;
    let query = format!("SELECT value FROM {table_name} WHERE key = $1 LIMIT 1");

    let row = ctx
        .layer_db()
        .pg_pool()
        .get()
        .await?
        .query_one(&query, &[&key])
        .await?;

    let value: Vec<u8> = row.get("value");
    Ok(value)
}

pub async fn upload_to_s3(
    ctx: &ServicesContext,
    cache_type: &str,
    key: &str,
    value: &[u8],
) -> BackfillResult<()> {
    dispatch_to_cache!(ctx, cache_type, s3_put_direct, key, value.to_vec());
    Ok(())
}
