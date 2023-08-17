use super::SchemaVariantDefinitionResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::ComponentType;
use dal::{schema::variant::definition::SchemaVariantDefinitionId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub code: String,
    pub handler: String,
    pub description: Option<String>,
    pub component_type: ComponentType,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveVariantDefResponse {
    pub success: bool,
}

pub async fn save_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SaveVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<SaveVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    super::save_variant_def(&ctx, &request).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "save_variant_def",
        serde_json::json!({
                    "variant_def_category": request.category,
                    "variant_def_name": request.name,
                    "variant_def_menu_name": request.menu_name,
                    // "variant_def_definition":  request.definition,
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(SaveVariantDefResponse { success: true }))
}
