use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, ComponentType, SchemaId, SchemaVariantId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SaveVariantRequest {
    pub id: SchemaId,
    pub default_schema_variant_id: SchemaVariantId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub code: String,
    pub description: Option<String>,
    pub component_type: ComponentType,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveVariantResponse {
    pub success: bool,
}

pub async fn save_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SaveVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    VariantAuthoringClient::save_variant_content(
        &ctx,
        request.default_schema_variant_id,
        request.name.clone(),
        request.menu_name.clone(),
        request.link.clone(),
        request.code.clone(),
        request.description.clone(),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "save_variant",
        serde_json::json!({
                "variant_id": request.id,
                "variant_category": request.category,
                "variant_name": request.name,
                "variant_menu_name": request.menu_name,
        }),
    );

    WsEvent::schema_variant_saved(&ctx, request.default_schema_variant_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(&SaveVariantResponse {
        success: true,
    })?)?)
}
