use crate::service::component::ComponentResult;
use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    track,
};
use axum::{
    extract::{Host, OriginalUri, Query},
    Json,
};
use dal::diagram::Diagram;
use dal::{slow_rt, Visibility};
use serde::{Deserialize, Serialize};

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
) -> ComponentResult<Json<Diagram>> {
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
