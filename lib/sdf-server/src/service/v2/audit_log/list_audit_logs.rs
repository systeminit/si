use axum::{
    extract::{Path, Query},
    Json,
};
use dal::audit_logging;
use serde::{Deserialize, Serialize};
use si_frontend_types as frontend_types;

use super::AuditLogResult;
use crate::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsRequest {
    size: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsResponse {
    logs: Vec<frontend_types::AuditLog>,
    can_load_more: bool,
}

pub async fn list_audit_logs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id)): Path<(dal::WorkspacePk, dal::ChangeSetId)>,
    Query(request): Query<ListAuditLogsRequest>,
) -> AuditLogResult<Json<ListAuditLogsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let (logs, can_load_more) = audit_logging::list(&ctx, request.size.unwrap_or(0)).await?;

    Ok(Json(ListAuditLogsResponse {
        logs,
        can_load_more,
    }))
}
