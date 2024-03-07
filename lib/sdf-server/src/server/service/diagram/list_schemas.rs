use axum::extract::{Json, Query};
use dal::schema::view::SchemaView;
use dal::Visibility;
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemaVariantsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ListSchemasResponse = Vec<SchemaView>;

pub async fn list_schemas(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListSchemaVariantsRequest>,
) -> DiagramResult<Json<ListSchemasResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let views = SchemaView::list(&ctx).await?;

    Ok(Json(views))
}
