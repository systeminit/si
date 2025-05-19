use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSet,
    WorkspacePk,
    WsEvent,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;

use super::Result;
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::v2::AccessBuilder,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub name: String,
}

pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path(_workspace_pk): Path<WorkspacePk>,
    Json(Request { name }): Json<Request>,
) -> Result<Json<si_frontend_types::ChangeSet>> {
    let ctx = builder.build_head(request_ctx).await?;

    let change_set_name = name.to_owned();

    let change_set = ChangeSet::fork_head(&ctx, change_set_name.clone()).await?;

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

    WsEvent::change_set_created(&ctx, change_set.id, change_set.workspace_snapshot_address)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let change_set = change_set.into_frontend_type(&ctx).await?;
    ctx.commit_no_rebase().await?;

    Ok(Json(change_set))
}
