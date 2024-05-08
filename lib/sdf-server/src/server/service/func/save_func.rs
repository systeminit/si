use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::FuncAssociations;
use dal::{ChangeSet, FuncId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::func::FuncResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncRequest {
    pub id: FuncId,
    pub display_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub associations: Option<FuncAssociations>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn save_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // Cache for posthog tracking.
    let func_id = request.id;
    let func_name = request.name.clone();

    FuncAuthoringClient::save_func(
        &ctx,
        request.id,
        request.display_name,
        request.name,
        request.description,
        request.code,
        request.associations,
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "save_func",
        serde_json::json!({
            "how": "/func/save_func",
            "func_id": func_id,
            "func_name": func_name.as_str(),
        }),
    );

    WsEvent::func_saved(&ctx, func_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
