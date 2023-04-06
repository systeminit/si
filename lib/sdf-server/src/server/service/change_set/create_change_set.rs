use axum::extract::OriginalUri;
use axum::Json;
use dal::ChangeSet;
use serde::{Deserialize, Serialize};

use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetRequest {
    pub change_set_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<Json<CreateChangeSetResponse>> {
    let ctx = builder.build(request_ctx.build_head()).await?;

    let change_set_name = &request.change_set_name;
    let change_set = ChangeSet::new(&ctx, change_set_name, None).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_change_set",
        serde_json::json!({
                    "change_set_name": change_set_name,
        }),
    );

    ctx.commit().await?;

    Ok(Json(CreateChangeSetResponse { change_set }))
}
