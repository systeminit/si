use axum::extract::Query;
use axum::Json;
use dal::{ExternalProvider, InternalProvider, SchemaVariantId, Visibility, WorkspacePk};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::provider::ProviderResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListAllProviderRequest {
    pub schema_variant_id: SchemaVariantId,
    pub workspace_pk: Option<WorkspacePk>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListAllProviderResponse {
    pub internal_providers: Vec<InternalProvider>,
    pub external_providers: Vec<ExternalProvider>,
}

pub async fn list_all_providers(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListAllProviderRequest>,
) -> ProviderResult<Json<ListAllProviderResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let internal_providers =
        InternalProvider::list_for_schema_variant(&ctx, request.schema_variant_id).await?;
    let external_providers =
        ExternalProvider::list_for_schema_variant(&ctx, request.schema_variant_id).await?;

    let response = ListAllProviderResponse {
        internal_providers,
        external_providers,
    };
    Ok(Json(response))
}
