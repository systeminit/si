use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Query,
    },
};
use dal::{
    diagram::Diagram,
    slow_rt,
};
use sdf_core::tracking::track;
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;

use super::DiagramResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn get_all_components_and_edges(
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<Request>,
) -> DiagramResult<Json<Diagram>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let ctx_clone = ctx.clone();

    let response = slow_rt::spawn(async move {
        let ctx = &ctx_clone;
        Diagram::assemble(ctx, None).await
    })?
    .await??;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "get_all_components_and_edges",
        serde_json::json!({
            "how": "/diagram/get_all_components_and_edges",
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(response))
}
