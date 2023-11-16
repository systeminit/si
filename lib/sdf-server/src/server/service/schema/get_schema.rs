use axum::{extract::Query, Json};
use dal::{Schema, SchemaId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

use super::{SchemaError, SchemaResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaRequest {
    pub schema_id: SchemaId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetSchemaResponse = Schema;

pub async fn get_schema(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetSchemaRequest>,
) -> SchemaResult<Json<GetSchemaResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let response = Schema::get_by_id(&ctx, request.schema_id).await?;

    Ok(Json(response))
}
