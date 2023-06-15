use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::func::intrinsics::IntrinsicFunc;
use dal::installed_pkg::{InstalledPkgAsset, InstalledPkgAssetAssetId, InstalledPkgAssetKind};
use dal::pkg::import_pkg_from_pkg;
use dal::{
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionId, SchemaVariantDefinitionJson,
        SchemaVariantDefinitionMetadataJson,
    },
    StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_pkg::{PkgSpec, SiPkg};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

// Should move this to the modules service
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPkgAssetView {
    pub asset_id: InstalledPkgAssetAssetId,
    pub asset_kind: InstalledPkgAssetKind,
    pub asset_hash: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefResponse {
    pub success: bool,
    pub installed_pkg_assets: Vec<InstalledPkgAssetView>,
}

pub async fn exec_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
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
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;
    let variant_spec = definition.to_spec(metadata.clone(), identity_func_spec.unique_id)?;
    let schema_spec = metadata.to_spec(variant_spec)?;
    let pkg_spec = PkgSpec::builder()
        .name(metadata.clone().name)
        .created_by("sally@systeminit.com")
        .func(identity_func_spec)
        .schema(schema_spec)
        .version("0.0.1")
        .build()?;

    let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;
    let installed_pkg_id = import_pkg_from_pkg(
        &ctx,
        &pkg,
        metadata.clone().name.as_str(),
        Some(dal::pkg::ImportOptions {
            schemas: None,
            no_definitions: true,
        }),
    )
    .await?;

    let installed_pkg_assets =
        InstalledPkgAsset::list_for_installed_pkg_id(&ctx, installed_pkg_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "exec_variant_def",
        serde_json::json!({
                    "variant_def_category": metadata.clone().category,
                    "variant_def_name": metadata.clone().name,
                    "variant_def_version": pkg_spec.clone().version,
                    "variant_def_schema_count":  pkg_spec.clone().schemas.len(),
                    "variant_def_function_count":  pkg_spec.clone().funcs.len(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(ExecVariantDefResponse {
        success: true,
        installed_pkg_assets: installed_pkg_assets
            .iter()
            .map(|ipa| InstalledPkgAssetView {
                asset_id: ipa.asset_id(),
                asset_hash: ipa.asset_hash().into(),
                asset_kind: ipa.asset_kind().to_owned(),
            })
            .collect(),
    }))
}
