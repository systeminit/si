use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;
use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::schema::variant::SchemaVariantMetadataView;
use dal::Visibility;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListVariantsResponse {
    pub variants: Vec<SchemaVariantMetadataView>,
}

pub async fn list_variants(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<ListVariantsRequest>,
) -> SchemaVariantResult<Json<ListVariantsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let schema_variant_metadata_views = SchemaVariantMetadataView::list(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_variant_def",
        serde_json::json!({}),
    );

    Ok(Json(ListVariantsResponse {
        variants: schema_variant_metadata_views,
    }))
}
