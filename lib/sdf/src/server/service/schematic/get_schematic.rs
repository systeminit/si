use axum::{extract::Query, Json};
use dal::{system::SystemId, Schematic, Visibility};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchematicRequest {
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetSchematicResponse = Schematic;

pub async fn get_schematic(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetSchematicRequest>,
) -> SchematicResult<Json<GetSchematicResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let response = Schematic::find(&ctx, request.system_id).await?;

    txns.commit().await?;
    Ok(Json(response))
}
