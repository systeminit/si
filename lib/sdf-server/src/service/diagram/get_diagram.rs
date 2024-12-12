use axum::{
    extract::{Host, OriginalUri, Query},
    Json,
};
use dal::{diagram::Diagram, slow_rt, Visibility};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    routes::AppError,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiagramRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetDiagramResponse = Diagram;

pub async fn get_diagram(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetDiagramRequest>,
) -> Result<Json<GetDiagramResponse>, AppError> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let ctx_clone = ctx.clone();

    let response = slow_rt::spawn(async move {
        let ctx = &ctx_clone;
        Ok::<Diagram, anyhow::Error>(Diagram::assemble_for_default_view(ctx).await?)
    })?
    .await??;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "get_diagram",
        serde_json::json!({
            "how": "/diagram/get_diagram",
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(response))
}
