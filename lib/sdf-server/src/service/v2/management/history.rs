use axum::{extract::Query, Json};
use chrono::{DateTime, Utc};
use dal::Visibility;
use serde::{Deserialize, Serialize};
use si_events::{
    ActionResultState, ChangeSetId, ComponentId, FuncId, FuncRun, FuncRunId, ManagementPrototypeId,
};

use crate::extract::{AccessBuilder, HandlerContext};

use super::{ManagementApiError, ManagementApiResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManagementHistoryRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementHistoryItem {
    pub func_run_id: FuncRunId,
    pub name: String,
    pub component_id: ComponentId,
    pub component_name: String,
    pub prototype_id: ManagementPrototypeId,
    pub schema_name: String,
    pub change_set_id: ChangeSetId,
    pub originating_change_set_name: String,
    pub func_id: FuncId,
    pub updated_at: DateTime<Utc>,
    pub status: ActionResultState,
}

pub async fn history(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ManagementHistoryRequest>,
) -> ManagementApiResult<Json<Vec<ManagementHistoryItem>>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut result = vec![];
    if let Some(management_history_list) = ctx
        .layer_db()
        .func_run()
        .list_management_history(
            ctx.events_tenancy().workspace_pk,
            ctx.events_tenancy().change_set_id,
        )
        .await?
    {
        for func_run in management_history_list {
            result.push(ManagementHistoryItem::try_from(func_run)?);
        }
    }

    Ok(Json(result))
}

impl TryFrom<FuncRun> for ManagementHistoryItem {
    type Error = ManagementApiError;

    fn try_from(func_run: FuncRun) -> Result<Self, Self::Error> {
        Ok(Self {
            func_run_id: func_run.id(),
            status: func_run
                .action_result_state()
                .unwrap_or(ActionResultState::Unknown),
            name: func_run
                .function_display_name()
                .unwrap_or_else(|| func_run.function_name())
                .to_string(),
            prototype_id: func_run.management_prototype_id().ok_or_else(|| {
                ManagementApiError::ManagementHistoryFieldMissing("prototype_id".to_string())
            })?,
            component_id: func_run.component_id().ok_or_else(|| {
                ManagementApiError::ManagementHistoryFieldMissing("component_id".to_string())
            })?,
            component_name: func_run
                .component_name()
                .ok_or_else(|| {
                    ManagementApiError::ManagementHistoryFieldMissing("component_name".to_string())
                })?
                .to_string(),
            schema_name: func_run
                .schema_name()
                .ok_or_else(|| {
                    ManagementApiError::ManagementHistoryFieldMissing("schema_name".to_string())
                })?
                .to_string(),
            change_set_id: func_run.change_set_id(),
            func_id: func_run.func_id().ok_or_else(|| {
                ManagementApiError::ManagementHistoryFieldMissing("func_id".to_string())
            })?,
            originating_change_set_name: func_run
                .action_originating_change_set_name()
                .ok_or_else(|| {
                    ManagementApiError::ManagementHistoryFieldMissing(
                        "action_originating_change_set_name".to_string(),
                    )
                })?
                .to_string(),
            updated_at: func_run.updated_at(),
        })
    }
}
