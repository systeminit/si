use axum::Json;
use dal::ComponentKind;
use dal::{Schema, Visibility, WsEvent};
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

pub async fn create_schema(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateSchemaRequest>,
) -> SchemaResult<Json<CreateSchemaResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let schema = Schema::new(&ctx, &request.name, ComponentKind::Standard).await?;
    let response = CreateSchemaResponse { schema };

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(response))
}
