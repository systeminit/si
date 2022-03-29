use axum::extract::Query;
use axum::Json;
use dal::{Schema, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

use super::SchemaResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
pub struct ListSchemaRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemaResponse {
    pub list: Vec<Schema>,
}

pub async fn list_schemas(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListSchemaRequest>,
) -> SchemaResult<Json<ListSchemaResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let list = Schema::list(&ctx).await?;
    let response = ListSchemaResponse { list };
    Ok(Json(response))
}
