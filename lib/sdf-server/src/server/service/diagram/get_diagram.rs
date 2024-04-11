use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::diagram::Diagram;
use dal::Visibility;
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiagramRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetDiagramResponse = Diagram;

pub async fn get_diagram(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetDiagramRequest>,
) -> DiagramResult<Json<GetDiagramResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let response = Diagram::assemble(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_diagram",
        serde_json::json!({
            "how": "/diagram/get_diagram",
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(response))
}
