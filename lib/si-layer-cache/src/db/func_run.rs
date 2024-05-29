use std::sync::Arc;
use std::time::Duration;

use si_events::{
    ActionId, ActionResultState, Actor, AttributeValueId, ContentHash, FuncRun, FuncRunId, Tenancy,
    WebEvent, WorkspacePk,
};

use crate::event::LayeredEventPayload;
use crate::pg::PgLayer;
use crate::LayerDbError;
use crate::{
    error::LayerDbResult,
    event::{LayeredEvent, LayeredEventKind},
    layer_cache::LayerCache,
    persister::PersisterClient,
};

use super::serialize;

pub const DBNAME: &str = "func_runs";
pub const CACHE_NAME: &str = DBNAME;
pub const PARTITION_KEY: &str = "workspace_id";

#[derive(Debug, Clone)]
pub struct FuncRunDb {
    pub cache: LayerCache<Arc<FuncRun>>,
    persister_client: PersisterClient,
    ready_many_for_workspace_id_query: String,
    get_last_qualification_for_attribute_value_id: String,
    list_action_history: String,
    get_last_action_by_action_id: String,
}

impl FuncRunDb {
    pub fn new(cache: LayerCache<Arc<FuncRun>>, persister_client: PersisterClient) -> Self {
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
        }
    }

    pub async fn list_action_history(
        &self,
        workspace_id: WorkspacePk,
    ) -> LayerDbResult<Option<Vec<FuncRun>>> {
        let maybe_rows = self
            .cache
            .pg()
            .query(&self.list_action_history, &[&workspace_id])
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

    pub async fn write(
        &self,
        value: Arc<FuncRun>,
        web_events: Option<Vec<WebEvent>>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<()> {
        let postcard_value = serialize::to_vec(&value)?;
        let cache_key: Arc<str> = value.id().to_string().into();
        let sort_key: Arc<str> = value.tenancy().workspace_pk.to_string().into();

        self.cache
            .insert_or_update(cache_key.clone(), value.clone())
            .await;

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

    pub async fn set_values_and_set_state_to_success(
        &self,
        func_run_id: FuncRunId,
        unprocessed_value_cas: Option<ContentHash>,
        value_cas: Option<ContentHash>,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<()> {
        let func_run_old = self.try_read(func_run_id).await?;
        let mut func_run_new = Arc::unwrap_or_clone(func_run_old);
        func_run_new.set_result_unprocessed_value_cas_address(unprocessed_value_cas);
        func_run_new.set_result_value_cas_address(value_cas);
        func_run_new.set_state_to_success();

        self.write(Arc::new(func_run_new), None, tenancy, actor)
            .await?;

        Ok(())
    }

    pub async fn set_state_to_success(
        &self,
        func_run_id: FuncRunId,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<()> {
        let func_run_old = self.try_read(func_run_id).await?;
        let mut func_run_new = Arc::unwrap_or_clone(func_run_old);
        func_run_new.set_state_to_success();

        self.write(Arc::new(func_run_new), None, tenancy, actor)
            .await?;

        Ok(())
    }

    pub async fn set_action_result_state(
        &self,
        func_run_id: FuncRunId,
        action_result_state: ActionResultState,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<()> {
        let func_run_old = self.try_read(func_run_id).await?;
        let mut func_run_new = Arc::unwrap_or_clone(func_run_old);
        func_run_new.set_action_result_state(Some(action_result_state));

        self.write(Arc::new(func_run_new), None, tenancy, actor)
            .await?;

        Ok(())
    }

    pub async fn set_action_result_state_for_action_id(
        &self,
        action_id: ActionId,
        action_result_state: ActionResultState,
        tenancy: Tenancy,
        actor: Actor,
    ) -> LayerDbResult<()> {
        let maybe_row = self
            .cache
            .pg()
            .query_opt(&self.get_last_action_by_action_id, &[&tenancy.workspace_pk])
            .await?
            .ok_or_else(|| LayerDbError::ActionIdNotFound(action_id))?;
        let mut func_run: FuncRun = serialize::from_bytes(maybe_row.get("value"))?;
        func_run.set_action_result_state(Some(action_result_state));

        self.write(Arc::new(func_run), None, tenancy, actor).await?;

        Ok(())
    }

    pub async fn read(&self, key: FuncRunId) -> LayerDbResult<Option<Arc<FuncRun>>> {
        self.cache.get(key.to_string().into()).await
    }

    pub async fn try_read(&self, key: FuncRunId) -> LayerDbResult<Arc<FuncRun>> {
        self.cache
            .get(key.to_string().into())
            .await?
            .ok_or_else(|| LayerDbError::MissingFuncRun(key))
    }

    // NOTE(nick): this is just to test that things are working. We probably want some customization
    // for where clauses, etc. in the real version. This should be a step closer to how we'll query
    // for history though.
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
