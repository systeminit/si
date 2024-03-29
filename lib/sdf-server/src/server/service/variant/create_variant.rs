use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
// use crate::server::tracking::track;
use crate::service::variant::SchemaVariantResult;
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use base64::engine::general_purpose;
use base64::Engine;
use dal::{
    ChangeSetPointer, ComponentType, Func, FuncBackendKind, FuncBackendResponseType, Schema,
    SchemaVariant, SchemaVariantId, Visibility,
};
use serde::{Deserialize, Serialize};

const DEFAULT_ASSET_CODE: &str = r#"function main() {
  return new AssetBuilder().build()
}"#;

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
    pub id: SchemaVariantId,
    pub success: bool,
}

pub async fn create_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Json(request): Json<CreateVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSetPointer::force_new(&mut ctx).await?;

    let code_base64 = general_purpose::STANDARD_NO_PAD.encode(DEFAULT_ASSET_CODE);
    let asset_func = Func::new(
        &ctx,
        request.name.clone(),
        request.display_name.clone(),
        request.description.clone(),
        request.link.clone(),
        false,
        false,
        FuncBackendKind::JsSchemaVariantDefinition,
        FuncBackendResponseType::SchemaVariantDefinition,
        Some("main"),
        Some(code_base64),
    )
    .await?;

    let schema = Schema::new(&ctx, request.name.clone()).await?;
    let (variant, _) = SchemaVariant::new(
        &ctx,
        schema.id(),
        request.name.clone(),
        request.display_name.clone(),
        request.category,
        request.color,
        ComponentType::Component,
        request.link.clone(),
        request.description.clone(),
        Some(asset_func.id),
    )
    .await?;

    schema
        .set_default_schema_variant(&ctx, variant.id())
        .await?;

    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "create_variant_def",
    //     serde_json::json!({
    //                 "variant_def_name": variant_def.name(),
    //                 "variant_def_category": variant_def.category(),
    //                 "variant_def_menu_name": variant_def.menu_name(),
    //                 "variant_def_id": variant_def.id(),
    //                 "variant_def_component_type": variant_def.component_type(),
    //     }),
    // );
    //
    // WsEvent::schema_variant_definition_created(&ctx, *variant_def.id())
    //     .await?
    //     .publish_on_commit(&ctx)
    //     .await?;
    //
    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(serde_json::to_string(&CreateVariantResponse {
        id: variant.id(),
        success: true,
    })?)?)
}
