use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, SchemaVariantId, Visibility, WsEvent};
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
    Json(SaveVariantRequest {
        variant,
        code,
        visibility,
    }): Json<SaveVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let variant_id: SchemaVariantId = variant.schema_variant_id.into();

    VariantAuthoringClient::save_variant_content(
        &ctx,
        variant_id,
        &variant.schema_name,
        variant.display_name.clone(),
        variant.category.clone(),
        variant.description.clone(),
        variant.link.clone(),
        variant.color.clone(),
        variant.component_type.into(),
        code,
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "save_variant",
        serde_json::json!({
                "variant_id": variant_id,
                "variant_category": variant.category.clone(),
                "variant_name": variant.schema_name.clone(),
                "variant_display_name": variant.display_name.clone(),
        }),
    );

    WsEvent::schema_variant_saved(
        &ctx,
        variant.schema_id.into(),
        variant_id,
        variant.schema_name,
        variant.category,
        variant.color,
        variant.component_type.into(),
        variant.link,
        variant.description,
        variant.display_name,
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
