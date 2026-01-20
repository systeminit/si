use std::{
    sync::Arc,
    time::Duration,
};

use si_events::{
    ActionId,
    Actor,
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    FuncId,
    FuncRun,
    FuncRunId,
    Tenancy,
    WebEvent,
    WorkspacePk,
};
use telemetry::prelude::*;

use super::serialize;
use crate::{
    LayerDbError,
    error::LayerDbResult,
    event::{
        LayeredEvent,
        LayeredEventKind,
        LayeredEventPayload,
    },
    layer_cache::LayerCache,
    persister::PersisterClient,
    pg::PgLayer,
};

pub const DBNAME: &str = "func_runs";
pub const CACHE_NAME: &str = DBNAME;
pub const PARTITION_KEY: &str = "workspace_id";

#[derive(Debug, Clone)]
pub struct FuncRunLayerDb {
    pub cache: Arc<LayerCache<Arc<FuncRun>>>,
    persister_client: PersisterClient,
    ready_many_for_workspace_id_query: String,
    get_last_qualification_for_attribute_value_id: String,
    list_action_history: String,
    get_last_action_by_action_id: String,
    list_management_history: String,
    get_last_management_by_func_and_component_id: String,
    paginated_workspace_query_with_cursor: String,
    paginated_workspace_query_no_cursor: String,
    paginated_component_query_with_cursor: String,
    paginated_component_query_no_cursor: String,
}

// This func run db will be deprecated in favor of the si_db::FuncRunDb, since
// we're doing away with the pg backend of layerdb and these never fit the model
// anyway. Don't use!
impl FuncRunLayerDb {
    // NOTE(victor): Won't migrate to si_db::FuncRunDb - layer cache internal func
    pub fn new(cache: Arc<LayerCache<Arc<FuncRun>>>, persister_client: PersisterClient) -> Self {
        Self {
            cache,
            persister_client,
            ready_many_for_workspace_id_query: format!(
                "SELECT * FROM {DBNAME} WHERE workspace_id = $1"
            ),
            get_last_qualification_for_attribute_value_id: format!(
                "SELECT value FROM {DBNAME}
                   WHERE attribute_value_id = $2 AND workspace_id = $1
                   ORDER BY updated_at DESC
                   LIMIT 1",
            ),
            list_action_history: format!(
                "SELECT value FROM {DBNAME}
                   WHERE function_kind = 'Action' AND workspace_id = $1
                   ORDER BY updated_at DESC",
            ),
            get_last_action_by_action_id: format!(
                "
                SELECT value FROM {DBNAME}
                  WHERE function_kind = 'Action' AND workspace_id = $1 AND action_id = $2
                  ORDER BY updated_at DESC
                  LIMIT 1",
            ),
            list_management_history: format!(
                r#"
                SELECT value FROM {DBNAME}
                WHERE function_kind = 'Management' AND workspace_id = $1 AND change_set_id = $2 AND action_id IS NOT NULL
                ORDER BY updated_at DESC
            "#
            ),
            get_last_management_by_func_and_component_id: format!(
                r#"
                SELECT value FROM {DBNAME}
                WHERE function_kind = 'Management' AND workspace_id = $1 AND change_set_id = $2 AND component_id = $3 AND action_id = $4
                ORDER BY updated_at DESC
                LIMIT 1
            "#
            ),
            paginated_workspace_query_with_cursor: format!(
                r#"
                SELECT * FROM {DBNAME}
                WHERE workspace_id = $1
                  AND change_set_id = $2
                  AND (
                    created_at < (SELECT created_at FROM {DBNAME} WHERE key = $3) OR
                    (created_at = (SELECT created_at FROM {DBNAME} WHERE key = $3) AND key < $3)
                )
                ORDER BY created_at DESC, key DESC
                LIMIT $4
                "#
            ),
            paginated_workspace_query_no_cursor: format!(
                r#"
                SELECT * FROM {DBNAME}
                WHERE workspace_id = $1
                  AND change_set_id = $2
                ORDER BY created_at DESC, key DESC
                LIMIT $3
                "#
            ),
            paginated_component_query_with_cursor: format!(
                r#"
                SELECT * FROM {DBNAME}
                WHERE workspace_id = $1
                  AND change_set_id = $2
                  AND component_id = $3
                  AND (
                    created_at < (SELECT created_at FROM {DBNAME} WHERE key = $4) OR
                    (created_at = (SELECT created_at FROM {DBNAME} WHERE key = $4) AND key < $4)
                )
                ORDER BY created_at DESC, key DESC
                LIMIT $5
                "#
            ),
            paginated_component_query_no_cursor: format!(
                r#"
                SELECT * FROM {DBNAME}
                WHERE workspace_id = $1
                  AND change_set_id = $2
                  AND component_id = $3
                ORDER BY created_at DESC, key DESC
                LIMIT $4
                "#
            ),
        }
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb as ::new()
    pub async fn write(
        &self,
        value: Arc<FuncRun>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<()> {
        let (postcard_value, size_hint) = serialize::to_vec(&value)?;
        let cache_key: Arc<str> = value.id().to_string().into();
        let sort_key: Arc<str> = value.tenancy().workspace_pk.to_string().into();

        self.cache
            .insert_or_update(cache_key.clone(), value, size_hint);

        let event = LayeredEvent::new(
            LayeredEventKind::FuncRunWrite,
            Arc::new(DBNAME.to_string()),
            cache_key,
            Arc::new(postcard_value),
            Arc::new(sort_key.to_string()),
            web_events,
            tenancy,
            actor,
        );
        let reader = self.persister_client.write_event(event)?;
        let _ = reader.get_status().await;

        Ok(())
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    #[instrument(level = "debug", skip_all)]
    pub async fn get_last_run_for_action_id_opt(
        &self,
        workspace_pk: WorkspacePk,
        action_id: ActionId,
    ) -> LayerDbResult<Option<FuncRun>> {
        let maybe_row = self
            .cache
            .pg()
            .query_opt(
                &self.get_last_action_by_action_id,
                &[&workspace_pk, &action_id],
            )
            .await?;

        let maybe_func = if let Some(row) = maybe_row {
            Some(serialize::from_bytes(row.get("value"))?)
        } else {
            None
        };

        Ok(maybe_func)
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    pub async fn get_last_run_for_action_id(
        &self,
        workspace_pk: WorkspacePk,
        action_id: ActionId,
    ) -> LayerDbResult<FuncRun> {
        self.get_last_run_for_action_id_opt(workspace_pk, action_id)
            .await?
            .ok_or_else(|| LayerDbError::ActionIdNotFound(action_id))
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    pub async fn list_management_history(
        &self,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> LayerDbResult<Option<Vec<FuncRun>>> {
        let maybe_rows = self
            .cache
            .pg()
            .query(
                &self.list_management_history,
                &[&workspace_pk, &change_set_id],
            )
            .await?;
        let result = match maybe_rows {
            Some(rows) => {
                let mut result_rows = Vec::with_capacity(rows.len());
                for row in rows.into_iter() {
                    let postcard_bytes: Vec<u8> = row.get("value");
                    let func_run: FuncRun = serialize::from_bytes(&postcard_bytes[..])?;
                    result_rows.push(func_run);
                }
                Some(result_rows)
            }
            None => None,
        };
        Ok(result)
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    pub async fn get_last_management_run_for_func_and_component_id(
        &self,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        component_id: ComponentId,
        func_id: FuncId,
    ) -> LayerDbResult<Option<FuncRun>> {
        let maybe_row = self
            .cache
            .pg()
            .query_opt(
                &self.get_last_management_by_func_and_component_id,
                &[&workspace_pk, &change_set_id, &component_id, &func_id],
            )
            .await?;

        let maybe_func = if let Some(row) = maybe_row {
            Some(serialize::from_bytes(row.get("value"))?)
        } else {
            None
        };

        Ok(maybe_func)
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    pub async fn get_last_qualification_for_attribute_value_id(
        &self,
        workspace_id: WorkspacePk,
        attribute_value_id: AttributeValueId,
    ) -> LayerDbResult<Option<FuncRun>> {
        let max_count = 100;
        let mut current_count = 0;
        while current_count < max_count {
            let maybe_row = self
                .cache
                .pg()
                .query_opt(
                    &self.get_last_qualification_for_attribute_value_id,
                    &[&workspace_id, &attribute_value_id],
                )
                .await?;
            let result = match maybe_row {
                Some(row) => {
                    let postcard_bytes: Vec<u8> = row.get("value");
                    let func_run: FuncRun = serialize::from_bytes(&postcard_bytes[..])?;
                    Some(func_run)
                }
                None => None,
            };
            if result.is_some() {
                return Ok(result);
            } else {
                current_count += 1;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        Ok(None)
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    pub async fn read(&self, key: FuncRunId) -> LayerDbResult<Option<Arc<FuncRun>>> {
        self.cache.get(key.to_string().into()).await
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    pub async fn try_read(&self, key: FuncRunId) -> LayerDbResult<Arc<FuncRun>> {
        self.cache
            .get(key.to_string().into())
            .await?
            .ok_or_else(|| LayerDbError::MissingFuncRun(key))
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    pub async fn read_many_for_workspace(
        &self,
        workspace_id: WorkspacePk,
    ) -> LayerDbResult<Option<Vec<Arc<FuncRun>>>> {
        let maybe_rows = self
            .cache
            .pg()
            .query(&self.ready_many_for_workspace_id_query, &[&workspace_id])
            .await?;
        match maybe_rows {
            Some(rows) => {
                let mut func_runs = Vec::new();
                for row in rows {
                    // NOTE(nick): higher order functions... yeah I want those errors, sorry.
                    func_runs.push(serialize::from_bytes(row.get("value"))?)
                }
                Ok(Some(func_runs))
            }
            None => Ok(None),
        }
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    /// Read function runs for a workspace with pagination support.
    ///
    /// This method uses cursor-based pagination where:
    /// - `limit` controls how many items to return per page
    /// - `cursor` is the ID of the last item from the previous page
    /// - Results are filtered by workspace_id and change_set_id
    ///
    /// Results are ordered by creation time (newest first).
    #[instrument(level = "debug", skip_all)]
    pub async fn read_many_for_workspace_paginated(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        limit: i64,
        cursor: Option<FuncRunId>,
    ) -> LayerDbResult<Option<Vec<Arc<FuncRun>>>> {
        // Choose the appropriate query and parameters based on whether a cursor is provided
        let maybe_rows = if let Some(cursor_id) = cursor {
            // When cursor is provided, fetch records older than the cursor
            self.cache
                .pg()
                .query(
                    &self.paginated_workspace_query_with_cursor,
                    &[
                        &workspace_id,
                        &change_set_id.to_string(),
                        &cursor_id.to_string(),
                        &limit,
                    ],
                )
                .await?
        } else {
            // Initial fetch with no cursor, just get the most recent ones
            self.cache
                .pg()
                .query(
                    &self.paginated_workspace_query_no_cursor,
                    &[&workspace_id, &change_set_id.to_string(), &limit],
                )
                .await?
        };

        // Process the results
        match maybe_rows {
            Some(rows) => {
                let mut func_runs = Vec::with_capacity(rows.len());
                for row in rows {
                    func_runs.push(serialize::from_bytes(row.get("value"))?)
                }
                Ok(Some(func_runs))
            }
            None => Ok(None),
        }
    }

    // NOTE(victor): Migrated to si_db::FuncRunDb
    /// Read function runs for a specific component with pagination support.
    ///
    /// This method uses cursor-based pagination where:
    /// - `limit` controls how many items to return per page
    /// - `cursor` is the ID of the last item from the previous page
    /// - Results are filtered by workspace_id, change_set_id, and component_id
    ///
    /// Results are ordered by creation time (newest first).
    #[instrument(level = "debug", skip_all)]
    pub async fn read_many_for_component_paginated(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        component_id: ComponentId,
        limit: i64,
        cursor: Option<FuncRunId>,
    ) -> LayerDbResult<Option<Vec<Arc<FuncRun>>>> {
        // Choose the appropriate query and parameters based on whether a cursor is provided
        let maybe_rows = if let Some(cursor_id) = cursor {
            // When cursor is provided, fetch records older than the cursor
            self.cache
                .pg()
                .query(
                    &self.paginated_component_query_with_cursor,
                    &[
                        &workspace_id,
                        &change_set_id.to_string(),
                        &component_id.to_string(),
                        &cursor_id.to_string(),
                        &limit,
                    ],
                )
                .await?
        } else {
            // Initial fetch with no cursor, just get the most recent ones
            self.cache
                .pg()
                .query(
                    &self.paginated_component_query_no_cursor,
                    &[
                        &workspace_id,
                        &change_set_id.to_string(),
                        &component_id.to_string(),
                        &limit,
                    ],
                )
                .await?
        };

        // Process the results
        match maybe_rows {
            Some(rows) => {
                let mut func_runs = Vec::with_capacity(rows.len());
                for row in rows {
                    func_runs.push(serialize::from_bytes(row.get("value"))?)
                }
                Ok(Some(func_runs))
            }
            None => Ok(None),
        }
    }

    // NOTE(victor): Won't migrate to si_db::FuncRunDb - internal layer cache func
    pub async fn insert_to_pg(
        pg: &PgLayer,
        event_payload: &LayeredEventPayload,
    ) -> LayerDbResult<()> {
        let func_run: FuncRun = serialize::from_bytes(&event_payload.value[..])?;
        let json: serde_json::Value = serde_json::to_value(func_run.clone())?;
        pg.insert_raw(
            &format!(
                "INSERT INTO {DBNAME} (
                    key,
                    sort_key,
                    created_at,
                    updated_at,
                    state,
                    function_kind,
                    workspace_id,
                    change_set_id,
                    actor_id,
                    component_id,
                    attribute_value_id,
                    action_id,
                    action_originating_change_set_id,
                    json_value,
                    value
                ) VALUES (
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9,
                    $10,
                    $11,
                    $12,
                    $13,
                    $14,
                    $15
                ) ON CONFLICT (key) DO UPDATE SET
                    updated_at = EXCLUDED.updated_at,
                    state = EXCLUDED.state,
                    json_value = EXCLUDED.json_value,
                    value = EXCLUDED.value;"
            ),
            &[
                &func_run.id().to_string(),
                &func_run.tenancy().workspace_pk.to_string(),
                &func_run.created_at(),
                &func_run.updated_at(),
                &func_run.state().to_string(),
                &func_run.function_kind().to_string(),
                &func_run.tenancy().workspace_pk.to_string(),
                &func_run.tenancy().change_set_id.to_string(),
                &func_run.actor().to_string(),
                &func_run.component_id().map(|v| v.to_string()),
                &func_run.attribute_value_id().map(|v| v.to_string()),
                &func_run.action_id().map(|v| v.to_string()),
                &func_run
                    .action_originating_change_set_id()
                    .map(|v| v.to_string()),
                &json,
                &&event_payload.value[..],
            ],
        )
        .await?;
        Ok(())
    }
}
