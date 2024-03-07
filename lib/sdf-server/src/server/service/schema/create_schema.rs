use axum::Json;
use dal::{Schema, Visibility};
use serde::{Deserialize, Serialize};

use super::SchemaResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchemaRequest {
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchemaResponse {
    pub schema: Schema,
}

// I believe this is dead code now - worth cleaning up. -- Adam, 2024-01-29
pub async fn create_schema(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateSchemaRequest>,
) -> SchemaResult<Json<CreateSchemaResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let schema = Schema::new(&ctx, &request.name).await?;
    let response = CreateSchemaResponse { schema };

    ctx.commit().await?;

    Ok(Json(response))
}
