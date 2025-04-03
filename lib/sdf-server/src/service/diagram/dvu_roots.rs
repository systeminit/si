use axum::extract::{Json, Query};
use dal::{workspace_snapshot::DependentValueRoot, Visibility};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::extract::{v1::AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DvuRootsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DvuRootsResponse {
    count: usize,
}

pub async fn dvu_roots(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<DvuRootsRequest>,
) -> DiagramResult<Json<DvuRootsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let count = DependentValueRoot::get_dependent_value_roots(&ctx)
        .await?
        .len();

    Ok(Json(DvuRootsResponse { count }))
}
