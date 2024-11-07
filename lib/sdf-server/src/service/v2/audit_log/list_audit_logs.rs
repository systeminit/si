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

#[remain::sorted]
#[derive(Deserialize, Debug)]
pub enum UserFilter {
    System,
    #[serde(untagged)]
    User(UserPk),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListAuditLogsRequest {
    page: Option<usize>,
    page_size: Option<usize>,
    sort_timestamp_ascending: Option<bool>,
    change_set_filter: Option<Vec<si_events::ChangeSetId>>,
    entity_type_filter: Option<Vec<String>>,
    kind_filter: Option<Vec<String>>,
    user_filter: Option<Vec<UserFilter>>,
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

    let (paginated_and_filtered_audit_logs, filtered_audit_logs_total) = audit_logging::list(
        &ctx,
        request.page.unwrap_or(0),
        request.page_size.unwrap_or(0),
        request.sort_timestamp_ascending.unwrap_or(false),
        match request.change_set_filter {
            Some(provided) => HashSet::from_iter(provided.into_iter()),
            None => HashSet::new(),
        },
        match request.entity_type_filter {
            Some(provided) => HashSet::from_iter(provided.into_iter()),
            None => HashSet::new(),
        },
        match request.kind_filter {
            Some(provided) => HashSet::from_iter(provided.into_iter()),
            None => HashSet::new(),
        },
        match request.user_filter {
            Some(provided) => HashSet::from_iter(provided.iter().map(|u| match u {
                UserFilter::System => None,
                UserFilter::User(user_id) => Some(*user_id),
            })),
            None => HashSet::new(),
        },
    )
    .await?;

    Ok(Json(ListAuditLogsResponse {
        logs: paginated_and_filtered_audit_logs,
        total: filtered_audit_logs_total,
    }))
}
