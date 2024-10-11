use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use dal::{ChangeSetId, WorkspacePk};
use si_frontend_types as frontend_types;

use super::AuditLogResult;
use crate::extract::{AccessBuilder, HandlerContext, PosthogClient};

pub async fn list_audit_logs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> AuditLogResult<Json<Vec<frontend_types::AuditLog>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let audit_logs = dal::audit_log::generate(&ctx).await?;

    Ok(Json(audit_logs))
}
