use axum::{extract::Query, Json};
use dal::{system::SystemId, Diagram, Visibility};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiagramRequest {
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetDiagramResponse = Diagram;

pub async fn get_diagram(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetDiagramRequest>,
) -> DiagramResult<Json<GetDiagramResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let response = Diagram::assemble(&ctx, request.system_id).await?;

    Ok(Json(response))
}
