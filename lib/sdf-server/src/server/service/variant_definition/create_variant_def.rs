use super::SchemaVariantDefinitionResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{
    component::ComponentKind,
    schema::variant::definition::{SchemaVariantDefinition, SchemaVariantDefinitionId},
    Func, FuncBackendKind, FuncBackendResponseType, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

const DEFAULT_ASSET_CODE: &str = r#"function createAsset() {
  const asset = new AssetBuilder();
  return asset.build()
}"#;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantDefRequest {
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub description: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantDefResponse {
    pub id: SchemaVariantDefinitionId,
    pub success: bool,
}

pub async fn create_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<CreateVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut asset_func = Func::new(
        &ctx,
        request.name.clone(),
        FuncBackendKind::JsSchemaVariantDefinition,
        FuncBackendResponseType::SchemaVariantDefinition,
    )
    .await?;
    asset_func.set_handler(&ctx, Some("createAsset")).await?;
    asset_func
        .set_code_plaintext(&ctx, Some(DEFAULT_ASSET_CODE))
        .await?;

    let variant_def = SchemaVariantDefinition::new(
        &ctx,
        request.name,
        request.menu_name.clone(),
        request.category,
        request.link,
        request.color,
        ComponentKind::Standard,
        request.description,
        *asset_func.id(),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_variant_def",
        serde_json::json!({
                    "variant_def_name": variant_def.name(),
                    "variant_def_category": variant_def.category(),
                    "variant_def_menu_name": variant_def.menu_name(),
                    "variant_def_id": variant_def.id(),
                    "variant_def_component_type": variant_def.component_type(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(CreateVariantDefResponse {
        id: *variant_def.id(),
        success: true,
    }))
}
