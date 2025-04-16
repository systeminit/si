use axum::extract::{Host, OriginalUri};
use axum::Json;
use dal::change_set::ChangeSet;
use dal::WsEvent;
use si_events::audit_log::AuditLogKind;
use si_frontend_types::{CreateChangeSetRequest, CreateChangeSetResponse};

use super::ChangeSetResult;
use sdf_core::tracking::track;
use sdf_extract::{v1::AccessBuilder, HandlerContext, PosthogClient};

pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<Json<CreateChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set_name = &request.change_set_name;

    let change_set = ChangeSet::fork_head(&ctx, change_set_name).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_change_set",
        serde_json::json!({
                    "change_set_name": change_set_name.clone(),
        }),
    );

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, change_set_name.to_string())
        .await?;

    WsEvent::change_set_created(&ctx, change_set.id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let change_set = change_set.into_frontend_type(&ctx).await?;
    ctx.commit_no_rebase().await?;

    Ok(Json(CreateChangeSetResponse { change_set }))
}
