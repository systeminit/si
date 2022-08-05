use axum::{extract::Query, Json};
use dal::component::diff::ComponentDiff;
use dal::{CodeView, ComponentId, Visibility};
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
    pub code_views: Vec<CodeView>,
}

pub async fn get_diff(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetDiffRequest>,
) -> ComponentResult<Json<GetDiffResponse>> {
    let txns = txns.start().await?;

    let head_ctx = builder.build(request_ctx.build_head(), &txns);
    let ctx = head_ctx.clone_with_new_visibility(request.visibility);

    let component_diff = ComponentDiff::new(&ctx, &head_ctx, request.component_id).await?;

    txns.commit().await?;

    Ok(Json(GetDiffResponse {
        code_views: component_diff.diffs,
    }))
}
