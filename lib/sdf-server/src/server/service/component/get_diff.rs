use axum::{extract::Query, Json};
use dal::component::diff::ComponentDiff;
use dal::{ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiffRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiffResponse {
    pub component_diff: ComponentDiff,
}

pub async fn get_diff(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetDiffRequest>,
) -> ComponentResult<Json<GetDiffResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component_diff = ComponentDiff::new(&ctx, request.component_id).await?;

    Ok(Json(GetDiffResponse { component_diff }))
}
