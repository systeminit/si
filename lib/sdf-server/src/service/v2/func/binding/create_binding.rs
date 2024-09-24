use axum::{
    extract::{Host, OriginalUri, Path},
    response::IntoResponse,
    Json,
};
use dal::{
    func::binding::{
        action::ActionBinding, attribute::AttributeBinding, authentication::AuthBinding,
        leaf::LeafBinding, AttributeArgumentBinding, EventualParent,
    },
    schema::variant::leaves::{LeafInputLocation, LeafKind},
    ChangeSet, ChangeSetId, Func, FuncId, SchemaVariant, WorkspacePk, WsEvent,
};
use si_frontend_types as frontend_types;

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::v2::func::{FuncAPIError, FuncAPIResult},
    track,
};

pub async fn create_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncBindings>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id_or_error(&ctx, func_id).await?;

    // add cycle check so we don't end up with a cycle as a result of creating this binding
    let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
    match func.kind {
        dal::func::FuncKind::Attribute | dal::func::FuncKind::Intrinsic => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Attribute {
                    func_id,
                    component_id,
                    schema_variant_id,
                    prop_id,
                    output_socket_id,
                    argument_bindings,
                    ..
                } = binding
                {
                    match func_id {
                        Some(func_id) => {
                            let eventual_parent = AttributeBinding::assemble_eventual_parent(
                                &ctx,
                                component_id,
                                schema_variant_id,
                            )
                            .await?;
                            let attribute_output_location =
                                AttributeBinding::assemble_attribute_output_location(
                                    prop_id,
                                    output_socket_id,
                                )?;
                            let mut arguments: Vec<AttributeArgumentBinding> = vec![];
                            for arg_binding in argument_bindings {
                                let input_location =
                                    AttributeArgumentBinding::assemble_attribute_input_location(
                                        arg_binding.prop_id,
                                        arg_binding.input_socket_id,
                                        arg_binding.static_value,
                                    )?;
                                arguments.push(AttributeArgumentBinding {
                                    func_argument_id: arg_binding
                                        .func_argument_id
                                        .into_raw_id()
                                        .into(),
                                    attribute_func_input_location: input_location,
                                    attribute_prototype_argument_id: None, // when creating a new prototype,
                                                                           // we don't have the attribute prototype arguments yet
                                });
                            }

                            AttributeBinding::upsert_attribute_binding(
                                &ctx,
                                func_id.into_raw_id().into(),
                                eventual_parent,
                                attribute_output_location,
                                arguments,
                            )
                            .await?;
                            if let Some(EventualParent::SchemaVariant(schema_variant_id)) =
                                eventual_parent
                            {
                                let schema = SchemaVariant::schema_id_for_schema_variant_id(
                                    &ctx,
                                    schema_variant_id,
                                )
                                .await?;
                                let schema_variant =
                                    SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id)
                                        .await?;

                                WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                                    .await?
                                    .publish_on_commit(&ctx)
                                    .await?;
                            }
                        }
                        None => {
                            return Err(FuncAPIError::MissingFuncId);
                        }
                    }
                }
            }
        }
        dal::func::FuncKind::Authentication => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Authentication {
                    schema_variant_id,
                    func_id,
                } = binding
                {
                    match func_id {
                        Some(func_id) => {
                            AuthBinding::create_auth_binding(
                                &ctx,
                                func_id.into(),
                                schema_variant_id.into(),
                            )
                            .await?;
                            let schema = SchemaVariant::schema_id_for_schema_variant_id(
                                &ctx,
                                schema_variant_id.into(),
                            )
                            .await?;
                            let schema_variant =
                                SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id.into())
                                    .await?;

                            WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                                .await?
                                .publish_on_commit(&ctx)
                                .await?;
                        }
                        None => return Err(FuncAPIError::MissingFuncId),
                    }
                }
            }
        }
        dal::func::FuncKind::Action => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Action {
                    schema_variant_id,
                    func_id,
                    kind,
                    ..
                } = binding
                {
                    match (kind, func_id, schema_variant_id) {
                        (Some(action_kind), Some(func_id), Some(schema_variant_id)) => {
                            ActionBinding::create_action_binding(
                                &ctx,
                                func_id.into(),
                                action_kind.into(),
                                schema_variant_id.into(),
                            )
                            .await?;
                            let schema = SchemaVariant::schema_id_for_schema_variant_id(
                                &ctx,
                                schema_variant_id.into(),
                            )
                            .await?;
                            let schema_variant =
                                SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id.into())
                                    .await?;

                            WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                                .await?
                                .publish_on_commit(&ctx)
                                .await?;
                        }
                        _ => {
                            return Err(FuncAPIError::MissingActionKindForActionFunc);
                        }
                    }
                } else {
                    return Err(FuncAPIError::MissingActionKindForActionFunc);
                }
            }
        }
        dal::func::FuncKind::CodeGeneration | dal::func::FuncKind::Qualification => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::CodeGeneration {
                    inputs,
                    schema_variant_id,
                    func_id,
                    ..
                } = binding
                {
                    match (schema_variant_id, func_id) {
                        (Some(schema_variant_id), Some(func_id)) => {
                            let inputs: Vec<LeafInputLocation> =
                                inputs.into_iter().map(|input| input.into()).collect();
                            LeafBinding::create_leaf_func_binding(
                                &ctx,
                                func_id.into(),
                                EventualParent::SchemaVariant(schema_variant_id.into()),
                                LeafKind::CodeGeneration,
                                &inputs,
                            )
                            .await?;
                            let schema = SchemaVariant::schema_id_for_schema_variant_id(
                                &ctx,
                                schema_variant_id.into(),
                            )
                            .await?;
                            let schema_variant =
                                SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id.into())
                                    .await?;

                            WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                                .await?
                                .publish_on_commit(&ctx)
                                .await?;
                        }
                        _ => {
                            return Err(FuncAPIError::MissingSchemaVariantAndFunc);
                        }
                    }
                } else if let frontend_types::FuncBinding::Qualification {
                    inputs,
                    schema_variant_id,
                    func_id,
                    ..
                } = binding
                {
                    match (schema_variant_id, func_id) {
                        (Some(schema_variant_id), Some(func_id)) => {
                            let inputs: Vec<LeafInputLocation> =
                                inputs.into_iter().map(|input| input.into()).collect();
                            LeafBinding::create_leaf_func_binding(
                                &ctx,
                                func_id.into(),
                                EventualParent::SchemaVariant(schema_variant_id.into()),
                                LeafKind::Qualification,
                                &inputs,
                            )
                            .await?;
                            let schema = SchemaVariant::schema_id_for_schema_variant_id(
                                &ctx,
                                schema_variant_id.into(),
                            )
                            .await?;
                            let schema_variant =
                                SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id.into())
                                    .await?;

                            WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                                .await?
                                .publish_on_commit(&ctx)
                                .await?;
                        }
                        _ => {
                            return Err(FuncAPIError::MissingSchemaVariantAndFunc);
                        }
                    }
                } else {
                    return Err(FuncAPIError::WrongFunctionKindForBinding);
                }
            }
        }

        _ => {
            return Err(FuncAPIError::WrongFunctionKindForBinding);
        }
    };
    drop(cycle_check_guard);
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "created_binding",
        serde_json::json!({
            "how": "/func/created_binding",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );

    let func_summary = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;

    let bindings = func_summary.clone().bindings;

    WsEvent::func_updated(&ctx, func_summary, None)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(&bindings)?)?)
}
