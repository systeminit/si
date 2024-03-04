use axum::extract::OriginalUri;
use axum::Json;
use dal::change_set_pointer::ChangeSetPointer;
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
    pub change_set: ChangeSetPointer,
}

pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<Json<CreateChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set_name = &request.change_set_name;

    let change_set_pointer = ChangeSetPointer::fork_head(&ctx, change_set_name).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_change_set",
        serde_json::json!({
                    "change_set_name": change_set_name,
        }),
    );

    ctx.commit_no_rebase().await?;

    Ok(Json(CreateChangeSetResponse {
        change_set: change_set_pointer,
    }))
}
