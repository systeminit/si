use std::collections::HashSet;

use axum::{
    extract::{OriginalUri, Path},
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
    kind_filter: HashSet<AuditLogKind>,
    service_filter: HashSet<AuditLogService>,
    change_set_filter: HashSet<ChangeSetId>,
    user_filter: HashSet<UserPk>,
}

pub async fn list_audit_logs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<ListAuditLogsRequest>,
) -> AuditLogResult<Json<Vec<frontend_types::AuditLog>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // NOTE(nick): right now, we just generate a ton of logs and then apply our filters. This is
    // obviously wasteful, but this will be driven with real data, real queries and real database
    // goodies in the future.
    let audit_logs = dal::audit_log::generate(&ctx, 200).await?;

    // NOTE(nick): this will be replaced with real queries.
    let filtered_and_paginated_audit_logs = dal::audit_log::filter_and_paginate(
        audit_logs,
        request.page,
        request.page_size,
        request.sort_timestamp_ascending,
        request.exclude_system_user,
        request.kind_filter,
        request.service_filter,
        request.change_set_filter,
        request.user_filter,
    )?;

    Ok(Json(filtered_and_paginated_audit_logs))
}
