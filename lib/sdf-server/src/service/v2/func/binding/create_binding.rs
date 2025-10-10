use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSet,
    ChangeSetId,
    Component,
    Func,
    FuncId,
    SchemaVariant,
    WorkspacePk,
    WsEvent,
    func::binding::{
        AttributeArgumentBinding,
        EventualParent,
        action::ActionBinding,
        attribute::AttributeBinding,
        authentication::AuthBinding,
        leaf::LeafBinding,
        management::ManagementBinding,
    },
    schema::variant::leaves::{
        LeafInputLocation,
        LeafKind,
    },
};
use si_events::audit_log::AuditLogKind;
use si_frontend_types as frontend_types;

use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::{
            AccessBuilder,
            func::{
                FuncAPIError,
                FuncAPIResult,
            },
        },
    },
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
) -> FuncAPIResult<ForceChangeSetResponse<Vec<si_frontend_types::FuncBinding>>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id(&ctx, func_id).await?;

    if func.is_transformation {
        return Err(FuncAPIError::WrongFunctionKindForBinding);
    }

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
                                    AttributeBinding::assemble_attribute_input_location(
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

                            let (_new_binding, old_func) =
                                AttributeBinding::upsert_attribute_binding(
                                    &ctx,
                                    func_id.into_raw_id().into(),
                                    eventual_parent.clone(),
                                    attribute_output_location,
                                    arguments,
                                )
                                .await?;
                            // Fire WS Event if the func has changed
                            if let Some(old_func_id) = old_func {
                                if old_func_id != func_id {
                                    let old_func_summary = Func::get_by_id(&ctx, old_func_id)
                                        .await?
                                        .into_frontend_type(&ctx)
                                        .await?;

                                    WsEvent::func_updated(&ctx, old_func_summary, None)
                                        .await?
                                        .publish_on_commit(&ctx)
                                        .await?;
                                }
                            }

                            let (subject_name, component_id, schema_variant_id): (
                                String,
                                Option<si_events::ComponentId>,
                                Option<si_events::SchemaVariantId>,
                            ) = match eventual_parent {
                                Some(eventual_parent) => match eventual_parent {
                                    EventualParent::SchemaVariant(schema_variant_id) => (
                                        SchemaVariant::get_by_id(&ctx, schema_variant_id)
                                            .await?
                                            .display_name()
                                            .to_string(),
                                        None,
                                        Some(schema_variant_id),
                                    ),
                                    EventualParent::Component(component_id) => (
                                        Component::get_by_id(&ctx, component_id)
                                            .await?
                                            .name(&ctx)
                                            .await?,
                                        Some(component_id),
                                        None,
                                    ),
                                    EventualParent::Schemas(_) => (String::new(), None, None),
                                },
                                None => (String::new(), None, None),
                            };
                            let destination_name = attribute_output_location
                                .get_name_of_destination(&ctx)
                                .await?;
                            ctx.write_audit_log(
                                AuditLogKind::AttachAttributeFunc {
                                    func_id: func.id,
                                    func_display_name: func.display_name.clone(),
                                    schema_variant_id,
                                    component_id,
                                    subject_name,
                                    prop_id,
                                    output_socket_id,
                                    destination_name,
                                },
                                func.name.clone(),
                            )
                            .await?;

                            if let Some(variant_id) = schema_variant_id {
                                let schema = SchemaVariant::schema_id(&ctx, variant_id).await?;
                                let schema_variant =
                                    SchemaVariant::get_by_id(&ctx, variant_id).await?;
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
                            AuthBinding::create_auth_binding(&ctx, func_id, schema_variant_id)
                                .await?;
                            let schema = SchemaVariant::schema_id(&ctx, schema_variant_id).await?;
                            let schema_variant =
                                SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;
                            let func = Func::get_by_id(&ctx, func_id).await?;
                            ctx.write_audit_log(
                                AuditLogKind::AttachAuthFunc {
                                    func_id: func.id,
                                    func_display_name: func.display_name.clone(),
                                    schema_variant_id: Some(schema_variant_id),
                                },
                                func.name.clone(),
                            )
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
                                func_id,
                                action_kind.into(),
                                schema_variant_id,
                            )
                            .await?;
                            let schema = SchemaVariant::schema_id(&ctx, schema_variant_id).await?;
                            let schema_variant =
                                SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;
                            let func = Func::get_by_id(&ctx, func_id).await?;
                            ctx.write_audit_log(
                                AuditLogKind::AttachActionFunc {
                                    func_id: func.id,
                                    func_display_name: func.display_name.clone(),
                                    schema_variant_id: Some(schema_variant_id),
                                    component_id: None,
                                    action_kind: Some(action_kind),
                                },
                                func.name.clone(),
                            )
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
                                func_id,
                                EventualParent::SchemaVariant(schema_variant_id),
                                LeafKind::CodeGeneration,
                                &inputs,
                            )
                            .await?;
                            let schema = SchemaVariant::schema_id(&ctx, schema_variant_id).await?;
                            let schema_variant =
                                SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;
                            let func = Func::get_by_id(&ctx, func_id).await?;
                            ctx.write_audit_log(
                                AuditLogKind::AttachCodeGenFunc {
                                    func_id: func.id,
                                    func_display_name: func.display_name.clone(),
                                    schema_variant_id: Some(schema_variant_id),
                                    component_id: None,
                                    subject_name: schema_variant.display_name().to_owned(),
                                },
                                func.name.clone(),
                            )
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
                                func_id,
                                EventualParent::SchemaVariant(schema_variant_id),
                                LeafKind::Qualification,
                                &inputs,
                            )
                            .await?;
                            let schema = SchemaVariant::schema_id(&ctx, schema_variant_id).await?;
                            let schema_variant =
                                SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;
                            let func = Func::get_by_id(&ctx, func_id).await?;
                            ctx.write_audit_log(
                                AuditLogKind::AttachQualificationFunc {
                                    func_id: func.id,
                                    func_display_name: func.display_name.clone(),
                                    schema_variant_id: Some(schema_variant_id),
                                    component_id: None,
                                    subject_name: schema_variant.display_name().to_owned(),
                                },
                                func.name.clone(),
                            )
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
        dal::func::FuncKind::Management => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Management {
                    schema_variant_id,
                    func_id,
                    ..
                } = binding
                {
                    match (func_id, schema_variant_id) {
                        (Some(func_id), Some(schema_variant_id)) => {
                            ManagementBinding::create_management_binding(
                                &ctx,
                                func_id,
                                None,
                                Some(schema_variant_id),
                            )
                            .await?;
                            let schema = SchemaVariant::schema_id(&ctx, schema_variant_id).await?;
                            let schema_variant =
                                SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;
                            let func = Func::get_by_id(&ctx, func_id).await?;
                            ctx.write_audit_log(
                                AuditLogKind::AttachManagementFunc {
                                    func_id: func.id,
                                    func_display_name: func.display_name.clone(),
                                    schema_variant_id: Some(schema_variant_id),
                                    schema_id: None,
                                    component_id: None,
                                    subject_name: schema_variant.display_name().to_owned(),
                                },
                                func.name.clone(),
                            )
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
        dal::func::FuncKind::Unknown | dal::func::FuncKind::SchemaVariantDefinition => {
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

    let func_summary = Func::get_by_id(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;

    let bindings = func_summary.clone().bindings;

    WsEvent::func_updated(&ctx, func_summary, None)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, bindings))
}
