use std::collections::HashMap;

use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};

use dal::pkg::import_pkg_from_pkg;
use dal::schema::variant::SchemaVariantMetadataJson;
use dal::{
    ChangeSet, ComponentType, Func, FuncBackendKind, FuncBackendResponseType, SchemaVariantId,
    Visibility, WsEvent,
};
use si_pkg::SiPkg;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::{
    build_asset_func_spec, build_pkg_spec_for_variant, execute_asset_func,
    generate_scaffold_func_name, SchemaVariantError, SchemaVariantResult,
};

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
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let code_base64 = general_purpose::STANDARD_NO_PAD.encode(DEFAULT_ASSET_CODE);
    let asset_func = Func::new(
        &ctx,
        generate_scaffold_func_name(request.name.clone()),
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

    let asset_func_spec = build_asset_func_spec(&asset_func)?;
    let definition = execute_asset_func(&ctx, &asset_func).await?;

    let metadata = SchemaVariantMetadataJson {
        name: request.name.clone(),
        menu_name: request.display_name.clone(),
        category: request.category.clone(),
        color: request.color,
        component_type: ComponentType::Component,
        link: request.link.clone(),
        description: request.description.clone(),
    };

    //TODO @stack72 - figure out how we get the current user in this!
    let pkg_spec = build_pkg_spec_for_variant(
        definition,
        &asset_func_spec,
        &metadata,
        "sally@systeminit.com",
    )?;

    let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;

    let (_, schema_variant_ids, _) = import_pkg_from_pkg(
        &ctx,
        &pkg,
        Some(dal::pkg::ImportOptions {
            schemas: None,
            skip_import_funcs: Some(HashMap::from_iter([(
                asset_func_spec.unique_id.to_owned(),
                asset_func.clone(),
            )])),
            no_record: true,
            is_builtin: false,
        }),
    )
    .await?;

    let schema_variant_id = schema_variant_ids
        .first()
        .copied()
        .ok_or(SchemaVariantError::NoAssetCreated)?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "create_variant",
        serde_json::json!({
            "variant_name": request.name.clone(),
            "variant_category": request.category.clone(),
            "variant_menu_name": request.display_name.clone(),
            "variant_id": schema_variant_id.clone(),
        }),
    );

    WsEvent::schema_variant_created(&ctx, schema_variant_id)
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
        id: schema_variant_id,
        success: true,
    })?)?)
}
