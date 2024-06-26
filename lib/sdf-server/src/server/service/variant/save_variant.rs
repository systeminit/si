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
    pub variant: si_frontend_types::SchemaVariant,
    pub code: Option<String>,
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
        request.schema_name.clone(),
        request.name.clone(),
        request.display_name.clone(),
        request.link.clone(),
        request.code.clone(),
        request.description.clone(),
        request.category.clone(),
        request.component_type,
        request.color.clone(),
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
                "variant_display_name": request.display_name,
        }),
    );

    WsEvent::schema_variant_saved(
        &ctx,
        request.id,
        request.default_schema_variant_id,
        request.name,
        request.category,
        request.color,
        request.component_type,
        request.link,
        request.description,
        request.display_name,
    )
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
