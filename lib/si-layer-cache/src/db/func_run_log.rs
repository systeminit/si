use std::sync::Arc;

use si_events::{
    Actor,
    FuncRunId,
    FuncRunLog,
    FuncRunLogId,
    Tenancy,
    WebEvent,
};

use super::serialize;
use crate::{
    error::LayerDbResult,
    event::{
        LayeredEvent,
        LayeredEventKind,
    },
    layer_cache::LayerCache,
    persister::PersisterClient,
};

pub const DBNAME: &str = "func_run_logs";
pub const CACHE_NAME: &str = DBNAME;
pub const PARTITION_KEY: &str = "workspace_id";

#[derive(Debug, Clone)]
pub struct FuncRunLogLayerDb {
    pub cache: Arc<LayerCache<Arc<FuncRunLog>>>,
    persister_client: PersisterClient,
    get_for_func_run_id_query: String,
    read_id_batch_query: String,
    read_id_batch_no_cutoff_query: String,
}

impl FuncRunLogLayerDb {
    // NOTE(victor): Won't migrate to si_db::FuncRunLogsDb - layer cache internal func
    pub fn new(cache: Arc<LayerCache<Arc<FuncRunLog>>>, persister_client: PersisterClient) -> Self {
        Self {
            cache,
            persister_client,
            get_for_func_run_id_query: format!("SELECT value FROM {DBNAME} WHERE func_run_id = $1"),
            read_id_batch_query: format!(
                "SELECT key FROM {DBNAME} WHERE key < $1 ORDER BY created_at DESC LIMIT $2"
            ),
            read_id_batch_no_cutoff_query: format!(
                "SELECT key FROM {DBNAME} ORDER BY created_at DESC LIMIT $1"
            ),
        }
    }

    // NOTE(victor): Migrated to si_db::FuncRunLogsDb as upsert
    pub async fn write(
        &self,
        value: Arc<FuncRunLog>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<()> {
        let (postcard_value, size_hint) = serialize::to_vec(&value)?;
        let cache_key: Arc<str> = value.id().to_string().into();
        let sort_key: Arc<str> = value.tenancy().workspace_pk.to_string().into();

        self.cache
            .insert_or_update(cache_key.clone(), value.clone(), size_hint);

        // We must insert directly before we persist, so that we get it in order.
        self.insert_to_pg(value.clone()).await?;

        let event = LayeredEvent::new(
            LayeredEventKind::FuncRunLogWrite,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new(sort_key.to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;
        let _ = reader.get_status().await?;

        Ok(())
    }

    // NOTE(victor): Migrated to si_db::FuncRunLogsDb
    pub async fn get_for_func_run_id(
        &self,
        func_run_id: FuncRunId,
    ) -> LayerDbResult<Option<Arc<FuncRunLog>>> {
        let maybe_row = self
            .cache
            .pg()
            .query_opt(&self.get_for_func_run_id_query, &[&func_run_id])
            .await?;
        if let Some(row) = maybe_row {
            Ok(Some(serialize::from_bytes(row.get("value"))?))
        } else {
            Ok(None)
        }
    }

    // NOTE(victor): Won't migrate to si_db::FuncRunLogsDb - internal layer cache func
    async fn insert_to_pg(&self, func_run_log: Arc<FuncRunLog>) -> LayerDbResult<()> {
        self.cache
            .pg()
            .insert_raw(
                &format!(
                    "INSERT INTO {DBNAME} (
                    key,
                    sort_key,
                    created_at,
                    updated_at,
                    workspace_id,
                    change_set_id,
                    func_run_id,
                    value
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8
                ) ON CONFLICT (key) DO UPDATE SET
                    updated_at = EXCLUDED.updated_at,
                    value = EXCLUDED.value;"
                ),
                &[
                    &func_run_log.id().to_string(),
                    &func_run_log.tenancy().workspace_pk.to_string(),
                    &func_run_log.created_at(),
                    &func_run_log.updated_at(),
                    &func_run_log.tenancy().workspace_pk.to_string(),
                    &func_run_log.tenancy().change_set_id.to_string(),
                    &func_run_log.func_run_id().to_string(),
                    &serialize::to_vec(&func_run_log)?.0,
                ],
            )
            .await?;
        Ok(())
    }

    // NOTE(victor): Created for the data migration only, won't be ported to si_db::FuncRunLogDb
    pub async fn read_batch_of_ids(
        &self,
        batch_size: i64,
        cutoff_id: Option<FuncRunLogId>,
    ) -> LayerDbResult<Vec<FuncRunLogId>> {
        let maybe_rows = if let Some(cutoff_id) = cutoff_id {
            let id_str = cutoff_id.to_string();
            self.cache
                .pg()
                .query(&self.read_id_batch_query, &[&id_str, &batch_size])
                .await?
        } else {
            self.cache
                .pg()
                .query(&self.read_id_batch_no_cutoff_query, &[&batch_size])
                .await?
        };

        let Some(rows) = maybe_rows else {
            return Ok(vec![]);
        };

        let mut func_run_log_ids = Vec::new();
        for row in rows {
            let id_string: String = row.get("key");
            let ulid = ulid::Ulid::from_string(&id_string)?;
            func_run_log_ids.push(FuncRunLogId::from(ulid));
        }

        Ok(func_run_log_ids)
    }

    // NOTE(victor): Created for the data migration only
    pub async fn try_read(&self, key: FuncRunLogId) -> LayerDbResult<Arc<FuncRunLog>> {
        let maybe_row = self
            .cache
            .pg()
            .query_opt(
                &format!("SELECT value FROM {DBNAME} WHERE key = $1"),
                &[&key.to_string()],
            )
            .await?;

        let Some(row) = maybe_row else {
            return Err(crate::LayerDbError::FuncRunLogNotFound(key));
        };

        serialize::from_bytes(row.get("value"))
    }
}
