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

use super::{
    DiagramError,
    DiagramResult,
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
) -> DiagramResult<Json<GetDiagramResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let ctx_clone = ctx.clone();

    let response = slow_rt::spawn(async move {
        let ctx = &ctx_clone;
        Ok::<Diagram, DiagramError>(Diagram::assemble_for_default_view(ctx).await?)
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
