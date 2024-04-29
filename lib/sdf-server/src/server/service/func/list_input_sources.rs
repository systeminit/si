use axum::{extract::Query, Json};
use dal::input_sources::InputSources;
use dal::{SchemaVariantId, Visibility};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListInputSourcesRequest {
    schema_variant_id: Option<SchemaVariantId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ListInputSourcesResponse = InputSources;

pub async fn list_input_sources(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListInputSourcesRequest>,
) -> FuncResult<Json<ListInputSourcesResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let input_sources = match request.schema_variant_id {
        Some(provided_schema_variant_id) => {
            InputSources::assemble(&ctx, provided_schema_variant_id).await?
        }
        None => InputSources::assemble_for_all_schema_variants(&ctx).await?,
    };

    Ok(Json(input_sources))
}
