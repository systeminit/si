use std::collections::HashSet;

use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use dal::audit_logging;
use serde::{Deserialize, Serialize};
use si_events::UserPk;
use si_frontend_types as frontend_types;

use super::AuditLogResult;
use crate::extract::{AccessBuilder, HandlerContext, PosthogClient, QueryWithVecParams};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsRequest {
    page: Option<usize>,
    page_size: Option<usize>,
    sort_timestamp_ascending: Option<bool>,
    exclude_system_user: Option<bool>,
    kind_filter: Option<Vec<String>>,
    change_set_filter: Option<Vec<si_events::ChangeSetId>>,
    user_filter: Option<Vec<UserPk>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsResponse {
    logs: Vec<frontend_types::AuditLog>,
    total: usize,
}

pub async fn list_audit_logs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(dal::WorkspacePk, dal::ChangeSetId)>,
    QueryWithVecParams(request): QueryWithVecParams<ListAuditLogsRequest>,
) -> AuditLogResult<Json<ListAuditLogsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // TODO(nick): filter and paginate in the same request.
    let audit_logs = audit_logging::list(&ctx).await?;

    // TODO(nick): repalce this with the above.
    let (filtered_and_paginated_audit_logs, total) = audit_logging::temporary::filter_and_paginate(
        audit_logs,
        request.page,
        request.page_size,
        request.sort_timestamp_ascending,
        request.exclude_system_user,
        match request.kind_filter {
            Some(provided) => HashSet::from_iter(provided.into_iter()),
            None => HashSet::new(),
        },
        match request.change_set_filter {
            Some(provided) => HashSet::from_iter(provided.into_iter()),
            None => HashSet::new(),
        },
        match request.user_filter {
            Some(provided) => HashSet::from_iter(provided.into_iter()),
            None => HashSet::new(),
        },
    )?;

    Ok(Json(ListAuditLogsResponse {
        logs: filtered_and_paginated_audit_logs,
        total,
    }))
}
