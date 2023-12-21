use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::service::change_set::ChangeSetError;
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::change_set_pointer::{ChangeSetPointer, ChangeSetPointerId};
use dal::ChangeSetStatus;
use serde::{Deserialize, Serialize};
//use telemetry::tracing::{info_span, Instrument, log::warn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    pub change_set_pk: ChangeSetPointerId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetResponse {
    pub change_set: ChangeSetPointer,
}

// TODO: This does not handle anything related to actions yet, after the switchover to workspace
//       snapshot graphs.
pub async fn apply_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ApplyChangeSetRequest>,
) -> ChangeSetResult<Json<ApplyChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let mut change_set = ChangeSetPointer::find(&ctx, request.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.apply_to_base_change_set(&ctx).await?;
    change_set
        .update_status(&ctx, ChangeSetStatus::Applied)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": request.change_set_pk,
        }),
    );

    ctx.blocking_commit().await?;

    Ok(Json(ApplyChangeSetResponse { change_set }))
}
