use std::collections::HashMap;

use base64::engine::general_purpose;
use base64::Engine;
use chrono::Utc;
use convert_case::{Case, Casing};
use thiserror::Error;

use pkg::import::{clone_and_import_funcs, import_schema_variant};
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, MergeSkip, PkgSpec,
    SchemaVariantSpec, SiPkg, SiPkgError, SpecError,
};
use telemetry::prelude::*;

use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::pkg::export::PkgExporter;
use crate::pkg::{import_pkg_from_pkg, PkgError};
use crate::schema::variant::{SchemaVariantJson, SchemaVariantMetadataJson};
use crate::{
    generate_unique_id, pkg, ComponentType, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncError, FuncId, Schema, SchemaError, SchemaVariant,
    SchemaVariantError, SchemaVariantId,
};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum VariantAuthoringError {
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func execution error: {0}")]
    FuncExecution(FuncId),
    #[error("func execution failure error: {0}")]
    FuncExecutionFailure(String),
    #[error("no new asset was created")]
    NoAssetCreated,
    #[error("pkg error: {0}")]
    Pkg(#[from] PkgError),
    #[error("constructed package has no schema node")]
    PkgMissingSchema,
    #[error("constructed package has no schema variant node")]
    PkgMissingSchemaVariant,
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant asset func not found: {0}")]
    SchemaVariantAssetNotFound(SchemaVariantId),
    #[error("schema variant not found: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si pkg error: {0}")]
    SiPkg(#[from] SiPkgError),
    #[error("spec error: {0}")]
    Spec(#[from] SpecError),
}

type VariantAuthoringResult<T> = Result<T, VariantAuthoringError>;

const DEFAULT_ASSET_CODE: &str = r#"function main() {
  return new AssetBuilder().build()
}"#;

#[derive(Debug)]
pub struct VariantAuthoringClient;

impl VariantAuthoringClient {
    /// Creates a [`SchemaVariant`] and returns the [result](SchemaVariant).
    #[instrument(name = "variant.authoring.create_variant", level = "info", skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn create_variant(
        ctx: &DalContext,
        name: String,
        display_name: Option<String>,
        description: Option<String>,
        link: Option<String>,
        category: String,
        color: String,
    ) -> VariantAuthoringResult<SchemaVariant> {
        let code_base64 = general_purpose::STANDARD_NO_PAD.encode(DEFAULT_ASSET_CODE);
        let asset_func = Func::new(
            ctx,
            generate_scaffold_func_name(name.clone()),
            display_name.clone(),
            description.clone(),
            link.clone(),
            false,
            false,
            FuncBackendKind::JsSchemaVariantDefinition,
            FuncBackendResponseType::SchemaVariantDefinition,
            Some("main"),
            Some(code_base64),
        )
        .await?;

        let asset_func_spec = build_asset_func_spec(&asset_func)?;
        let definition = execute_asset_func(ctx, &asset_func).await?;

        let metadata = SchemaVariantMetadataJson {
            name: name.clone(),
            menu_name: display_name.clone(),
            category,
            color,
            component_type: ComponentType::Component,
            link: link.clone(),
            description: description.clone(),
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
            ctx,
            &pkg,
            Some(pkg::ImportOptions {
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
            .ok_or(VariantAuthoringError::NoAssetCreated)?;

        Ok(SchemaVariant::get_by_id(ctx, schema_variant_id).await?)
    }

    #[instrument(name = "variant.authoring.clone_variant", level = "info", skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn clone_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> VariantAuthoringResult<(SchemaVariant, Schema)> {
        let variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
        let schema = variant.schema(ctx).await?;

        let new_name = format!("{} Clone {}", schema.name(), generate_unique_id(4));
        let menu_name = variant.display_name().map(|mn| format!("{mn} Clone"));

        if let Some(asset_func_id) = variant.asset_func_id() {
            let old_func = Func::get_by_id_or_error(ctx, asset_func_id).await?;

            let cloned_func = old_func.duplicate(ctx, new_name.clone()).await?;
            let cloned_func_spec = build_asset_func_spec(&cloned_func)?;
            let definition = execute_asset_func(ctx, &cloned_func).await?;
            let metadata = SchemaVariantMetadataJson {
                name: new_name.clone(),
                menu_name: menu_name.clone(),
                category: variant.category().to_string(),
                color: variant.get_color(ctx).await?,
                component_type: variant.component_type(),
                link: variant.link().clone(),
                description: variant.description().clone(),
            };
            //TODO @stack72 - figure out how we get the current user in this!
            let pkg_spec = build_pkg_spec_for_variant(
                definition,
                &cloned_func_spec,
                &metadata,
                "sally@systeminit.com",
            )?;

            let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;

            let (_, schema_variant_ids, _) = import_pkg_from_pkg(
                ctx,
                &pkg,
                Some(pkg::ImportOptions {
                    schemas: None,
                    skip_import_funcs: Some(HashMap::from_iter([(
                        cloned_func_spec.unique_id.to_owned(),
                        cloned_func.clone(),
                    )])),
                    no_record: true,
                    is_builtin: false,
                }),
            )
            .await?;

            let new_schema_variant_id = schema_variant_ids
                .first()
                .copied()
                .ok_or(VariantAuthoringError::NoAssetCreated)?;

            Ok((
                SchemaVariant::get_by_id(ctx, new_schema_variant_id).await?,
                schema,
            ))
        } else {
            return Err(VariantAuthoringError::SchemaVariantAssetNotFound(
                schema_variant_id,
            ));
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(
        name = "variant.authoring.update_and_generate_variant_with_new_version",
        level = "info",
        skip_all
    )]
    pub async fn update_and_generate_variant_with_new_version(
        ctx: &DalContext,
        old_asset_func: &Func,
        current_sv_id: SchemaVariantId,
        name: String,
        menu_name: Option<String>,
        category: String,
        color: String,
        link: Option<String>,
        code: String,
        description: Option<String>,
        component_type: ComponentType,
    ) -> VariantAuthoringResult<SchemaVariantId> {
        let new_asset_func = old_asset_func
            .duplicate(ctx, generate_scaffold_func_name(name.clone()))
            .await?;

        let code_base64 = general_purpose::STANDARD_NO_PAD.encode(code);
        new_asset_func
            .clone()
            .modify(ctx, |func| {
                func.backend_kind = FuncBackendKind::JsSchemaVariantDefinition;
                func.backend_response_type = FuncBackendResponseType::SchemaVariantDefinition;
                func.display_name = menu_name
                    .clone()
                    .map(|display_name| display_name.to_owned());
                func.code_base64 = Some(code_base64);
                func.description = description.clone();
                func.handler = Some("main".to_string());
                func.hidden = false;
                func.link = link.clone();
                Ok(())
            })
            .await?;

        let asset_func_spec = build_asset_func_spec(&new_asset_func.clone())?;
        let definition = execute_asset_func(ctx, &new_asset_func).await?;

        let metadata = SchemaVariantMetadataJson {
            name: name.clone(),
            menu_name: menu_name.clone(),
            category,
            color,
            component_type,
            link: link.clone(),
            description: description.clone(),
        };

        let (new_variant_spec, _skips, variant_funcs) =
            build_variant_spec_based_on_existing_variant(
                ctx,
                definition,
                &asset_func_spec,
                &metadata,
                current_sv_id,
            )
            .await?;

        let schema_spec = metadata.to_spec(new_variant_spec)?;

        //TODO @stack72 - figure out how we get the current user in this!
        let pkg_spec = PkgSpec::builder()
            .name(&metadata.name)
            .created_by("sally@systeminit.com")
            .funcs(variant_funcs.clone())
            .func(asset_func_spec)
            .schema(schema_spec)
            .version("0")
            .build()?;
        let pkg = SiPkg::load_from_spec(pkg_spec)?;

        let pkg_schemas = pkg.schemas()?;
        let pkg_variants = pkg_schemas
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchema)?
            .variants()?;

        let schema_spec = pkg_schemas
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchema)?;
        let variant_pkg_spec = pkg_variants
            .first()
            .ok_or(VariantAuthoringError::PkgMissingSchemaVariant)?;

        let mut schema = SchemaVariant::get_by_id(ctx, current_sv_id)
            .await?
            .schema(ctx)
            .await?;

        let mut thing_map = clone_and_import_funcs(ctx, pkg.funcs()?).await?;

        if let Some(new_schema_variant) = import_schema_variant(
            ctx,
            None,
            &mut schema,
            schema_spec.clone(),
            variant_pkg_spec,
            None,
            &mut thing_map,
        )
        .await?
        {
            schema
                .set_default_schema_variant(ctx, new_schema_variant.id)
                .await?;
            return Ok(new_schema_variant.id);
        } else {
            return Err(VariantAuthoringError::NoAssetCreated);
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(
        name = "variant.authoring.save_variant_content",
        level = "info",
        skip_all
    )]
    pub async fn save_variant_content(
        ctx: &DalContext,
        current_sv_id: SchemaVariantId,
        name: String,
        menu_name: Option<String>,
        link: Option<String>,
        code: String,
        description: Option<String>,
    ) -> VariantAuthoringResult<()> {
        let current_sv = SchemaVariant::get_by_id(ctx, current_sv_id).await?;

        if let Some(asset_func_id) = current_sv.asset_func_id {
            let code_base64 = general_purpose::STANDARD_NO_PAD.encode(code);

            let current_func = Func::get_by_id_or_error(ctx, asset_func_id).await?;
            current_func
                .modify(ctx, |func| {
                    func.name = name;
                    func.backend_kind = FuncBackendKind::JsSchemaVariantDefinition;
                    func.backend_response_type = FuncBackendResponseType::SchemaVariantDefinition;
                    func.display_name = menu_name
                        .clone()
                        .map(|display_name| display_name.to_owned());
                    func.code_base64 = Some(code_base64);
                    func.description = description.clone();
                    func.handler = Some("main".to_string());
                    func.hidden = false;
                    func.link = link.clone();
                    Ok(())
                })
                .await?;
            Ok(())
        } else {
            return Err(VariantAuthoringError::SchemaVariantAssetNotFound(
                current_sv_id,
            ));
        }
    }
}

async fn build_variant_spec_based_on_existing_variant(
    ctx: &DalContext,
    definition: SchemaVariantJson,
    asset_func_spec: &FuncSpec,
    metadata: &SchemaVariantMetadataJson,
    existing_variant_id: SchemaVariantId,
) -> VariantAuthoringResult<(SchemaVariantSpec, Vec<MergeSkip>, Vec<FuncSpec>)> {
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let existing_variant = SchemaVariant::get_by_id(ctx, existing_variant_id).await?;

    let variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_spec.unique_id,
    )?;

    let (existing_variant_spec, variant_funcs) =
        PkgExporter::export_variant_standalone(ctx, &existing_variant).await?;

    let (merged_variant, skips) = variant_spec.merge_prototypes_from(&existing_variant_spec);

    Ok((merged_variant, skips, variant_funcs))
}

#[allow(clippy::result_large_err)]
fn build_asset_func_spec(asset_func: &Func) -> VariantAuthoringResult<FuncSpec> {
    let mut schema_variant_func_spec = FuncSpec::builder();
    schema_variant_func_spec.name(asset_func.name.clone());
    schema_variant_func_spec.unique_id(asset_func.id.to_string());
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
        func_spec_data_builder.handler(handler.to_string());
    }
    if let Some(description) = asset_func.description.clone() {
        func_spec_data_builder.description(description.to_string());
    }
    if let Some(display_name) = asset_func.display_name.clone() {
        func_spec_data_builder.display_name(display_name.to_string());
    }
    Ok(schema_variant_func_spec
        .data(func_spec_data_builder.build()?)
        .build()?)
}
async fn execute_asset_func(
    ctx: &DalContext,
    asset_func: &Func,
) -> VariantAuthoringResult<SchemaVariantJson> {
    let (_, return_value) =
        FuncBinding::create_and_execute(ctx, serde_json::Value::Null, asset_func.id, vec![])
            .await?;
    if let Some(error) = return_value
        .value()
        .ok_or(VariantAuthoringError::FuncExecution(asset_func.id))?
        .as_object()
        .ok_or(VariantAuthoringError::FuncExecution(asset_func.id))?
        .get("error")
        .and_then(|e| e.as_str())
    {
        return Err(VariantAuthoringError::FuncExecutionFailure(
            error.to_owned(),
        ));
    }
    let func_resp = return_value
        .value()
        .ok_or(VariantAuthoringError::FuncExecution(asset_func.id))?
        .as_object()
        .ok_or(VariantAuthoringError::FuncExecution(asset_func.id))?
        .get("definition")
        .ok_or(VariantAuthoringError::FuncExecution(asset_func.id))?;
    Ok(serde_json::from_value::<SchemaVariantJson>(
        func_resp.to_owned(),
    )?)
}
#[allow(clippy::result_large_err)]
fn build_pkg_spec_for_variant(
    definition: SchemaVariantJson,
    asset_func_spec: &FuncSpec,
    metadata: &SchemaVariantMetadataJson,
    user_email: &str,
) -> VariantAuthoringResult<PkgSpec> {
    // we need to change this to use the PkgImport
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;
    let variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_spec.unique_id,
    )?;
    let schema_spec = metadata.to_spec(variant_spec)?;
    Ok(PkgSpec::builder()
        .name(metadata.clone().name)
        .created_by(user_email)
        .func(identity_func_spec)
        .func(asset_func_spec.clone())
        .schema(schema_spec)
        .version("0.0.1")
        .build()?)
}

fn generate_scaffold_func_name(name: String) -> String {
    let version = Utc::now().format("%Y%m%d%H%M").to_string();
    let generated_name = format!("{}Scaffold_{}", name.to_case(Case::Camel), version);
    generated_name
}
