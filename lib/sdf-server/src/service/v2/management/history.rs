use axum::{
    Json,
    extract::Query,
};
use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{FuncRunDb, Visibility};
use si_events::{
    ActionResultState,
    ComponentId,
    FuncId,
    FuncRun,
    FuncRunId,
    FuncRunState,
    ManagementPrototypeId,
};

use super::{
    ManagementApiError,
    ManagementApiResult,
};
use crate::{
    extract::HandlerContext,
    service::v2::AccessBuilder,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManagementHistoryRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

// Fields that also exist in FuncRunView are given the same name here
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementHistoryItem {
    pub id: FuncRunId,
    pub state: FuncRunState,
    pub function_name: String,
    pub function_display_name: Option<String>,
    pub component_id: ComponentId,
    pub component_name: String,
    pub management_prototype_id: ManagementPrototypeId,
    pub schema_name: String,
    pub originating_change_set_name: String,
    pub func_id: FuncId,
    pub updated_at: DateTime<Utc>,
    pub action_result_state: Option<ActionResultState>,
}

pub async fn history(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ManagementHistoryRequest>,
) -> ManagementApiResult<Json<Vec<ManagementHistoryItem>>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut result = vec![];

    for func_run in FuncRunDb::list_management_history(
        &ctx,
        ctx.events_tenancy().workspace_pk,
        ctx.events_tenancy().change_set_id,
    ).await? {
        result.push(ManagementHistoryItem::try_from(func_run)?);
    }

    Ok(Json(result))
}

impl TryFrom<FuncRun> for ManagementHistoryItem {
    type Error = ManagementApiError;

    fn try_from(func_run: FuncRun) -> Result<Self, Self::Error> {
        Ok(Self {
            id: func_run.id(),
            state: func_run.state(),
            action_result_state: func_run.action_result_state(),
            function_name: func_run.function_name().to_string(),
            function_display_name: func_run.function_display_name().map(str::to_string),
            management_prototype_id: func_run.management_prototype_id().ok_or_else(|| {
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
