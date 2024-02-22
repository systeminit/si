use std::collections::HashMap;

use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use chrono::Utc;
use convert_case::{Case, Casing};
use dal::component::ComponentKind;
use dal::pkg::import::{clone_and_import_funcs, import_schema_variant};
use dal::pkg::PkgExporter;
use hyper::Uri;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::service::async_route::handle_error;
use dal::ws_event::{AttributePrototypeView, FinishSchemaVariantDefinitionPayload};
use dal::{
    func::intrinsics::IntrinsicFunc,
    pkg::{attach_resource_payload_to_value, import_pkg_from_pkg},
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
    },
    ChangeSet, DalContext, Func, FuncBinding, HistoryActor, SchemaVariant, SchemaVariantError,
    SchemaVariantId, StandardModel, User, WsEvent,
};
use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData, MergeSkip, PkgSpec,
    SchemaVariantSpec, SiPkg,
};
use telemetry::prelude::*;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant_definition::migrate_authentication_funcs_to_new_schema_variant;

use super::{
    maybe_delete_schema_variant_connected_to_variant_def, migrate_actions_to_new_schema_variant,
    migrate_attribute_functions_to_new_schema_variant,
    migrate_leaf_functions_to_new_schema_variant, SaveVariantDefRequest,
    SchemaVariantDefinitionError, SchemaVariantDefinitionResult,
};

pub type ExecVariantDefRequest = SaveVariantDefRequest;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefResponse {
    pub task_id: Ulid,
    pub success: bool,
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

    let task_id = Ulid::new();

    let request_span = Span::current();

    tokio::task::spawn(async move {
        let (schema_variant_id, detached_attribute_prototypes) = match exec_variant_def_inner(
            &ctx,
            &request,
            &original_uri,
            PosthogClient(posthog_client),
            request_span,
        )
        .await
        {
            Ok(values) => values,
            Err(err) => {
                return handle_error(&ctx, original_uri, task_id, err).await;
            }
        };

        let event = match WsEvent::schema_variant_definition_finish(
            &ctx,
            FinishSchemaVariantDefinitionPayload {
                task_id,
                schema_variant_id,
                detached_attribute_prototypes,
            },
        )
        .await
        {
            Ok(event) => event,
            Err(err) => {
                return handle_error(&ctx, original_uri, task_id, err).await;
            }
        };

        if let Err(err) = event.publish_on_commit(&ctx).await {
            return handle_error(&ctx, original_uri, task_id, err).await;
        };

        if let Err(err) = ctx.commit().await {
            handle_error(&ctx, original_uri, task_id, err).await;
        }
    });

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }

    Ok(
        response.body(serde_json::to_string(&ExecVariantDefResponse {
            task_id,
            success: true,
        })?)?,
    )
}

fn generate_scaffold_func_name(name: String) -> String {
    let version = Utc::now().format("%Y%m%d%H%M").to_string();
    let generated_name = format!("{}Scaffold_{}", name.to_case(Case::Camel), version);
    generated_name
}

async fn execute_asset_func(
    ctx: &DalContext,
    asset_func: &Func,
) -> SchemaVariantDefinitionResult<SchemaVariantDefinitionJson> {
    let (_, return_value) =
        FuncBinding::create_and_execute(ctx, serde_json::Value::Null, *asset_func.id(), vec![])
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

    Ok(serde_json::from_value::<SchemaVariantDefinitionJson>(
        func_resp.to_owned(),
    )?)
}

#[allow(clippy::result_large_err)]
fn build_asset_func_spec(asset_func: &Func) -> SchemaVariantDefinitionResult<FuncSpec> {
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

    Ok(schema_variant_func_spec
        .data(func_spec_data_builder.build()?)
        .build()?)
}

fn inc_variant_name(name: &str) -> String {
    let (prefix, suffix) = name.split_at(1);

    let new_suffix = match suffix.parse::<i64>() {
        Ok(number) => (number + 1).to_string(),
        Err(_) => format!("{}_new", suffix),
    };

    format!("{}{}", prefix, new_suffix)
}

async fn build_variant_spec_based_on_existing_variant(
    ctx: &DalContext,
    definition: SchemaVariantDefinitionJson,
    asset_func_spec: &FuncSpec,
    metadata: &SchemaVariantDefinitionMetadataJson,
    existing_variant_id: SchemaVariantId,
) -> SchemaVariantDefinitionResult<(SchemaVariantSpec, Vec<MergeSkip>, Vec<FuncSpec>)> {
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let existing_variant = SchemaVariant::get_by_id(ctx, &existing_variant_id)
        .await?
        .ok_or(SchemaVariantError::NotFound(existing_variant_id))?;

    let new_name = inc_variant_name(existing_variant.name());

    let variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_spec.unique_id,
        &new_name,
    )?;

    let (existing_variant_spec, variant_funcs) =
        PkgExporter::export_variant_standalone(ctx, &existing_variant).await?;

    let (merged_variant, skips) = variant_spec.merge_prototypes_from(&existing_variant_spec);

    Ok((merged_variant, skips, variant_funcs))
}

#[allow(clippy::result_large_err)]
fn build_pkg_spec_for_variant(
    definition: SchemaVariantDefinitionJson,
    asset_func_spec: &FuncSpec,
    metadata: &SchemaVariantDefinitionMetadataJson,
    user_email: &str,
) -> SchemaVariantDefinitionResult<PkgSpec> {
    // we need to change this to use the PkgImport
    let identity_func_spec = IntrinsicFunc::Identity.to_spec()?;

    let variant_spec = definition.to_spec(
        metadata.clone(),
        &identity_func_spec.unique_id,
        &asset_func_spec.unique_id,
        "v0",
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

#[instrument(name = "async_task.exec_variant_def", level = "info", skip_all)]
pub async fn exec_variant_def_inner(
    ctx: &DalContext,
    request: &ExecVariantDefRequest,
    original_uri: &Uri,
    PosthogClient(posthog_client): PosthogClient,
    request_span: Span,
) -> SchemaVariantDefinitionResult<(SchemaVariantId, Vec<AttributePrototypeView>)> {
    let current_span = Span::current();
    let span = current_span.follows_from(request_span.id());

    let user_email = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(ctx, *user_pk)
            .await?
            .map(|user| user.email().to_owned()),
        _ => None,
    }
    .unwrap_or("unauthenticated user email".into());

    let current_variant_def = SchemaVariantDefinition::get_by_id(ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    // Exec forks here if the multi variant editing flag is on and the definition is for an existing asset
    if let Some(&current_schema_variant_id) = current_variant_def.schema_variant_id() {
        if request.multi_variant_editing_flag {
            return exec_variant_def_multi_variant_editing(
                ctx,
                request.to_owned(),
                original_uri,
                posthog_client,
                span,
                user_email,
                current_variant_def,
                current_schema_variant_id,
            )
            .await;
        }
    }

    let scaffold_func_name = generate_scaffold_func_name(request.name.clone());

    // Ensure we save all details before "exec"
    super::save_variant_def(ctx, request, Some(scaffold_func_name)).await?;
    let mut variant_def = SchemaVariantDefinition::get_by_id(ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    let asset_func = Func::get_by_id(ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;

    let metadata: SchemaVariantDefinitionMetadataJson = variant_def.clone().into();

    let (maybe_previous_variant_id, leaf_funcs_to_migrate, attribute_prototypes) =
        maybe_delete_schema_variant_connected_to_variant_def(ctx, &mut variant_def).await?;

    // Execute asset function to get schema variant definition
    let asset_func_built = build_asset_func_spec(&asset_func)?;
    let definition = execute_asset_func(ctx, &asset_func).await?;
    let pkg_spec =
        build_pkg_spec_for_variant(definition, &asset_func_built, &metadata, &user_email)?;

    let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;

    let (_, schema_variant_ids, _) = import_pkg_from_pkg(
        ctx,
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
        request.override_builtin_schema_feature_flag,
    )
    .await?;

    let schema_variant_id = schema_variant_ids
        .get(0)
        .copied()
        .ok_or(SchemaVariantDefinitionError::NoAssetCreated)?;

    let detached_attribute_prototypes = match maybe_previous_variant_id {
        Some(previous_schema_variant_id) => {
            migrate_leaf_functions_to_new_schema_variant(
                ctx,
                leaf_funcs_to_migrate,
                schema_variant_id,
            )
            .await?;
            migrate_actions_to_new_schema_variant(
                ctx,
                previous_schema_variant_id,
                schema_variant_id,
            )
            .await?;
            migrate_authentication_funcs_to_new_schema_variant(
                ctx,
                previous_schema_variant_id,
                schema_variant_id,
            )
            .await?;

            let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
                .await?
                .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;

            let attribute_prototypes = migrate_attribute_functions_to_new_schema_variant(
                ctx,
                attribute_prototypes,
                &schema_variant,
            )
            .await?;
            let mut detached_attribute_prototypes = Vec::with_capacity(attribute_prototypes.len());

            for attribute_prototype in attribute_prototypes {
                let func = Func::get_by_id(ctx, &attribute_prototype.func_id)
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

            detached_attribute_prototypes
        }
        None => {
            attach_resource_payload_to_value(ctx, schema_variant_id).await?;
            vec![]
        }
    };

    track(
        &posthog_client,
        ctx,
        original_uri,
        "exec_variant_def",
        serde_json::json!({
                    "variant_def_category": metadata.clone().category,
                    "variant_def_name": metadata.clone().name,
                    "variant_def_version": pkg_spec.clone().version,
                    "variant_def_function_count":  pkg_spec.clone().funcs.len(),
                    "multi_variant_editing": false,
        }),
    );

    Ok((schema_variant_id, detached_attribute_prototypes))
}

#[instrument(
    name = "async_task.exec_variant_def_multi_variant_editing",
    level = "info",
    skip_all
)]
#[allow(clippy::too_many_arguments)]
pub async fn exec_variant_def_multi_variant_editing(
    ctx: &DalContext,
    request: ExecVariantDefRequest,
    original_uri: &Uri,
    posthog_client: crate::server::state::PosthogClient,
    span: &Span,
    user_email: String,
    current_variant_def: SchemaVariantDefinition,
    current_schema_variant_id: SchemaVariantId,
) -> SchemaVariantDefinitionResult<(SchemaVariantId, Vec<AttributePrototypeView>)> {
    Span::current().follows_from(span.id());

    // we need asset func drafts or some similar concept so that we don't write
    // to the last asset func before we get here
    let current_asset_func = Func::get_by_id(ctx, &current_variant_def.func_id())
        .await?
        .ok_or(SchemaVariantDefinitionError::FuncNotFound(
            current_variant_def.func_id(),
        ))?;

    let scaffold_func_name = generate_scaffold_func_name(request.name.clone());
    let new_asset_func = current_asset_func
        .duplicate(ctx, Some(scaffold_func_name))
        .await?;

    let mut new_definition = SchemaVariantDefinition::new(
        ctx,
        request.name,
        request.menu_name,
        request.category,
        request.link,
        request.color,
        ComponentKind::Standard,
        request.description,
        *new_asset_func.id(),
    )
    .await?;

    let metadata = new_definition.clone().into();

    // Execute asset function to get schema variant prop tree etc
    let definition = execute_asset_func(ctx, &new_asset_func).await?;
    let asset_func_built = build_asset_func_spec(&new_asset_func)?;
    let (new_variant_spec, skips, variant_funcs) = build_variant_spec_based_on_existing_variant(
        ctx,
        definition,
        &asset_func_built,
        &metadata,
        current_schema_variant_id,
    )
    .await?;

    dbg!(skips);

    let schema_spec = metadata.to_spec(new_variant_spec)?;
    let pkg_spec = PkgSpec::builder()
        .name(&metadata.name)
        .created_by(user_email)
        .funcs(variant_funcs.clone())
        .func(asset_func_built)
        .schema(schema_spec)
        .version("0")
        .build()?;

    // Copy some data here for sending to posthog track
    let variant_def_version = pkg_spec.version.to_owned();
    let variant_def_function_count = pkg_spec.funcs.len();
    let variant_def_category = metadata.category.to_owned();
    let variant_def_name = metadata.name.to_owned();

    let pkg = SiPkg::load_from_spec(pkg_spec)?;

    let mut schema = SchemaVariant::get_by_id(ctx, &current_schema_variant_id)
        .await?
        .ok_or(SchemaVariantError::NotFound(current_schema_variant_id))?
        .schema(ctx)
        .await?
        .ok_or(SchemaVariantError::MissingSchema(current_schema_variant_id))?;

    let pkg_variant = pkg
        .schemas()?
        .into_iter()
        .next()
        .ok_or(SchemaVariantDefinitionError::PkgMissingSchema)?
        .variants()?
        .into_iter()
        .next()
        .ok_or(SchemaVariantDefinitionError::PkgMissingSchemaVariant)?;

    let mut thing_map = clone_and_import_funcs(ctx, variant_funcs).await?;
    let new_schema_variant = import_schema_variant(
        ctx,
        ctx.visibility().change_set_pk,
        &mut schema,
        &pkg_variant,
        None,
        &mut thing_map,
        &pkg.metadata()?,
    )
    .await?
    .ok_or(SchemaVariantDefinitionError::NoAssetCreated)?;

    let schema_variant_id = *new_schema_variant.id();

    new_definition
        .set_schema_variant_id(ctx, Some(schema_variant_id))
        .await?;

    schema
        .set_default_schema_variant_id(ctx, Some(schema_variant_id))
        .await?;

    track(
        &posthog_client,
        ctx,
        original_uri,
        "exec_variant_def",
        serde_json::json!({
                    "variant_def_category": variant_def_category,
                    "variant_def_name": variant_def_name,
                    "variant_def_version": variant_def_version,
                    "variant_def_function_count": variant_def_function_count,
                    "multi_variant_editing": true,
        }),
    );

    Ok((schema_variant_id, vec![]))
}
