use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant_definition::SchemaVariantResult;
use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::schema::variant::SchemaVariantMetadataView;
use dal::Visibility;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantDefsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantDefsResponse {
    pub variant_defs: Vec<SchemaVariantMetadataView>,
}

pub async fn list_variant_defs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<ListVariantDefsRequest>,
) -> SchemaVariantResult<Json<ListVariantDefsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let schema_variant_definition_views = SchemaVariantMetadataView::list(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_variant_def",
        serde_json::json!({}),
    );

    Ok(Json(ListVariantDefsResponse {
        variant_defs: schema_variant_definition_views,
    }))
}
