use axum::extract::Query;
use axum::Json;
use dal::{ExternalProvider, InternalProvider, SchemaVariantId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::provider::ProviderResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListAllProviderRequest {
    pub schema_variant_id: SchemaVariantId,
    pub workspace_id: Option<WorkspaceId>,
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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListAllProviderRequest>,
) -> ProviderResult<Json<ListAllProviderResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

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
