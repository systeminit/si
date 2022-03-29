use super::ApplicationResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{Component, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationRequest {
    pub name: String,
    pub workspace_id: WorkspaceId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateApplicationResponse {
    pub application: Component,
}

pub async fn create_application(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateApplicationRequest>,
) -> ApplicationResult<Json<CreateApplicationResponse>> {
    let txns = txns.start().await?;
    // You can only create applications directly to head? This feels wrong, but..
    let ctx = builder.build(request_ctx.build_head(), &txns);

    let (application, _application_node) =
        Component::new_application_with_node(&ctx, &request.name).await?;

    txns.commit().await?;

    Ok(Json(CreateApplicationResponse { application }))
}
