use axum::extract::Query;
use axum::Json;
use chrono::{DateTime, Utc};
use dal::Visibility;
use serde::{Deserialize, Serialize};
use si_events::{
    ActionId, ActionKind, ActionPrototypeId, ActionResultState, ChangeSetId, ComponentId, FuncRun,
    FuncRunId,
};

use super::{ActionError, ActionResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionHistoryView {
    pub id: ActionId,
    pub func_run_id: FuncRunId,
    pub prototype_id: ActionPrototypeId,
    pub name: String,
    pub component_id: ComponentId,
    pub component_name: String,
    pub schema_name: String,
    pub action_name: String,
    pub kind: ActionKind,
    pub originating_change_set_id: ChangeSetId,
    pub originating_change_set_name: String,
    pub updated_at: DateTime<Utc>,
    pub result: ActionResultState,
}

impl TryFrom<FuncRun> for ActionHistoryView {
    type Error = ActionError;

    fn try_from(func_run: FuncRun) -> Result<Self, Self::Error> {
        Ok(Self {
            id: func_run
                .action_id()
                .ok_or_else(|| ActionError::ActionHistoryFieldMissing("action_id".to_string()))?,
            func_run_id: func_run.id(),
            name: func_run
                .action_display_name()
                .ok_or_else(|| {
                    ActionError::ActionHistoryFieldMissing("action_display_name".to_string())
                })?
                .to_string(),
            prototype_id: func_run.action_prototype_id().ok_or_else(|| {
                ActionError::ActionHistoryFieldMissing("action_prototype_id".to_string())
            })?,
            component_id: func_run.component_id().ok_or_else(|| {
                ActionError::ActionHistoryFieldMissing("component_id".to_string())
            })?,
            component_name: func_run
                .component_name()
                .ok_or_else(|| {
                    ActionError::ActionHistoryFieldMissing("component_name".to_string())
                })?
                .to_string(),
            schema_name: func_run
                .schema_name()
                .ok_or_else(|| ActionError::ActionHistoryFieldMissing("schema_name".to_string()))?
                .to_string(),
            action_name: func_run
                .action_display_name()
                .ok_or_else(|| {
                    ActionError::ActionHistoryFieldMissing("action_display_name".to_string())
                })?
                .to_string(),
            kind: func_run
                .action_kind()
                .ok_or_else(|| ActionError::ActionHistoryFieldMissing("action_kind".to_string()))?,
            originating_change_set_id: func_run.action_originating_change_set_id().ok_or_else(
                || {
                    ActionError::ActionHistoryFieldMissing(
                        "action_originating_change_set_id".to_string(),
                    )
                },
            )?,
            originating_change_set_name: func_run
                .action_originating_change_set_name()
                .ok_or_else(|| {
                    ActionError::ActionHistoryFieldMissing(
                        "action_originating_change_set_name".to_string(),
                    )
                })?
                .to_string(),
            result: func_run.action_result_state().ok_or_else(|| {
                ActionError::ActionHistoryFieldMissing("action_result_state".to_string())
            })?,
            updated_at: func_run.updated_at(),
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActionHistoryRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ActionHistoryResponse = Vec<ActionHistoryView>;

pub async fn history(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ActionHistoryRequest>,
) -> ActionResult<Json<ActionHistoryResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut result = Vec::new();
    if let Some(action_history_list) = ctx
        .layer_db()
        .func_run()
        .list_action_history(ctx.events_tenancy().workspace_pk)
        .await?
    {
        for action_history in action_history_list.into_iter() {
            result.push(action_history.try_into()?);
        }
    }

    Ok(Json(result))
}
