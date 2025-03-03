use anyhow::Result;
use axum::extract::{Json, Query};
use dal::Visibility;
use serde::{Deserialize, Serialize};

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
) -> Result<Json<DvuRootsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let count = ctx
        .workspace_snapshot()?
        .get_dependent_value_roots()
        .await?
        .len();

    Ok(Json(DvuRootsResponse { count }))
}
