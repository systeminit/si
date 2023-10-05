use super::{
    maybe_delete_schema_variant_connected_to_variant_def, migrate_actions_to_new_schema_variant,
    migrate_attribute_functions_to_new_schema_variant,
    migrate_leaf_functions_to_new_schema_variant,
    migrate_validation_functions_to_new_schema_variant, AttributePrototypeContextKind,
    SaveVariantDefRequest, SchemaVariantDefinitionError, SchemaVariantDefinitionResult,
    ValidationPrototypeDefinition,
};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use chrono::Utc;
use convert_case::{Case, Casing};
use dal::{
    func::intrinsics::IntrinsicFunc,
    pkg::import_pkg_from_pkg,
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
    },
    AttributePrototypeId, ChangeSet, Func, FuncBinding, FuncId, HistoryActor, SchemaVariant,
    SchemaVariantError, SchemaVariantId, StandardModel, User, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, PkgSpec, SiPkg,
};
use std::collections::HashMap;

pub type ExecVariantDefRequest = SaveVariantDefRequest;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    pub id: AttributePrototypeId,
    pub func_id: FuncId,
    pub key: Option<String>,
    pub context: AttributePrototypeContextKind,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefResponse {
    pub success: bool,
    pub schema_variant_id: SchemaVariantId,
    pub func_exec_response: serde_json::Value,
    pub detached_validation_prototypes: Vec<ValidationPrototypeDefinition>,
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

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(), None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

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

    let mut variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    let (
        maybe_previous_variant_id,
        leaf_funcs_to_migrate,
        attribute_prototypes,
        validation_prototypes,
    ) = maybe_delete_schema_variant_connected_to_variant_def(
        &ctx,
        &mut variant_def,
        request.auto_reattach_functions,
    )
    .await?;

    let asset_func = Func::get_by_id(&ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;

    let metadata: SchemaVariantDefinitionMetadataJson = variant_def.clone().into();

    let (_, return_value) =
        FuncBinding::create_and_execute(&ctx, serde_json::Value::Null, *asset_func.id()).await?;

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
    let definition: SchemaVariantDefinitionJson = serde_json::from_value(func_resp.to_owned())?;

    // we need to change this to use the PkgImport
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

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
    let asset_func_built = schema_variant_func_spec
        .data(func_spec_data_builder.build()?)
        .build()?;

    let variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_built.unique_id,
    )?;
    let schema_spec = metadata.to_spec(variant_spec)?;
    let pkg_spec = PkgSpec::builder()
        .name(metadata.clone().name)
        .created_by(&user_email)
        .func(identity_func_spec)
        .func(asset_func_built.clone())
        .schema(schema_spec)
        .version("0.0.1")
        .build()?;

    let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;
    let (_, schema_variant_ids) = import_pkg_from_pkg(
        &ctx,
        &pkg,
        Some(dal::pkg::ImportOptions {
            schemas: None,
            skip_import_funcs: Some(HashMap::from_iter([(
                asset_func_built.unique_id.to_owned(),
                asset_func.clone(),
            )])),
            no_record: true,
            is_builtin: false,
        }),
    )
    .await?;

    let schema_variant_id = schema_variant_ids
        .get(0)
        .copied()
        .ok_or(SchemaVariantDefinitionError::NoAssetCreated)?;

    let (detached_attribute_prototypes, detached_validation_prototypes) =
        if let Some(previous_schema_variant_id) = maybe_previous_variant_id {
            migrate_leaf_functions_to_new_schema_variant(
                &ctx,
                leaf_funcs_to_migrate,
                schema_variant_id,
            )
            .await?;
            migrate_actions_to_new_schema_variant(
                &ctx,
                previous_schema_variant_id,
                schema_variant_id,
            )
            .await?;

            let schema_variant = SchemaVariant::get_by_id(&ctx, &schema_variant_id)
                .await?
                .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;

            let attribute_prototypes = migrate_attribute_functions_to_new_schema_variant(
                &ctx,
                attribute_prototypes,
                &schema_variant,
            )
            .await?;
            let mut detached_attribute_prototypes = Vec::with_capacity(attribute_prototypes.len());
            for attribute_prototype in attribute_prototypes {
                detached_attribute_prototypes.push(AttributePrototypeView {
                    id: attribute_prototype.id,
                    func_id: attribute_prototype.func_id,
                    key: attribute_prototype.key,
                    context: attribute_prototype.context,
                });
            }

            let detached_validation_prototypes =
                migrate_validation_functions_to_new_schema_variant(
                    &ctx,
                    validation_prototypes,
                    schema_variant_id,
                )
                .await?;

            (
                detached_attribute_prototypes,
                detached_validation_prototypes,
            )
        } else {
            (Vec::new(), Vec::new())
        };

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

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }

    Ok(
        response.body(serde_json::to_string(&ExecVariantDefResponse {
            success: true,
            func_exec_response: func_resp.to_owned(),
            schema_variant_id,
            detached_validation_prototypes,
            detached_attribute_prototypes,
        })?)?,
    )
}

fn generate_scaffold_func_name(name: String) -> String {
    let version = Utc::now().format("%Y%m%d%H%M").to_string();
    let generated_name = format!("{}Scaffold_{}", name.to_case(Case::Camel), version);
    generated_name
}
