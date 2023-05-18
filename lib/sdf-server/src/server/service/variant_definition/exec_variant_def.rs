use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::pkg::import_pkg_from_pkg;
use dal::{
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionId, SchemaVariantDefinitionJson,
        SchemaVariantDefinitionMetadataJson,
    },
    schema::SchemaUiMenu,
    Schema, SchemaVariant, SchemaVariantId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncUniqueId, PkgSpec, SiPkg,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefResponse {
    pub success: bool,
}

pub async fn exec_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ExecVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<ExecVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    let metadata: SchemaVariantDefinitionMetadataJson = variant_def.clone().into();
    let definition: SchemaVariantDefinitionJson = variant_def.try_into()?;

    // we need to change this to use the PkgImport
    let identity_func_spec = FuncSpec::builder()
        .name("si:identity")
        .handler("si:identity")
        .code_base64("")
        .response_type(FuncSpecBackendResponseType::Json)
        .backend_kind(FuncSpecBackendKind::JsAttribute)
        .hidden(false)
        .build()?;
    let variant_spec = definition.to_spec(metadata, identity_func_spec.unique_id)?;
    let schema_spec = metadata.to_spec(variant_spec)?;
    let pkg_spec = PkgSpec::builder()
        .name(metadata.name.to_owned())
        .created_by("sally@systeminit.com")
        .func(identity_func_spec)
        .schema(schema_spec)
        .build()?;

    let pkg = SiPkg::load_from_spec(pkg_spec)?;
    import_pkg_from_pkg(&ctx, &pkg, metadata.name.as_str()).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(ExecVariantDefResponse { success: true }))
}
