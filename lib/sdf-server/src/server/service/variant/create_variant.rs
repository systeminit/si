use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, SchemaId, Visibility, WsEvent};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub description: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantResponse {
    pub id: SchemaId,
    pub success: bool,
}

pub async fn create_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let created_schema_variant = VariantAuthoringClient::create_variant(
        &ctx,
        request.name.clone(),
        request.display_name.clone(),
        request.description.clone(),
        request.link.clone(),
        request.category.clone(),
        request.color.clone(),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_variant",
        serde_json::json!({
            "variant_name": request.name.clone(),
            "variant_category": request.category.clone(),
            "variant_menu_name": request.display_name.clone(),
            "variant_id": created_schema_variant.id().clone(),
        }),
    );

    let schema = created_schema_variant.schema(&ctx).await?;

    WsEvent::schema_variant_created(&ctx, created_schema_variant.id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&CreateVariantResponse {
        id: schema.id(),
        success: true,
    })?)?)
}
