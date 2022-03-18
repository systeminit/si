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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetSchemaRequest>,
) -> SchemaResult<Json<GetSchemaResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let response = Schema::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        &request.schema_id,
    )
    .await?
    .ok_or(SchemaError::SchemaNotFound)?;

    txns.commit().await?;
    Ok(Json(response))
}
