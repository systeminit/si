use std::collections::HashSet;

use axum::{
    extract::{OriginalUri, Path, Query},
    Json,
};
use dal::{ChangeSetId, WorkspacePk};
use serde::{Deserialize, Serialize};
use si_events::{
    audit_log::{AuditLogKind, AuditLogService},
    UserPk,
};
use si_frontend_types as frontend_types;

use super::AuditLogResult;
use crate::extract::{AccessBuilder, HandlerContext, PosthogClient};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsRequest {
    page: Option<usize>,
    page_size: Option<usize>,
    sort_timestamp_ascending: Option<bool>,
    exclude_system_user: Option<bool>,
    kind_filter: Option<Vec<AuditLogKind>>,
    service_filter: Option<Vec<AuditLogService>>,
    change_set_filter: Option<Vec<ChangeSetId>>,
    user_filter: Option<Vec<UserPk>>,
}

#[derive(Deserialize, Serialize, Debug)]
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
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<ListAuditLogsRequest>,
) -> AuditLogResult<Json<ListAuditLogsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // NOTE(nick): right now, we just generate a ton of logs and then apply our filters. This is
    // obviously wasteful, but this will be driven with real data, real queries and real database
    // goodies in the future.
    let audit_logs = dal::audit_log::generate(&ctx, 200).await?;

    // NOTE(nick): this will be replaced with real queries.
    let (filtered_and_paginated_audit_logs, total) = dal::audit_log::filter_and_paginate(
        audit_logs,
        request.page,
        request.page_size,
        request.sort_timestamp_ascending,
        request.exclude_system_user,
        match request.kind_filter {
            Some(provided) => HashSet::from_iter(provided.into_iter()),
            None => HashSet::new(),
        },
        match request.service_filter {
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
