use std::collections::HashMap;

use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};

use dal::func::binding::FuncBinding;
use dal::func::intrinsics::IntrinsicFunc;
use dal::pkg::import_pkg_from_pkg;
use dal::schema::variant::{SchemaVariantJson, SchemaVariantMetadataJson};
use dal::{
    ChangeSet, ComponentType, Func, FuncBackendKind, FuncBackendResponseType, SchemaVariantId,
    Visibility,
};
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, PkgSpec, SiPkg,
};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
// use crate::server::tracking::track;
use crate::service::variant::{
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
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
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

    let mut asset_func_spec_builder = FuncSpec::builder();
    asset_func_spec_builder.name(asset_func.name.clone());
    asset_func_spec_builder.unique_id(asset_func.id.to_string());
    let mut func_spec_data_builder = FuncSpecData::builder();
    func_spec_data_builder
        .name(asset_func.name.clone())
        .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
        .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
        .hidden(asset_func.hidden);
    if let Some(code) = asset_func.code_plaintext()? {
        func_spec_data_builder.code_plaintext(code);
    }
    if let Some(handler) = asset_func.handler.clone() {
        func_spec_data_builder.handler(handler);
    }
    if let Some(description) = asset_func.description.clone() {
        func_spec_data_builder.description(description);
    }
    if let Some(display_name) = asset_func.display_name.clone() {
        func_spec_data_builder.display_name(display_name);
    }
    let asset_func_spec = asset_func_spec_builder
        .data(func_spec_data_builder.build()?)
        .build()?;

    let (_, return_value) =
        FuncBinding::create_and_execute(&ctx, serde_json::Value::Null, asset_func.id, vec![])
            .await?;

    if let Some(error) = return_value
        .value()
        .ok_or(SchemaVariantError::FuncExecution(asset_func.id))?
        .as_object()
        .ok_or(SchemaVariantError::FuncExecution(asset_func.id))?
        .get("error")
        .and_then(|e| e.as_str())
    {
        return Err(SchemaVariantError::FuncExecutionFailure(error.to_owned()));
    }

    let func_resp = return_value
        .value()
        .ok_or(SchemaVariantError::FuncExecution(asset_func.id))?
        .as_object()
        .ok_or(SchemaVariantError::FuncExecution(asset_func.id))?
        .get("definition")
        .ok_or(SchemaVariantError::FuncExecution(asset_func.id))?;

    let definition = serde_json::from_value::<SchemaVariantJson>(func_resp.to_owned())?;
    let metadata = SchemaVariantMetadataJson {
        name: request.name.clone(),
        menu_name: request.display_name.clone(),
        category: request.category,
        color: request.color,
        component_type: ComponentType::Component,
        link: request.link.clone(),
        description: request.description.clone(),
    };

    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;
    let variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_spec.unique_id,
    )?;
    let schema_spec = metadata.to_spec(variant_spec)?;
    let pkg_spec = PkgSpec::builder()
        .name(metadata.clone().name)
        //TODO @stack72 - figure out how we get the current user in this!
        .created_by("sally@systeminit.com".to_string())
        .func(identity_func_spec)
        .func(asset_func_spec.clone())
        .schema(schema_spec)
        .version("0.0.1")
        .build()?;

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
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&CreateVariantResponse {
        id: schema_variant_id,
        success: true,
    })?)?)
}
