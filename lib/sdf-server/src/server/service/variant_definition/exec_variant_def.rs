use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use chrono::Utc;
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;

use dal::{
    func::intrinsics::IntrinsicFunc,
    pkg::import_pkg_from_pkg,
    pkg::PkgExporter,
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
    },
    AttributePrototypeId, ChangeSet, Func, FuncBinding, FuncId, HistoryActor, SchemaVariant,
    SchemaVariantError, SchemaVariantId, StandardModel, User,
};
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, PkgSpec, SiPkg,
};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

use super::{
    super::func::FuncVariant, AttributePrototypeContextKind, SaveVariantDefRequest,
    SchemaVariantDefinitionError, SchemaVariantDefinitionResult,
};

pub type ExecVariantDefRequest = SaveVariantDefRequest;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    pub id: AttributePrototypeId,
    pub func_id: FuncId,
    pub func_name: String,
    pub variant: Option<FuncVariant>,
    pub key: Option<String>,
    pub context: AttributePrototypeContextKind,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefResponse {
    pub success: bool,
    pub schema_variant_id: SchemaVariantId,
    pub func_exec_response: serde_json::Value,
    pub detached_attribute_prototypes: Vec<AttributePrototypeView>,
}

pub async fn exec_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ExecVariantDefRequest>,
) -> SchemaVariantDefinitionResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    let scaffold_func_name = generate_scaffold_func_name(request.name.clone());

    // Ensure we save all details before "exec"
    super::save_variant_def(&ctx, &request, Some(scaffold_func_name)).await?;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk).await?,
        _ => None,
    };
    let user_email = user
        .map(|user| user.email().to_owned())
        .unwrap_or("unauthenticated user email".into());

    let variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;
    let maybe_previous_variant_id = variant_def.schema_variant_id().copied();

    let asset_func = Func::get_by_id(&ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;

    let metadata: SchemaVariantDefinitionMetadataJson = variant_def.clone().into();

    // Execute asset function
    let (definition, func_exec_response) = {
        let (_, return_value) = FuncBinding::create_and_execute(
            &ctx,
            serde_json::Value::Null,
            *asset_func.id(),
            vec![],
        )
        .await?;

        if let Some(error) = return_value
            .value()
            .ok_or(SchemaVariantDefinitionError::FuncExecution(
                *asset_func.id(),
            ))?
            .as_object()
            .ok_or(SchemaVariantDefinitionError::FuncExecution(
                *asset_func.id(),
            ))?
            .get("error")
            .and_then(|e| e.as_str())
        {
            return Err(SchemaVariantDefinitionError::FuncExecutionFailure(
                error.to_owned(),
            ));
        }

        let func_resp = return_value
            .value()
            .ok_or(SchemaVariantDefinitionError::FuncExecution(
                *asset_func.id(),
            ))?
            .as_object()
            .ok_or(SchemaVariantDefinitionError::FuncExecution(
                *asset_func.id(),
            ))?
            .get("definition")
            .ok_or(SchemaVariantDefinitionError::FuncExecution(
                *asset_func.id(),
            ))?;

        (
            serde_json::from_value::<SchemaVariantDefinitionJson>(func_resp.to_owned())?,
            func_resp.to_owned(),
        )
    };

    let asset_func_built = {
        let mut schema_variant_func_spec = FuncSpec::builder();
        schema_variant_func_spec.name(asset_func.name());
        schema_variant_func_spec.unique_id(asset_func.id().to_string());
        let mut func_spec_data_builder = FuncSpecData::builder();
        func_spec_data_builder
            .name(asset_func.name())
            .backend_kind(FuncSpecBackendKind::JsSchemaVariantDefinition)
            .response_type(FuncSpecBackendResponseType::SchemaVariantDefinition)
            .hidden(asset_func.hidden());
        if let Some(code) = asset_func.code_plaintext()? {
            func_spec_data_builder.code_plaintext(code);
        }
        if let Some(handler) = asset_func.handler() {
            func_spec_data_builder.handler(handler.to_string());
        }
        if let Some(description) = asset_func.description() {
            func_spec_data_builder.description(description.to_string());
        }
        if let Some(display_name) = asset_func.display_name() {
            func_spec_data_builder.display_name(display_name.to_string());
        }
        schema_variant_func_spec
            .data(func_spec_data_builder.build()?)
            .build()?
    };

    // we need to change this to use the PkgImport
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let mut variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_built.unique_id,
    )?;

    let mut func_specs = Vec::new();

    let detached_attribute_prototypes = match maybe_previous_variant_id {
        Some(previous_schema_variant_id) => {
            let variant = SchemaVariant::get_by_id(&ctx, &previous_schema_variant_id)
                .await?
                .ok_or(SchemaVariantError::NotFound(previous_schema_variant_id))?;
            let exporter = PkgExporter::new_workspace_exporter(
                "temporary",
                "SystemInit".to_owned(),
                "1.0",
                "Temporary pkg created to update schemas",
            );
            variant_spec.leaf_functions = exporter
                .export_leaf_funcs(&ctx, Some(ctx.visibility().change_set_pk), *variant.id())
                .await?;
            variant_spec.action_funcs = exporter
                .export_action_funcs(&ctx, Some(ctx.visibility().change_set_pk), *variant.id())
                .await?;
            variant_spec.auth_funcs = exporter
                .export_auth_funcs(&ctx, Some(ctx.visibility().change_set_pk), *variant.id())
                .await?;
            variant_spec.si_prop_funcs = exporter
                .export_si_prop_funcs(&ctx, Some(ctx.visibility().change_set_pk), &variant)
                .await?;
            variant_spec.root_prop_funcs = exporter
                .export_root_prop_funcs(&ctx, Some(ctx.visibility().change_set_pk), &variant)
                .await?;

            let unique_ids = variant_spec
                .leaf_functions
                .iter()
                .map(|f| f.func_unique_id.clone())
                .chain(
                    variant_spec
                        .action_funcs
                        .iter()
                        .map(|f| f.func_unique_id.clone()),
                )
                .chain(
                    variant_spec
                        .auth_funcs
                        .iter()
                        .map(|f| f.func_unique_id.clone()),
                )
                .chain(
                    variant_spec
                        .si_prop_funcs
                        .iter()
                        .map(|f| f.func_unique_id.clone()),
                )
                .chain(
                    variant_spec
                        .root_prop_funcs
                        .iter()
                        .map(|f| f.func_unique_id.clone()),
                );
            for unique_id in unique_ids {
                if let Some(func_id) = FuncId::from_str(&unique_id).ok() {
                    if let Some(func) = Func::get_by_id(&ctx, &func_id).await? {
                        let (spec, _) = exporter
                            .export_func(&ctx, Some(ctx.visibility().change_set_pk), &func)
                            .await?;
                        if spec.data.is_some() {
                            func_specs.push(spec);
                        }
                    }
                }
            }

            let detached_attribute_prototypes = Vec::new();
            /*
            let attribute_prototypes = migrate_attribute_functions_to_new_schema_variant(
                &ctx,
                attribute_prototypes,
                &schema_variant,
            )
            .await?;
            let mut detached_attribute_prototypes = Vec::with_capacity(attribute_prototypes.len());

            for attribute_prototype in attribute_prototypes {
                let func = Func::get_by_id(&ctx, &attribute_prototype.func_id)
                    .await?
                    .ok_or_else(|| {
                        SchemaVariantDefinitionError::FuncNotFound(attribute_prototype.func_id)
                    })?;
                detached_attribute_prototypes.push(AttributePrototypeView {
                    id: attribute_prototype.id,
                    func_id: attribute_prototype.func_id,
                    func_name: func.name().to_owned(),
                    variant: (&func).try_into().ok(),
                    key: attribute_prototype.key,
                    context: attribute_prototype.context,
                });
            }
            */

            detached_attribute_prototypes
        }
        None => {
            vec![]
        }
    };

    let schema_spec = metadata.to_spec(variant_spec)?;

    let pkg_spec = {
        let mut builder = PkgSpec::builder();

        for spec in func_specs {
            builder.func(spec);
        }

        for intrinsic in IntrinsicFunc::iter() {
            let spec = intrinsic.to_spec()?;
            builder.func(spec);
        }

        builder
            .name(metadata.clone().name)
            .created_by(&user_email)
            .func(identity_func_spec)
            .func(asset_func_built.clone())
            .schema(schema_spec)
            .version("0.0.1");

        builder.build()?
    };

    let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;

    let (_, schema_variant_ids, _) = import_pkg_from_pkg(
        &ctx,
        &pkg,
        None,
        request.override_builtin_schema_feature_flag,
    )
    .await?;
    let schema_variant_id = schema_variant_ids
        .get(0)
        .copied()
        .ok_or(SchemaVariantDefinitionError::NoAssetCreated)?;

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

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }

    Ok(
        response.body(serde_json::to_string(&ExecVariantDefResponse {
            success: true,
            func_exec_response,
            schema_variant_id,
            detached_attribute_prototypes,
        })?)?,
    )
}

fn generate_scaffold_func_name(name: String) -> String {
    let version = Utc::now().format("%Y%m%d%H%M").to_string();
    let generated_name = format!("{}Scaffold_{}", name.to_case(Case::Camel), version);
    generated_name
}
