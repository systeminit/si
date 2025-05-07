use std::sync::Arc;

use axum::{
    Json,
    extract::Path,
};
use dal::{
    AttributeValueId,
    WorkspacePk,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::get_func_run::FuncRunLogView;
use crate::{
    extract::HandlerContext,
    service::v2::{
        AccessBuilder,
        func::FuncAPIResult,
    },
};

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRunLogsResponse {
    pub logs: Option<FuncRunLogView>,
}

pub async fn get_func_run_logs_av(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, attribute_value_id)): Path<(
        WorkspacePk,
        dal::ChangeSetId,
        AttributeValueId,
    )>,
) -> FuncAPIResult<Json<GetFuncRunLogsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let maybe_av_run = ctx
        .layer_db()
        .func_run()
        .get_last_qualification_for_attribute_value_id(
            ctx.events_tenancy().workspace_pk,
            attribute_value_id,
        )
        .await?;
    match maybe_av_run {
        Some(av_run) => {
            let logs = ctx
                .layer_db()
                .func_run_log()
                .get_for_func_run_id(av_run.id())
                .await?
                .map(Arc::unwrap_or_clone)
                .map(|v| v.into());

            Ok(Json(GetFuncRunLogsResponse { logs }))
        }
        None => Ok(Json(GetFuncRunLogsResponse { logs: None })), // todo return friendly error?
    }
}
