use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
    http::StatusCode,
    response::IntoResponse,
};
use dal::{
    ChangeSet,
    WorkspacePk,
    WsEvent,
};
use sdf_extract::{
    EddaClient,
    FriggStore,
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
    service::v2::{
        AccessBuilder,
        change_set::create_index_for_new_change_set_and_watch,
    },
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub name: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    FriggStore(frigg): FriggStore,
    EddaClient(edda_client): EddaClient,
    Path(workspace_pk): Path<WorkspacePk>,
    Json(Request { name }): Json<Request>,
) -> Result<impl IntoResponse> {
    let ctx = builder.build_head(request_ctx).await?;

    let change_set_name = name.to_owned();

    let change_set = ChangeSet::fork_head(&ctx, change_set_name.clone()).await?;
    let change_set_id = change_set.id;

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
    if create_index_for_new_change_set_and_watch(
        &frigg,
        &edda_client,
        workspace_pk,
        change_set_id,
        ctx.change_set_id(), // note DalCtx is built for Head here to create a change set!
        ctx.workspace_snapshot()?.address().await,
    )
    .await?
    {
        // Return 200 if the build succeeded in time
        Ok((StatusCode::OK, Json(change_set)))
    } else {
        // Return 202 Accepted with the same response body if the build didn't succeed in time
        // to let the caller know the create succeeded, we're just waiting on downstream work
        Ok((StatusCode::ACCEPTED, Json(change_set)))
    }
}
