use std::{collections::HashMap, str::FromStr};

use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use chrono::Utc;
use convert_case::{Case, Casing};
use hyper::Uri;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use ulid::Ulid;

use dal::ws_event::FinishSchemaVariantDefinitionPayload;
use dal::{
    func::intrinsics::IntrinsicFunc,
    pkg::import_pkg_from_pkg,
    pkg::{ImportAttributeSkip, ImportSkips, PkgExporter},
    prop::PropPath,
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
    },
    ChangeSet, DalContext, Func, FuncBinding, FuncId, HistoryActor, SchemaVariant,
    SchemaVariantError, SchemaVariantId, StandardModel, User, WsEvent,
};
use si_pkg::{
    FlatPropSpec, FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, FuncSpecData,
    PkgSpec, PropSpec, PropSpecKind, SiPkg, SocketSpecKind,
};
use telemetry::prelude::*;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

use super::{SaveVariantDefRequest, SchemaVariantDefinitionError, SchemaVariantDefinitionResult};

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
        let (schema_variant_id, skips) = match exec_variant_def_inner(
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
                return handle_error(&ctx, task_id, err.to_string()).await;
            }
        };

        let event = match WsEvent::schema_variant_definition_finish(
            &ctx,
            FinishSchemaVariantDefinitionPayload {
                task_id,
                schema_variant_id,
                skips,
            },
        )
        .await
        {
            Ok(event) => event,
            Err(err) => {
                return error!("Unable to make ws event of finish: {err}");
            }
        };

        if let Err(err) = event.publish_on_commit(&ctx).await {
            return error!("Unable to publish ws event of finish: {err}");
        };

        if let Err(err) = ctx.commit().await {
            handle_error(&ctx, task_id, err.to_string()).await;
        }

        async fn handle_error(ctx: &DalContext, id: Ulid, err: String) {
            error!("Unable to export workspace: {err}");
            match WsEvent::async_error(ctx, id, err).await {
                Ok(event) => match event.publish_on_commit(ctx).await {
                    Ok(()) => {}
                    Err(err) => error!("Unable to publish ws event of error: {err}"),
                },
                Err(err) => {
                    error!("Unable to make ws event of error: {err}");
                }
            }
            if let Err(err) = ctx.commit().await {
                error!("Unable to commit errors in exec_variant_def: {err}");
            }
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

#[instrument(name = "async_task.exec_variant_def", level = "info", skip_all)]
pub async fn exec_variant_def_inner(
    ctx: &DalContext,
    request: &ExecVariantDefRequest,
    original_uri: &Uri,
    PosthogClient(posthog_client): PosthogClient,
    request_span: Span,
) -> SchemaVariantDefinitionResult<(SchemaVariantId, Vec<ImportSkips>)> {
    Span::current().follows_from(request_span.id());

    let scaffold_func_name = generate_scaffold_func_name(request.name.clone());

    // Ensure we save all details before "exec"
    super::save_variant_def(ctx, request, Some(scaffold_func_name)).await?;

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(ctx, *user_pk).await?,
        _ => None,
    };
    let user_email = user
        .map(|user| user.email().to_owned())
        .unwrap_or("unauthenticated user email".into());

    let mut variant_def = SchemaVariantDefinition::get_by_id(ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;
    let maybe_previous_variant_id = variant_def.schema_variant_id().copied();

    let asset_func = Func::get_by_id(ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;

    let metadata: SchemaVariantDefinitionMetadataJson = variant_def.clone().into();

    // Execute asset function
    let (definition, _) = {
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
    let mut initial_skips = ImportSkips::default();

    if let Some(previous_schema_variant_id) = maybe_previous_variant_id {
        let previous_variant = SchemaVariant::get_by_id(&ctx, &previous_schema_variant_id)
            .await?
            .ok_or(SchemaVariantError::NotFound(previous_schema_variant_id))?;
        let mut exporter = PkgExporter::new_workspace_exporter(
            "temporary",
            "SystemInit".to_owned(),
            "1.0",
            "Temporary pkg created to update schemas",
        );
        variant_spec.leaf_functions = exporter
            .export_leaf_funcs(
                &ctx,
                Some(ctx.visibility().change_set_pk),
                *previous_variant.id(),
            )
            .await?;
        variant_spec.action_funcs = exporter
            .export_action_funcs(
                &ctx,
                Some(ctx.visibility().change_set_pk),
                *previous_variant.id(),
            )
            .await?;
        variant_spec.auth_funcs = exporter
            .export_auth_funcs(
                &ctx,
                Some(ctx.visibility().change_set_pk),
                *previous_variant.id(),
            )
            .await?;
        variant_spec.si_prop_funcs = exporter
            .export_si_prop_funcs(
                &ctx,
                Some(ctx.visibility().change_set_pk),
                &previous_variant,
            )
            .await?;
        variant_spec.root_prop_funcs = exporter
            .export_root_prop_funcs(
                &ctx,
                Some(ctx.visibility().change_set_pk),
                &previous_variant,
            )
            .await?;

        'outer: for old_socket in exporter
            .export_sockets(
                &ctx,
                Some(ctx.visibility().change_set_pk),
                *previous_variant.id(),
            )
            .await?
        {
            for socket in &mut variant_spec.sockets {
                if socket.inputs.is_empty() && socket.name == old_socket.name {
                    socket.inputs = old_socket.inputs.clone();
                    if let Some(data) = &mut socket.data {
                        data.func_unique_id = old_socket
                            .data
                            .as_ref()
                            .and_then(|s| s.func_unique_id.clone());
                    } else {
                        socket.data = old_socket.data;
                    }
                    continue 'outer;
                }
            }

            if let Some(data) = &old_socket.data {
                if let Some(func_unique_id) = &data.func_unique_id {
                    if let Ok(func_id) = FuncId::from_str(func_unique_id) {
                        if let Some(func) = &Func::get_by_id(&ctx, &func_id).await? {
                            if data.kind == SocketSpecKind::Input {
                                initial_skips.attribute_skips.push((
                                    old_socket.name.clone(),
                                    vec![ImportAttributeSkip::MissingInputSocket {
                                        name: old_socket.name,
                                        variant: func.try_into().ok(),
                                        func_id: *func.id(),
                                    }],
                                ));
                            } else {
                                initial_skips.attribute_skips.push((
                                    old_socket.name.clone(),
                                    vec![ImportAttributeSkip::MissingOutputSocket {
                                        name: old_socket.name,
                                        variant: func.try_into().ok(),
                                        func_id: *func.id(),
                                    }],
                                ));
                            }
                        }
                    }
                }
            }
        }

        let mut func_ids: Vec<String> = Vec::new();

        exporter
            .export_func(&ctx, Some(ctx.visibility().change_set_pk), &asset_func)
            .await?;
        let previous_variant_spec = exporter
            .export_variant(
                &ctx,
                Some(ctx.visibility().change_set_pk),
                &previous_variant,
            )
            .await?;
        {
            let mut previous_map = previous_variant_spec.flatten_domain()?;
            let mut map = variant_spec.flatten_domain()?;
            for (path, spec) in map.iter_mut() {
                if let Some(previous_spec) = previous_map.remove(path) {
                    if let (Some(data), Some(previous_data)) = (&mut spec.data, &previous_spec.data)
                    {
                        if data.inputs.as_ref().map_or(true, |i| i.is_empty())
                            && spec.kind == previous_spec.kind
                        {
                            data.inputs = previous_data.inputs.clone();
                            data.func_unique_id = previous_data.func_unique_id.clone();
                        }
                    } else {
                        spec.data = previous_spec.data.clone();
                    }
                } else {
                    info!("Prop missing: {path}");
                }

                if let Some(func_id) = spec.data.as_ref().and_then(|s| s.func_unique_id.clone()) {
                    func_ids.push(func_id);
                }
            }

            for (path, spec) in previous_map {
                initial_skips.attribute_skips.push((
                    spec.name.clone(),
                    vec![ImportAttributeSkip::MissingProp {
                        path: PropPath::from(path),
                        variant: None,
                        func_id: None,
                    }],
                ));
            }

            let mut parent_prop_spec = PropSpec::Object {
                name: "domain".to_owned(),
                data: None,
                unique_id: None,
                entries: Vec::new(),
            };
            recursive_prop_spec_builder("root/domain".to_owned(), &mut parent_prop_spec, &mut map)?;
            variant_spec.domain = parent_prop_spec;

            let mut previous_map = previous_variant_spec.flatten_secrets()?;
            let mut map = variant_spec.flatten_secrets()?;
            for (path, spec) in map.iter_mut() {
                if let Some(previous_spec) = previous_map.remove(path) {
                    if let (Some(data), Some(previous_data)) = (&mut spec.data, &previous_spec.data)
                    {
                        if data.inputs.as_ref().map_or(true, |i| i.is_empty())
                            && spec.kind == previous_spec.kind
                        {
                            data.inputs = previous_data.inputs.clone();
                            data.func_unique_id = previous_data.func_unique_id.clone();
                        }
                    } else {
                        spec.data = previous_spec.data.clone();
                    }
                } else {
                    info!("Prop missing: {path}");
                }

                if let Some(func_id) = spec.data.as_ref().and_then(|s| s.func_unique_id.clone()) {
                    func_ids.push(func_id);
                }
            }

            for (path, spec) in previous_map {
                initial_skips.attribute_skips.push((
                    spec.name.clone(),
                    vec![ImportAttributeSkip::MissingProp {
                        path: PropPath::from(path),
                        variant: None,
                        func_id: None,
                    }],
                ));
            }

            let mut parent_prop_spec = PropSpec::Object {
                name: "secrets".to_owned(),
                data: None,
                unique_id: None,
                entries: Vec::new(),
            };
            recursive_prop_spec_builder(
                "root/secrets".to_owned(),
                &mut parent_prop_spec,
                &mut map,
            )?;
            variant_spec.secrets = parent_prop_spec;

            let mut previous_map = previous_variant_spec.flatten_secret_definition()?;
            let mut map = variant_spec.flatten_secret_definition()?;
            for (path, spec) in map.iter_mut() {
                if let Some(previous_spec) = previous_map.remove(path) {
                    if let (Some(data), Some(previous_data)) = (&mut spec.data, &previous_spec.data)
                    {
                        if data.inputs.as_ref().map_or(true, |i| i.is_empty())
                            && spec.kind == previous_spec.kind
                        {
                            data.inputs = previous_data.inputs.clone();
                            data.func_unique_id = previous_data.func_unique_id.clone();
                        }
                    } else {
                        spec.data = previous_spec.data.clone();
                    }
                } else {
                    info!("Prop missing: {path}");
                }

                if let Some(func_id) = spec.data.as_ref().and_then(|s| s.func_unique_id.clone()) {
                    func_ids.push(func_id);
                }
            }

            for (path, spec) in previous_map {
                initial_skips.attribute_skips.push((
                    spec.name.clone(),
                    vec![ImportAttributeSkip::MissingProp {
                        path: PropPath::from(path),
                        variant: None,
                        func_id: None,
                    }],
                ));
            }

            if let Some(definition) = &mut variant_spec.secret_definition {
                let mut parent_prop_spec = PropSpec::Object {
                    name: "secret_definition".to_owned(),
                    data: None,
                    unique_id: None,
                    entries: Vec::new(),
                };
                recursive_prop_spec_builder(
                    "root/secret_definition".to_owned(),
                    &mut parent_prop_spec,
                    &mut map,
                )?;
                *definition = parent_prop_spec;
            }

            // TODO: make this iterative
            fn recursive_prop_spec_builder(
                path: String,
                spec: &mut PropSpec,
                map: &mut HashMap<String, FlatPropSpec>,
            ) -> SchemaVariantDefinitionResult<()> {
                if let Some(prop) = map.remove(&path) {
                    *spec.data_mut() = prop.data;
                    *spec.unique_id_mut() = prop.unique_id;

                    // TODO: do we need to handle maps and arrays type props?
                    if let PropSpec::Object { entries, .. } = spec {
                        // TODO: improve complexity of finding children
                        for (inner_path, inner_spec) in map.clone().into_iter() {
                            let name = inner_path.replace(&format!("{path}/"), "");
                            if !name.contains('/') {
                                let mut child_spec = match inner_spec.kind {
                                    PropSpecKind::String => PropSpec::String {
                                        name,
                                        data: None,
                                        unique_id: None,
                                    },
                                    PropSpecKind::Number => PropSpec::Number {
                                        name,
                                        data: None,
                                        unique_id: None,
                                    },
                                    PropSpecKind::Boolean => PropSpec::Boolean {
                                        name,
                                        data: None,
                                        unique_id: None,
                                    },
                                    PropSpecKind::Array => PropSpec::Array {
                                        name,
                                        type_prop: inner_spec.type_prop.clone().ok_or(SchemaVariantDefinitionError::TypePropMissingForMapOrArray)?,
                                        data: None,
                                        unique_id: None,
                                    },
                                    PropSpecKind::Map => PropSpec::Map {
                                        name,
                                        type_prop: inner_spec.type_prop.clone().ok_or(SchemaVariantDefinitionError::TypePropMissingForMapOrArray)?,
                                        map_key_funcs: inner_spec.map_key_funcs.clone(),
                                        data: None,
                                        unique_id: None,
                                    },
                                    PropSpecKind::Object => PropSpec::Object {
                                        name,
                                        data: None,
                                        unique_id: None,
                                        entries: Vec::new(),
                                    },
                                };
                                recursive_prop_spec_builder(
                                    inner_path.clone(),
                                    &mut child_spec,
                                    map,
                                )?;
                                entries.push(child_spec);
                            }
                        }
                    }
                }
                Ok(())
            }
        }

        func_ids.extend(
            variant_spec
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
                )
                .chain(
                    variant_spec
                        .sockets
                        .iter()
                        .flat_map(|s| s.data.as_ref())
                        .flat_map(|s| s.func_unique_id.clone()),
                ),
        );
        for unique_id in func_ids {
            if let Ok(func_id) = FuncId::from_str(&unique_id) {
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
    } else {
        variant_def.delete_by_id(&ctx).await?;
    }

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
            .func(asset_func_built.clone())
            .schema(schema_spec)
            .version("0.0.1");

        builder.build()?
    };

    let pkg = SiPkg::load_from_spec(pkg_spec.clone())?;

    let (_, schema_variant_ids, mut skips) = import_pkg_from_pkg(
        ctx,
        &pkg,
        None,
        request.override_builtin_schema_feature_flag,
    )
    .await?;
    skips.push(initial_skips);

    let schema_variant_id = schema_variant_ids
        .get(0)
        .copied()
        .ok_or(SchemaVariantDefinitionError::NoAssetCreated)?;

    track(
        &posthog_client,
        ctx,
        original_uri,
        "exec_variant_def",
        serde_json::json!({
                    "variant_def_category": metadata.clone().category,
                    "variant_def_name": metadata.clone().name,
                    "variant_def_version": pkg_spec.clone().version,
                    "variant_def_schema_count":  pkg_spec.clone().schemas.len(),
                    "variant_def_function_count":  pkg_spec.clone().funcs.len(),
        }),
    );

    Ok((schema_variant_id, skips))
}

fn generate_scaffold_func_name(name: String) -> String {
    let version = Utc::now().format("%Y%m%d%H%M").to_string();
    let generated_name = format!("{}Scaffold_{}", name.to_case(Case::Camel), version);
    generated_name
}
