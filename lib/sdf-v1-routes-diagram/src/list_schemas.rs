use axum::extract::{Json, Query};
use dal::{Visibility, schema::view::SchemaView};
use sdf_extract::{HandlerContext, v1::AccessBuilder};
use serde::{Deserialize, Serialize};

use super::DiagramResult;

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
