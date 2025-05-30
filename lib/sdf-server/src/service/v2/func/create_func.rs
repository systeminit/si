use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    ChangeSet,
    Component,
    SchemaVariant,
    WsEvent,
    func::{
        FuncKind,
        authoring::FuncAuthoringClient,
        binding::{
            AttributeArgumentBinding,
            AttributeFuncArgumentSource,
            AttributeFuncDestination,
            EventualParent,
        },
    },
    schema::variant::leaves::{
        LeafInputLocation,
        LeafKind,
    },
};
use sdf_extract::change_set::ChangeSetDalContext;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use si_frontend_types::{
    FuncBinding,
    FuncCode,
    FuncSummary,
};

use super::{
    FuncAPIError,
    FuncAPIResult,
    get_code_response,
};
use crate::{
    extract::PosthogClient,
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncRequest {
    name: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    binding: Option<FuncBinding>,
    kind: FuncKind,
    #[serde(default)]
    is_transformation: bool,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncResponse {
    summary: FuncSummary,
    code: FuncCode,
}

pub async fn create_func(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CreateFuncRequest>,
) -> FuncAPIResult<ForceChangeSetResponse<CreateFuncResponse>> {
    if let Some(name) = request.name.as_deref() {
        if dal::func::is_intrinsic(name) {
            return Err(FuncAPIError::FuncNameReserved(name.into()));
        }
    }

    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let func = match request.kind {
        FuncKind::Action => {
            if let Some(FuncBinding::Action {
                schema_variant_id: Some(schema_variant_id),
                kind: Some(kind),
                ..
            }) = request.binding
            {
                let func = FuncAuthoringClient::create_new_action_func(
                    ctx,
                    request.name,
                    kind.into(),
                    schema_variant_id,
                )
                .await?;
                ctx.write_audit_log(
                    AuditLogKind::CreateFunc {
                        func_display_name: func.display_name.clone(),
                        func_kind: func.kind.into(),
                    },
                    func.name.clone(),
                )
                .await?;
                ctx.write_audit_log(
                    AuditLogKind::AttachActionFunc {
                        func_id: func.id,
                        func_display_name: func.display_name.clone(),
                        schema_variant_id: Some(schema_variant_id),
                        component_id: None,
                        action_kind: Some(kind),
                    },
                    func.name.clone(),
                )
                .await?;
                func
            } else {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        }
        FuncKind::Attribute => {
            if request.is_transformation {
                FuncAuthoringClient::create_new_transformation_func(ctx, request.name).await?
            } else {
                match request.binding.clone() {
                    Some(FuncBinding::Attribute {
                        prop_id,
                        output_socket_id,
                        argument_bindings,
                        component_id,
                        ..
                    }) => {
                        let output_location = if let Some(prop_id) = prop_id {
                            AttributeFuncDestination::Prop(prop_id)
                        } else if let Some(output_socket_id) = output_socket_id {
                            AttributeFuncDestination::OutputSocket(output_socket_id)
                        } else {
                            return Err(FuncAPIError::MissingOutputLocationForAttributeFunc);
                        };
                        let eventual_parent = component_id.map(EventualParent::Component);
                        let mut arg_bindings = vec![];
                        for arg_binding in argument_bindings {
                            let input_location = if let Some(prop_id) = arg_binding.prop_id {
                                AttributeFuncArgumentSource::Prop(prop_id)
                            } else if let Some(input_socket_id) = arg_binding.input_socket_id {
                                AttributeFuncArgumentSource::InputSocket(input_socket_id)
                            } else {
                                return Err(FuncAPIError::MissingInputLocationForAttributeFunc);
                            };
                            arg_bindings.push(AttributeArgumentBinding {
                                func_argument_id: arg_binding.func_argument_id,
                                attribute_prototype_argument_id: arg_binding
                                    .attribute_prototype_argument_id,
                                attribute_func_input_location: input_location,
                            });
                        }

                        let func = FuncAuthoringClient::create_new_attribute_func(
                            ctx,
                            request.name,
                            eventual_parent,
                            output_location,
                            arg_bindings,
                        )
                        .await?;
                        ctx.write_audit_log(
                            AuditLogKind::CreateFunc {
                                func_display_name: func.display_name.clone(),
                                func_kind: func.kind.into(),
                            },
                            func.name.clone(),
                        )
                        .await?;
                        let (subject_name, component_id, schema_variant_id): (
                            String,
                            Option<si_events::ComponentId>,
                            Option<si_events::SchemaVariantId>,
                        ) = match eventual_parent {
                            Some(eventual_parent) => {
                                if let EventualParent::Component(component_id) = eventual_parent {
                                    (
                                        Component::get_by_id(ctx, component_id)
                                            .await?
                                            .name(ctx)
                                            .await?,
                                        Some(component_id),
                                        None,
                                    )
                                } else {
                                    return Err(FuncAPIError::MissingSchemaVariantAndFunc);
                                }
                            }
                            None => {
                                let schema_variant_id =
                                    output_location.find_schema_variant(ctx).await?;
                                (
                                    SchemaVariant::get_by_id(ctx, schema_variant_id)
                                        .await?
                                        .display_name()
                                        .to_string(),
                                    None,
                                    Some(schema_variant_id),
                                )
                            }
                        };
                        let destination_name = output_location.get_name_of_destination(ctx).await?;
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
                        func
                    }
                    _ => {
                        return Err(FuncAPIError::CreatingAttributeFuncWithoutBinding);
                    }
                }
            }
        }
        FuncKind::Authentication => match request.binding.clone() {
            Some(FuncBinding::Authentication {
                schema_variant_id,
                func_id: _,
            }) => {
                let func =
                    FuncAuthoringClient::create_new_auth_func(ctx, request.name, schema_variant_id)
                        .await?;
                ctx.write_audit_log(
                    AuditLogKind::CreateFunc {
                        func_display_name: func.display_name.clone(),
                        func_kind: func.kind.into(),
                    },
                    func.name.clone(),
                )
                .await?;
                ctx.write_audit_log(
                    AuditLogKind::AttachAuthFunc {
                        func_id: func.id,
                        func_display_name: func.display_name.clone(),
                        schema_variant_id: Some(schema_variant_id),
                    },
                    func.name.clone(),
                )
                .await?;
                func
            }
            _ => {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        },
        FuncKind::CodeGeneration => match request.binding.clone() {
            Some(FuncBinding::CodeGeneration {
                schema_variant_id: Some(schema_variant_id),
                inputs,
                ..
            }) => {
                let inputs = if inputs.is_empty() {
                    vec![LeafInputLocation::Domain]
                } else {
                    inputs.into_iter().map(|input| input.into()).collect()
                };
                let func = FuncAuthoringClient::create_new_leaf_func(
                    ctx,
                    request.name,
                    LeafKind::CodeGeneration,
                    EventualParent::SchemaVariant(schema_variant_id),
                    &inputs,
                )
                .await?;
                ctx.write_audit_log(
                    AuditLogKind::CreateFunc {
                        func_display_name: func.display_name.clone(),
                        func_kind: func.kind.into(),
                    },
                    func.name.clone(),
                )
                .await?;
                let schema_variant_name = SchemaVariant::get_by_id(ctx, schema_variant_id)
                    .await?
                    .display_name()
                    .to_string();
                ctx.write_audit_log(
                    AuditLogKind::AttachCodeGenFunc {
                        func_id: func.id,
                        func_display_name: func.display_name.clone(),
                        schema_variant_id: Some(schema_variant_id),
                        component_id: None,
                        subject_name: schema_variant_name,
                    },
                    func.name.clone(),
                )
                .await?;
                func
            }
            _ => {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        },
        FuncKind::Qualification => match request.binding.clone() {
            Some(FuncBinding::Qualification {
                schema_variant_id: Some(schema_variant_id),
                inputs,
                ..
            }) => {
                let inputs = if inputs.is_empty() {
                    vec![LeafInputLocation::Domain, LeafInputLocation::Code]
                } else {
                    inputs.into_iter().map(|input| input.into()).collect()
                };

                let func = FuncAuthoringClient::create_new_leaf_func(
                    ctx,
                    request.name,
                    LeafKind::Qualification,
                    EventualParent::SchemaVariant(schema_variant_id),
                    &inputs,
                )
                .await?;
                ctx.write_audit_log(
                    AuditLogKind::CreateFunc {
                        func_display_name: func.display_name.clone(),
                        func_kind: func.kind.into(),
                    },
                    func.name.clone(),
                )
                .await?;
                let schema_variant_name = SchemaVariant::get_by_id(ctx, schema_variant_id)
                    .await?
                    .display_name()
                    .to_string();
                ctx.write_audit_log(
                    AuditLogKind::AttachQualificationFunc {
                        func_id: func.id,
                        func_display_name: func.display_name.clone(),
                        schema_variant_id: Some(schema_variant_id),
                        component_id: None,
                        subject_name: schema_variant_name,
                    },
                    func.name.clone(),
                )
                .await?;
                func
            }
            _ => {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        },
        FuncKind::Management => match request.binding.clone() {
            Some(FuncBinding::Management {
                schema_variant_id: Some(schema_variant_id),
                ..
            }) => {
                let func = FuncAuthoringClient::create_new_management_func(
                    ctx,
                    request.name,
                    schema_variant_id,
                )
                .await?;
                ctx.write_audit_log(
                    AuditLogKind::CreateFunc {
                        func_display_name: func.display_name.clone(),
                        func_kind: func.kind.into(),
                    },
                    func.name.clone(),
                )
                .await?;
                let schema_variant_name = SchemaVariant::get_by_id(ctx, schema_variant_id)
                    .await?
                    .display_name()
                    .to_string();
                ctx.write_audit_log(
                    AuditLogKind::AttachManagementFunc {
                        func_id: func.id,
                        func_display_name: func.display_name.clone(),
                        schema_variant_id: Some(schema_variant_id),
                        component_id: None,
                        subject_name: schema_variant_name,
                    },
                    func.name.clone(),
                )
                .await?;
                func
            }
            _ => {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        },
        FuncKind::Unknown | FuncKind::SchemaVariantDefinition | FuncKind::Intrinsic => {
            return Err(FuncAPIError::WrongFunctionKindForBinding);
        }
    };

    let code = get_code_response(ctx, func.id).await?;
    let summary = func.into_frontend_type(ctx).await?;
    WsEvent::func_created(ctx, summary.clone())
        .await?
        .publish_on_commit(ctx)
        .await?;
    track(
        &posthog_client,
        ctx,
        &original_uri,
        &host_name,
        "created_func",
        serde_json::json!({
            "how": "/func/created_func",
            "func_id": summary.func_id,
            "func_name": summary.name.to_owned(),
            "func_kind": summary.kind,
        }),
    );

    match request.binding {
        Some(FuncBinding::Action {
            schema_variant_id: Some(schema_variant_id),
            ..
        })
        | Some(FuncBinding::Attribute {
            schema_variant_id: Some(schema_variant_id),
            ..
        })
        | Some(FuncBinding::Authentication {
            schema_variant_id, ..
        })
        | Some(FuncBinding::CodeGeneration {
            schema_variant_id: Some(schema_variant_id),
            ..
        })
        | Some(FuncBinding::Qualification {
            schema_variant_id: Some(schema_variant_id),
            ..
        })
        | Some(FuncBinding::Management {
            schema_variant_id: Some(schema_variant_id),
            ..
        }) => {
            let schema_id = SchemaVariant::schema_id(ctx, schema_variant_id).await?;
            let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
            WsEvent::schema_variant_updated(ctx, schema_id, schema_variant)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
        _ => {}
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        CreateFuncResponse { summary, code },
    ))
}
