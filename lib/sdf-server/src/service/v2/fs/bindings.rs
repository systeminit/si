use std::collections::{
    BTreeMap,
    HashSet,
};

use axum::{
    Json,
    extract::{
        Path,
        Query,
    },
};
use dal::{
    ChangeSetId,
    DalContext,
    Func,
    FuncId,
    InputSocket,
    OutputSocket,
    Prop,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    WsEvent,
    attribute::prototype::argument::AttributePrototypeArgument,
    func::{
        FuncKind,
        argument::FuncArgument,
        binding::{
            AttributeArgumentBinding,
            AttributeFuncArgumentSource,
            AttributeFuncDestination,
            EventualParent,
            action::ActionBinding,
            attribute::AttributeBinding,
            authentication::AuthBinding,
            leaf::LeafBinding,
            management::ManagementBinding,
        },
        intrinsics::IntrinsicFunc,
    },
    prop::PropPath,
    schema::variant::leaves::LeafKind,
};
use si_events::{
    ActionKind,
    audit_log::AuditLogKind,
};
use si_frontend_types::{
    FuncBinding,
    fs::{
        self,
        AttributeFuncInput,
        AttributeInputFrom,
        AttributeOutputTo,
        Binding,
        IdentityBindings,
        PropIdentityBinding,
        SetFuncBindingsRequest,
        SocketIdentityBinding,
        VariantQuery,
    },
};
use si_id::WorkspaceId;

use super::{
    FsError,
    FsResult,
    check_change_set,
    check_change_set_and_not_head,
    dal_func_to_fs_func,
    func_types_size,
    get_or_unlock_schema,
    lookup_variant_for_schema,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogEventTracker,
    },
    service::v2::{
        AccessBuilder,
        func::binding::update_binding::{
            update_action_func_bindings,
            update_attribute_func_bindings,
            update_leaf_func_bindings,
        },
    },
};

pub async fn get_bindings_for_func_and_schema_variant(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: Option<SchemaVariantId>,
) -> FsResult<Vec<FuncBinding>> {
    let func = dal::Func::get_by_id(ctx, func_id).await?;

    Ok(match func.kind {
        dal::func::FuncKind::Action => {
            ActionBinding::assemble_action_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::Intrinsic => {
            AttributeBinding::assemble_intrinsic_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::Attribute => {
            AttributeBinding::assemble_attribute_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::Authentication => {
            AuthBinding::assemble_auth_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::CodeGeneration => {
            LeafBinding::assemble_leaf_func_bindings(ctx, func_id, LeafKind::CodeGeneration).await?
        }
        dal::func::FuncKind::Management => {
            ManagementBinding::assemble_management_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::Qualification => {
            LeafBinding::assemble_leaf_func_bindings(ctx, func_id, LeafKind::Qualification).await?
        }
        dal::func::FuncKind::Unknown | dal::func::FuncKind::SchemaVariantDefinition => vec![],
    }
    .into_iter()
    .filter(|binding| {
        if let Some(schema_variant_id) = schema_variant_id {
            binding.get_schema_variant() == Some(schema_variant_id)
        } else {
            true
        }
    })
    .map(Into::into)
    .collect())
}

pub async fn get_bindings(
    ctx: &DalContext,
    func_id: FuncId,
    schema_id: SchemaId,
) -> FsResult<(fs::Bindings, Vec<FuncBinding>)> {
    let func = Func::get_by_id(ctx, func_id).await?;

    // Locked functions could be on unlocked schemas
    let bindings = if func.is_locked {
        if let Some(locked) = lookup_variant_for_schema(ctx, schema_id, false).await? {
            let bindings =
                get_bindings_for_func_and_schema_variant(ctx, func_id, Some(locked.id())).await?;
            if bindings.is_empty() {
                if let Some(unlocked) = lookup_variant_for_schema(ctx, schema_id, true).await? {
                    get_bindings_for_func_and_schema_variant(ctx, func_id, Some(unlocked.id()))
                        .await?
                } else {
                    vec![]
                }
            } else {
                bindings
            }
        } else {
            vec![]
        }
    } else if let Some(unlocked) = lookup_variant_for_schema(ctx, schema_id, true).await? {
        get_bindings_for_func_and_schema_variant(ctx, func_id, Some(unlocked.id())).await?
    } else {
        vec![]
    };

    let mut fs_bindings = vec![];
    for binding in bindings.clone() {
        fs_bindings.push(func_binding_to_fs_binding(ctx, func_id, binding).await?);
    }

    Ok((
        fs::Bindings {
            bindings: fs_bindings,
        },
        bindings,
    ))
}

pub async fn func_binding_to_fs_binding(
    ctx: &DalContext,
    func_id: FuncId,
    binding: FuncBinding,
) -> FsResult<fs::Binding> {
    Ok(match binding {
        FuncBinding::Action { kind, .. } => fs::Binding::Action {
            kind: kind.unwrap_or(ActionKind::Manual),
        },
        FuncBinding::Attribute {
            prop_id,
            output_socket_id,
            argument_bindings,
            ..
        } => {
            attribute_binding_to_fs_attribute_binding(
                ctx,
                func_id,
                prop_id,
                output_socket_id,
                argument_bindings,
            )
            .await?
        }
        FuncBinding::Authentication { .. } => fs::Binding::Authentication,
        FuncBinding::CodeGeneration { inputs, .. } => fs::Binding::CodeGeneration { inputs },
        FuncBinding::Management { .. } => fs::Binding::Management,
        FuncBinding::Qualification { inputs, .. } => fs::Binding::Qualification { inputs },
    })
}

async fn attribute_binding_to_fs_attribute_binding(
    ctx: &DalContext,
    func_id: FuncId,
    prop_id: Option<dal::PropId>,
    output_socket_id: Option<dal::OutputSocketId>,
    argument_bindings: Vec<si_frontend_types::AttributeArgumentBinding>,
) -> FsResult<fs::Binding> {
    let output_to = if let Some(prop_id) = prop_id {
        let path = Prop::get_by_id(ctx, prop_id)
            .await?
            .path(ctx)
            .await?
            .with_replaced_sep("/");

        AttributeOutputTo::Prop(path)
    } else if let Some(output_socket_id) = output_socket_id {
        let name = OutputSocket::get_by_id(ctx, output_socket_id)
            .await?
            .name()
            .to_string();
        AttributeOutputTo::OutputSocket(name)
    } else {
        return Err(FsError::AttributeFuncNotBound);
    };
    let mut inputs = BTreeMap::new();
    for func_arg in FuncArgument::list_for_func(ctx, func_id).await? {
        let func_arg_kind = func_arg.kind;
        let func_arg_name = func_arg.name;
        let func_arg_element_kind = func_arg.element_kind;

        let input_from = match argument_bindings
            .iter()
            .find(|binding| binding.func_argument_id == func_arg.id)
        {
            Some(arg_binding) => Some(if let Some(input_prop_id) = arg_binding.prop_id {
                let path = Prop::get_by_id(ctx, input_prop_id)
                    .await?
                    .path(ctx)
                    .await?
                    .with_replaced_sep("/");

                AttributeInputFrom::Prop(path)
            } else if let Some(input_socket_id) = arg_binding.input_socket_id {
                let name = InputSocket::get_by_id(ctx, input_socket_id)
                    .await?
                    .name()
                    .to_string();
                AttributeInputFrom::InputSocket(name)
            } else {
                return Err(FsError::AttributeInputNotBound);
            }),
            None => None,
        };

        inputs.insert(
            func_arg_name,
            AttributeFuncInput {
                kind: func_arg_kind.into(),
                element_kind: func_arg_element_kind.map(Into::into),
                input: input_from,
            },
        );
    }
    Ok(fs::Binding::Attribute { output_to, inputs })
}

fn parse_code_gen_bindings_for_update(
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
    update_inputs: &[si_frontend_types::LeafInputLocation],
) -> FsResult<()> {
    let FuncBinding::CodeGeneration {
        schema_variant_id,
        component_id,
        func_id,
        attribute_prototype_id,
        ..
    } = func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    final_bindings.push(FuncBinding::CodeGeneration {
        schema_variant_id,
        component_id,
        func_id,
        attribute_prototype_id,
        inputs: update_inputs.to_owned(),
    });

    Ok(())
}

pub async fn output_to_into_func_destination(
    ctx: &DalContext,
    output_to: &AttributeOutputTo,
    schema_variant_id: SchemaVariantId,
) -> FsResult<AttributeFuncDestination> {
    Ok(match output_to {
        AttributeOutputTo::OutputSocket(name) => {
            let socket = OutputSocket::find_with_name(ctx, name, schema_variant_id)
                .await?
                .ok_or(FsError::OutputSocketNotFound(name.to_owned()))?;

            AttributeFuncDestination::OutputSocket(socket.id())
        }
        AttributeOutputTo::Prop(prop_path_string) => {
            let prop_path = PropPath::new(prop_path_string.split("/"));
            let prop_id = Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &prop_path)
                .await?
                .ok_or(FsError::PropNotFound(prop_path_string.to_owned()))?;

            AttributeFuncDestination::Prop(prop_id)
        }
    })
}

async fn parse_attr_bindings_for_update(
    ctx: &DalContext,
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
    output_to: &AttributeOutputTo,
    inputs: &BTreeMap<String, AttributeFuncInput>,
) -> FsResult<()> {
    let FuncBinding::Attribute {
        func_id,
        attribute_prototype_id,
        schema_variant_id,
        ..
    } = func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    // todo: make special errors for all 3
    let schema_variant_id = schema_variant_id.ok_or(FsError::FuncBindingKindMismatch)?;
    let func_id = func_id.ok_or(FsError::FuncBindingKindMismatch)?;
    let proto_id = attribute_prototype_id.ok_or(FsError::FuncBindingKindMismatch)?;

    let (prop_id, output_socket_id) =
        match output_to_into_func_destination(ctx, output_to, schema_variant_id).await? {
            AttributeFuncDestination::Prop(prop_id) => (Some(prop_id), None),
            AttributeFuncDestination::OutputSocket(output_socket_id) => {
                (None, Some(output_socket_id))
            }
            AttributeFuncDestination::InputSocket(_) => unreachable!(
                "this enum variant will never be produced by output_to_into_func_destination"
            ),
        };

    let argument_bindings = inputs_into_attribute_argument_bindings(
        inputs,
        ctx,
        func_id,
        Some(proto_id),
        schema_variant_id,
    )
    .await?;

    final_bindings.push(FuncBinding::Attribute {
        func_id: Some(func_id),
        attribute_prototype_id: Some(proto_id),
        component_id: None,
        schema_variant_id: Some(schema_variant_id),
        prop_id,
        output_socket_id,
        argument_bindings,
    });

    Ok(())
}

/// Note: this will also create new func args as necessary, and delete func args no longer in the "binding"
async fn inputs_into_attribute_argument_bindings(
    inputs: &BTreeMap<String, AttributeFuncInput>,
    ctx: &DalContext,
    func_id: FuncId,
    proto_id: Option<dal::AttributePrototypeId>,
    schema_variant_id: SchemaVariantId,
) -> FsResult<Vec<si_frontend_types::AttributeArgumentBinding>> {
    let mut argument_bindings = vec![];

    let mut current_arg_names = HashSet::new();
    for (arg_name, input) in inputs {
        let (func_arg, apa_id) = match FuncArgument::find_by_name_for_func(ctx, arg_name, func_id)
            .await?
        {
            Some(func_arg) => {
                let func_arg_id = func_arg.id;
                let proto_id = match proto_id {
                    Some(proto_id) => AttributePrototypeArgument::find_by_func_argument_id_and_attribute_prototype_id(
                        ctx,
                        func_arg_id,
                        proto_id
                    ).await?,
                    None => None,
                };
                (func_arg, proto_id)
            }
            None => {
                let func_arg = FuncArgument::new(
                    ctx,
                    arg_name,
                    input.kind.into(),
                    input.element_kind.map(Into::into),
                    func_id,
                )
                .await?;

                let apa_id = match proto_id {
                    Some(proto_id) => Some(
                        AttributePrototypeArgument::new_without_source(ctx, proto_id, func_arg.id)
                            .await?
                            .id(),
                    ),
                    None => None,
                };

                (func_arg, apa_id)
            }
        };

        current_arg_names.insert(func_arg.name.clone());

        let (prop_id, input_socket_id) = match input.input.as_ref() {
            Some(AttributeInputFrom::InputSocket(input_socket_name)) => {
                let socket = InputSocket::find_with_name(ctx, input_socket_name, schema_variant_id)
                    .await?
                    .ok_or(FsError::InputSocketNotFound(input_socket_name.to_owned()))?;

                (None, Some(socket.id()))
            }
            Some(AttributeInputFrom::Prop(prop_path_string)) => {
                let prop_path = PropPath::new(prop_path_string.split("/"));
                let prop_id = Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &prop_path)
                    .await?
                    .ok_or(FsError::PropNotFound(prop_path_string.to_owned()))?;

                (Some(prop_id), None)
            }
            None => (None, None),
        };

        if prop_id.is_some() || input_socket_id.is_some() {
            argument_bindings.push(si_frontend_types::AttributeArgumentBinding {
                func_argument_id: func_arg.id,
                attribute_prototype_argument_id: apa_id,
                prop_id,
                input_socket_id,
                static_value: None,
            });
        }
    }

    for arg in FuncArgument::list_for_func(ctx, func_id).await? {
        if !current_arg_names.contains(&arg.name) {
            FuncArgument::remove(ctx, arg.id).await?;
        }
    }

    Ok(argument_bindings)
}

fn parse_action_bindings(
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
    update_kind: ActionKind,
) -> FsResult<()> {
    let FuncBinding::Action {
        schema_variant_id,
        action_prototype_id,
        func_id,
        ..
    } = func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    final_bindings.push(FuncBinding::Action {
        schema_variant_id,
        action_prototype_id,
        func_id,
        kind: Some(update_kind),
    });

    Ok(())
}

async fn parse_binding_for_update(
    ctx: &DalContext,
    binding_update: &Binding,
    bindings_to_update: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
) -> Result<(), FsError> {
    match binding_update {
        Binding::Action { kind: update_kind } => {
            parse_action_bindings(bindings_to_update, func_binding, *update_kind)?;
        }
        Binding::Attribute { output_to, inputs } => {
            parse_attr_bindings_for_update(
                ctx,
                bindings_to_update,
                func_binding,
                output_to,
                inputs,
            )
            .await?;
        }
        Binding::Authentication => {}
        Binding::CodeGeneration {
            inputs: update_inputs,
        } => {
            parse_code_gen_bindings_for_update(bindings_to_update, func_binding, update_inputs)?;
        }
        Binding::Management => {}
        Binding::Qualification {
            inputs: update_inputs,
        } => {
            parse_qualification_bindings_for_update(
                bindings_to_update,
                func_binding,
                update_inputs,
            )?;
        }
    };
    Ok(())
}

async fn parse_binding_for_create(
    ctx: &DalContext,
    binding: Binding,
    schema_variant_id: SchemaVariantId,
    func_id: FuncId,
) -> FsResult<FuncBinding> {
    Ok(match binding {
        Binding::Action { kind } => FuncBinding::Action {
            schema_variant_id: Some(schema_variant_id),
            action_prototype_id: None,
            func_id: Some(func_id),
            kind: Some(kind),
        },
        Binding::Attribute { output_to, inputs } => {
            let (prop_id, output_socket_id) =
                match output_to_into_func_destination(ctx, &output_to, schema_variant_id).await? {
                    AttributeFuncDestination::Prop(prop_id) => (Some(prop_id), None),
                    AttributeFuncDestination::OutputSocket(output_socket_id) => {
                        (None, Some(output_socket_id))
                    }
                    AttributeFuncDestination::InputSocket(_) => {
                        unreachable!("this will never happen")
                    }
                };

            FuncBinding::Attribute {
                func_id: Some(func_id),
                attribute_prototype_id: None,
                component_id: None,
                schema_variant_id: Some(schema_variant_id),
                prop_id,
                output_socket_id,
                argument_bindings: if inputs.is_empty() {
                    vec![]
                } else {
                    inputs_into_attribute_argument_bindings(
                        &inputs,
                        ctx,
                        func_id,
                        None,
                        schema_variant_id,
                    )
                    .await?
                },
            }
        }
        Binding::Authentication => FuncBinding::Authentication {
            schema_variant_id,
            func_id: Some(func_id),
        },
        Binding::CodeGeneration { inputs } => FuncBinding::CodeGeneration {
            schema_variant_id: Some(schema_variant_id),
            component_id: None,
            func_id: Some(func_id),
            attribute_prototype_id: None,
            inputs,
        },
        Binding::Management => FuncBinding::Management {
            schema_ids: None,
            schema_variant_id: Some(schema_variant_id),
            management_prototype_id: None,
            func_id: Some(func_id),
        },
        Binding::Qualification { inputs } => FuncBinding::Qualification {
            schema_variant_id: Some(schema_variant_id),
            component_id: None,
            func_id: Some(func_id),
            attribute_prototype_id: None,
            inputs,
        },
    })
}

fn parse_qualification_bindings_for_update(
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
    update_inputs: &[si_frontend_types::LeafInputLocation],
) -> FsResult<()> {
    let FuncBinding::Qualification {
        schema_variant_id,
        component_id,
        func_id,
        attribute_prototype_id,
        ..
    } = func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    final_bindings.push(FuncBinding::Qualification {
        schema_variant_id,
        component_id,
        func_id,
        attribute_prototype_id,
        inputs: update_inputs.to_owned(),
    });

    Ok(())
}

async fn create_action_binding(
    ctx: &DalContext,
    func_id: FuncId,
    action_kind: ActionKind,
    schema_variant_id: SchemaVariantId,
) -> FsResult<()> {
    ActionBinding::create_action_binding(ctx, func_id, action_kind.into(), schema_variant_id)
        .await?;
    let func = Func::get_by_id(ctx, func_id).await?;
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

    Ok(())
}

async fn create_auth_binding(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> FsResult<()> {
    AuthBinding::create_auth_binding(ctx, func_id, schema_variant_id).await?;

    let func = Func::get_by_id(ctx, func_id).await?;
    ctx.write_audit_log(
        AuditLogKind::AttachAuthFunc {
            func_id: func.id,
            func_display_name: func.display_name.clone(),
            schema_variant_id: Some(schema_variant_id),
        },
        func.name.clone(),
    )
    .await?;

    Ok(())
}

async fn create_leaf_binding(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
    leaf_kind: LeafKind,
    inputs: Vec<si_frontend_types::LeafInputLocation>,
) -> FsResult<()> {
    let inputs: Vec<_> = inputs.into_iter().map(Into::into).collect();
    LeafBinding::create_leaf_func_binding(
        ctx,
        func_id,
        EventualParent::SchemaVariant(schema_variant_id),
        leaf_kind,
        &inputs,
    )
    .await?;

    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
    let func = Func::get_by_id(ctx, func_id).await?;
    ctx.write_audit_log(
        match leaf_kind {
            LeafKind::CodeGeneration => AuditLogKind::AttachCodeGenFunc {
                func_id: func.id,
                func_display_name: func.display_name.clone(),
                schema_variant_id: Some(schema_variant_id),
                component_id: None,
                subject_name: schema_variant.display_name().to_owned(),
            },
            LeafKind::Qualification => AuditLogKind::AttachQualificationFunc {
                func_id: func.id,
                func_display_name: func.display_name.clone(),
                schema_variant_id: Some(schema_variant_id),
                component_id: None,
                subject_name: schema_variant.display_name().to_owned(),
            },
        },
        func.name,
    )
    .await?;

    Ok(())
}

async fn create_management_binding(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> FsResult<()> {
    ManagementBinding::create_management_binding(ctx, func_id, None, Some(schema_variant_id))
        .await?;

    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
    let func = Func::get_by_id(ctx, func_id).await?;
    ctx.write_audit_log(
        AuditLogKind::AttachManagementFunc {
            func_id: func.id,
            func_display_name: func.display_name.clone(),
            schema_variant_id: Some(schema_variant_id),
            component_id: None,
            subject_name: schema_variant.display_name().to_owned(),
        },
        func.name.clone(),
    )
    .await?;

    Ok(())
}

async fn create_attribute_binding(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
    prop_id: Option<dal::PropId>,
    output_socket_id: Option<dal::OutputSocketId>,
    argument_bindings: Vec<si_frontend_types::AttributeArgumentBinding>,
    func_id: FuncId,
) -> FsResult<()> {
    let eventual_parent = EventualParent::SchemaVariant(schema_variant_id);
    let output_location =
        AttributeBinding::assemble_attribute_output_location(prop_id, output_socket_id)?;
    let prototype_arguments = argument_bindings
        .into_iter()
        .filter_map(|binding| {
            AttributeBinding::assemble_attribute_input_location(
                binding.prop_id,
                binding.input_socket_id,
                None,
            )
            .ok()
            .map(
                |input_location| dal::func::binding::AttributeArgumentBinding {
                    func_argument_id: binding.func_argument_id.into_raw_id().into(),
                    attribute_prototype_argument_id: None,
                    attribute_func_input_location: input_location,
                },
            )
        })
        .collect();
    let (_, old_func) = AttributeBinding::upsert_attribute_binding(
        ctx,
        func_id,
        Some(eventual_parent),
        output_location,
        prototype_arguments,
    )
    .await?;

    if let Some(old_func_id) = old_func {
        if old_func_id != func_id {
            let old_func_summary = Func::get_by_id(ctx, old_func_id)
                .await?
                .into_frontend_type(ctx)
                .await?;

            WsEvent::func_updated(ctx, old_func_summary, None)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
    }

    let subject_name = SchemaVariant::get_by_id(ctx, schema_variant_id)
        .await?
        .display_name()
        .to_string();
    let destination_name = output_location.get_name_of_destination(ctx).await?;

    let func = Func::get_by_id(ctx, func_id).await?;
    ctx.write_audit_log(
        AuditLogKind::AttachAttributeFunc {
            func_id: func.id,
            func_display_name: func.display_name.clone(),
            schema_variant_id: Some(schema_variant_id),
            component_id: None,
            subject_name,
            prop_id,
            output_socket_id,
            destination_name,
        },
        func.name.clone(),
    )
    .await?;

    Ok(())
}

async fn create_func_binding(ctx: &DalContext, binding: FuncBinding) -> FsResult<()> {
    let maybe_schema_variant_id = match binding {
        FuncBinding::Action {
            schema_variant_id,
            func_id,
            kind,
            ..
        } => match (kind, schema_variant_id, func_id) {
            (Some(action_kind), Some(schema_variant_id), Some(func_id)) => {
                create_action_binding(ctx, func_id, action_kind, schema_variant_id).await?;
                Some(schema_variant_id)
            }
            _ => None,
        },
        FuncBinding::Attribute {
            func_id,
            schema_variant_id,
            prop_id,
            output_socket_id,
            argument_bindings,
            ..
        } => match schema_variant_id.zip(func_id) {
            Some((schema_variant_id, func_id)) => {
                create_attribute_binding(
                    ctx,
                    schema_variant_id,
                    prop_id,
                    output_socket_id,
                    argument_bindings,
                    func_id,
                )
                .await?;
                Some(schema_variant_id)
            }
            None => todo!(),
        },
        FuncBinding::Authentication {
            schema_variant_id,
            func_id,
        } => match func_id {
            Some(func_id) => {
                create_auth_binding(ctx, func_id, schema_variant_id).await?;
                Some(schema_variant_id)
            }
            None => None,
        },
        FuncBinding::CodeGeneration {
            schema_variant_id,
            func_id,
            inputs,
            ..
        } => match schema_variant_id.zip(func_id) {
            Some((schema_variant_id, func_id)) => {
                create_leaf_binding(
                    ctx,
                    func_id,
                    schema_variant_id,
                    LeafKind::CodeGeneration,
                    inputs,
                )
                .await?;
                Some(schema_variant_id)
            }
            None => None,
        },
        FuncBinding::Management {
            schema_variant_id,
            func_id,
            ..
        } => match schema_variant_id.zip(func_id) {
            Some((schema_variant_id, func_id)) => {
                create_management_binding(ctx, func_id, schema_variant_id).await?;
                Some(schema_variant_id)
            }
            None => None,
        },
        FuncBinding::Qualification {
            schema_variant_id,
            func_id,
            inputs,
            ..
        } => match schema_variant_id.zip(func_id) {
            Some((schema_variant_id, func_id)) => {
                create_leaf_binding(
                    ctx,
                    func_id,
                    schema_variant_id,
                    LeafKind::Qualification,
                    inputs,
                )
                .await?;
                Some(schema_variant_id)
            }
            None => None,
        },
    };

    if let Some(schema_variant_id) = maybe_schema_variant_id {
        let schema_id = SchemaVariant::schema_id(ctx, schema_variant_id).await?;

        let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
        WsEvent::schema_variant_updated(ctx, schema_id, schema_variant)
            .await?
            .publish_on_commit(ctx)
            .await?;
    }

    Ok(())
}

async fn delete_binding(
    ctx: &DalContext,
    func_binding: FuncBinding,
    schema_variant_id: SchemaVariantId,
    func_id: FuncId,
) -> FsResult<()> {
    let did_delete = match func_binding {
        FuncBinding::Action {
            action_prototype_id: Some(action_prototype_id),
            ..
        } => {
            ActionBinding::delete_action_binding(ctx, action_prototype_id).await?;
            true
        }
        FuncBinding::Authentication { .. } => {
            AuthBinding::delete_auth_binding(ctx, func_id, schema_variant_id).await?;
            true
        }
        FuncBinding::Management {
            management_prototype_id: Some(mgmt_proto_id),
            ..
        } => {
            ManagementBinding::delete_management_binding(ctx, mgmt_proto_id).await?;
            true
        }
        FuncBinding::CodeGeneration {
            attribute_prototype_id: Some(attribute_prototype_id),
            ..
        }
        | FuncBinding::Qualification {
            attribute_prototype_id: Some(attribute_prototype_id),
            ..
        } => {
            LeafBinding::delete_leaf_func_binding(ctx, attribute_prototype_id).await?;
            true
        }
        _ => false,
    };

    if did_delete {
        let func = Func::get_by_id(ctx, func_id).await?;
        let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
        let schema_id = SchemaVariant::schema_id(ctx, schema_variant_id).await?;

        ctx.write_audit_log(
            AuditLogKind::DetachFunc {
                func_id,
                func_display_name: func.display_name.clone(),
                schema_variant_id: Some(schema_variant_id),
                schema_ids: None,
                component_id: None,
                subject_name: schema_variant.display_name().to_owned(),
            },
            func.name.clone(),
        )
        .await?;

        WsEvent::schema_variant_updated(ctx, schema_id, schema_variant)
            .await?
            .publish_on_commit(ctx)
            .await?;
    }

    Ok(())
}

pub async fn get_identity_bindings_for_variant(
    ctx: &DalContext,
    variant_id: SchemaVariantId,
) -> FsResult<IdentityBindings> {
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?;
    let unset_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;

    let identity_bindings =
        get_bindings_for_func_and_schema_variant(ctx, identity_func_id, Some(variant_id)).await?;
    let identity_func_arg = FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;
    let unset_bindings =
        get_bindings_for_func_and_schema_variant(ctx, unset_func_id, Some(variant_id)).await?;
    let mut result = IdentityBindings {
        props: BTreeMap::new(),
        output_sockets: BTreeMap::new(),
    };

    for binding in identity_bindings
        .into_iter()
        .chain(unset_bindings.into_iter())
    {
        let FuncBinding::Attribute {
            func_id: Some(func_id),
            component_id: None,
            schema_variant_id: Some(schema_variant_id),
            prop_id,
            output_socket_id,
            argument_bindings,
            ..
        } = binding
        else {
            continue;
        };

        if schema_variant_id != variant_id {
            continue;
        }

        if ![identity_func_id, unset_func_id].contains(&func_id) {
            continue;
        }

        let is_identity = func_id == identity_func_id;

        let identity_arg = argument_bindings
            .into_iter()
            .find(|binding| binding.func_argument_id == identity_func_arg.id);

        match (prop_id, output_socket_id) {
            (None, Some(output_socket_id)) => {
                let input = if is_identity {
                    let Some(input_prop_id) = identity_arg.and_then(|arg| arg.prop_id) else {
                        continue;
                    };

                    let prop_path = Prop::path_by_id(ctx, input_prop_id)
                        .await?
                        .with_replaced_sep("/");

                    SocketIdentityBinding::Prop(prop_path)
                } else {
                    SocketIdentityBinding::Unset
                };

                let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;

                result
                    .output_sockets
                    .insert(output_socket.name().to_owned(), input);
            }
            (Some(prop_id), None) => {
                let prop = Prop::get_by_id(ctx, prop_id).await?;
                let output_prop_path = Prop::path_by_id(ctx, prop_id).await?.with_replaced_sep("/");

                let frontend_prop = prop.clone().into_frontend_type(ctx).await?;
                if !frontend_prop.eligible_to_receive_data {
                    continue;
                }

                let input = if is_identity {
                    match identity_arg.map(|arg| (arg.prop_id, arg.input_socket_id)) {
                        Some((Some(input_prop_id), None)) => {
                            let prop_path = Prop::path_by_id(ctx, input_prop_id)
                                .await?
                                .with_replaced_sep("/");
                            PropIdentityBinding::Prop(prop_path)
                        }
                        Some((None, Some(input_socket_id))) => {
                            let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;

                            PropIdentityBinding::InputSocket(input_socket.name().to_owned())
                        }
                        _ => continue,
                    }
                } else {
                    PropIdentityBinding::Unset
                };

                result.props.insert(output_prop_path, input);
            }
            _ => continue,
        }
    }

    Ok(result)
}

pub async fn set_func_bindings(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id, func_id)): Path<(
        WorkspaceId,
        ChangeSetId,
        SchemaId,
        FuncId,
    )>,
    Json(request): Json<SetFuncBindingsRequest>,
) -> FsResult<Json<Option<fs::Func>>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    tracker.track(
        &ctx,
        "fs/set_func_bindings",
        serde_json::json!({
            "schema_id": schema_id,
            "func_id": func_id,
            "payload": &request
        }),
    );

    let func = Func::get_by_id_opt(&ctx, func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    let variant = if request.is_attaching_existing {
        get_or_unlock_schema(&ctx, schema_id).await?
    } else {
        lookup_variant_for_schema(&ctx, schema_id, true)
            .await?
            .ok_or(FsError::ResourceNotFound)?
    };

    if matches!(
        func.kind,
        FuncKind::Authentication | FuncKind::Intrinsic | FuncKind::Unknown
    ) {
        return Err(FsError::FuncBindingKindMismatch);
    }

    let (fs_bindings, current_bindings) = get_bindings(&ctx, func_id, schema_id).await?;

    if !current_bindings.is_empty() && request.is_attaching_existing {
        let types_size = func_types_size(&ctx, func.id).await?;
        return Ok(Json(Some(dal_func_to_fs_func(
            &func,
            fs_bindings.byte_size(),
            types_size,
        ))));
    }

    let request_bindings = request.bindings.bindings;

    let mut delete_bindings = vec![];
    let mut bindings_to_update = vec![];
    for (idx, func_binding) in current_bindings.into_iter().enumerate() {
        match request_bindings.get(idx) {
            Some(binding_update) => {
                parse_binding_for_update(
                    &ctx,
                    binding_update,
                    &mut bindings_to_update,
                    func_binding,
                )
                .await?;
            }
            None => {
                delete_bindings.push(func_binding);
            }
        }
    }

    let bindings_to_create = request_bindings.get(bindings_to_update.len()..);

    if request.is_attaching_existing {
        match bindings_to_create {
            Some(new_bindings) => {
                let Some(new_binding) = new_bindings.first() else {
                    return Ok(Json(None));
                };

                let func_binding =
                    parse_binding_for_create(&ctx, new_binding.to_owned(), variant.id(), func_id)
                        .await?;
                create_func_binding(&ctx, func_binding).await?;
            }
            None => {
                return Ok(Json(None));
            }
        }
    } else {
        match func.kind {
            FuncKind::Attribute => {
                let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
                update_attribute_func_bindings(&ctx, bindings_to_update).await?;
                drop(cycle_check_guard);
            }
            FuncKind::Action => {
                update_action_func_bindings(&ctx, bindings_to_update).await?;
            }
            FuncKind::CodeGeneration | FuncKind::Qualification => {
                let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
                update_leaf_func_bindings(&ctx, bindings_to_update).await?;
                drop(cycle_check_guard);
            }
            _ => return Err(FsError::FuncBindingKindMismatch),
        }

        if let Some(new_bindings) = bindings_to_create {
            for new_binding in new_bindings {
                create_func_binding(
                    &ctx,
                    parse_binding_for_create(&ctx, new_binding.to_owned(), variant.id(), func_id)
                        .await?,
                )
                .await?;
            }
        }

        for binding_to_delete in delete_bindings {
            delete_binding(&ctx, binding_to_delete, variant.id(), func_id).await?;
        }
    }

    let func_summary = Func::get_by_id(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;
    WsEvent::func_updated(&ctx, func_summary.clone(), None)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let (fs_bindings, _) = get_bindings(&ctx, func_id, schema_id).await?;
    let types_size = func_types_size(&ctx, func.id).await?;

    ctx.commit().await?;

    Ok(Json(Some(dal_func_to_fs_func(
        &func,
        fs_bindings.byte_size(),
        types_size,
    ))))
}

pub async fn get_func_bindings(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id, func_id)): Path<(
        WorkspaceId,
        ChangeSetId,
        SchemaId,
        FuncId,
    )>,
) -> FsResult<Json<fs::Bindings>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(
        &ctx,
        "fs/get_func_bindings",
        serde_json::json!({
            "schema_id": schema_id,
            "func_id": func_id,
        }),
    );

    let (fs_bindings, _) = get_bindings(&ctx, func_id, schema_id).await?;

    Ok(Json(fs_bindings))
}

pub async fn get_identity_bindings(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
    Query(variant_query): Query<VariantQuery>,
) -> FsResult<Json<IdentityBindings>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(
        &ctx,
        "fs/get_identity_bindings",
        serde_json::json!({
            "schema_id": schema_id,
            "unlocked": variant_query.unlocked,
        }),
    );

    let variant = lookup_variant_for_schema(&ctx, schema_id, variant_query.unlocked)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    let result = get_identity_bindings_for_variant(&ctx, variant.id()).await?;

    Ok(Json(result))
}

pub async fn set_identity_bindings(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
    Json(request): Json<IdentityBindings>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    tracker.track(
        &ctx,
        "fs/set_identity_bindings",
        serde_json::json!({
            "schema_id": schema_id,
            "payload": &request,
        }),
    );

    let unlocked_variant = get_or_unlock_schema(&ctx, schema_id).await?;
    let schema_variant_id = unlocked_variant.id();
    let identity_func_id = Func::find_intrinsic(&ctx, IntrinsicFunc::Identity).await?;
    let unset_func_id = Func::find_intrinsic(&ctx, IntrinsicFunc::Unset).await?;
    let identity_func_arg = FuncArgument::find_by_name_for_func(&ctx, "identity", identity_func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    for (output_socket_name, output_socket_binding) in request.output_sockets {
        let (func_id, prototype_arguments) = match output_socket_binding {
            SocketIdentityBinding::Prop(prop_path) => {
                let prop_id = Prop::find_prop_id_by_path_opt(
                    &ctx,
                    schema_variant_id,
                    &PropPath::new(prop_path.split("/")),
                )
                .await?
                .ok_or(FsError::PropNotFound(prop_path))?;

                let arguments = vec![AttributeArgumentBinding {
                    func_argument_id: identity_func_arg.id,
                    attribute_prototype_argument_id: None,
                    attribute_func_input_location:
                        dal::func::binding::AttributeFuncArgumentSource::Prop(prop_id),
                }];
                (identity_func_id, arguments)
            }
            SocketIdentityBinding::Unset => (unset_func_id, vec![]),
        };

        let output_socket =
            OutputSocket::find_with_name_or_error(&ctx, output_socket_name, schema_variant_id)
                .await?;

        let output_location = AttributeFuncDestination::OutputSocket(output_socket.id());

        AttributeBinding::upsert_attribute_binding(
            &ctx,
            func_id,
            Some(EventualParent::SchemaVariant(schema_variant_id)),
            output_location,
            prototype_arguments,
        )
        .await?;
    }

    for (prop_path, prop_binding) in request.props {
        let prop_id = Prop::find_prop_id_by_path_opt(
            &ctx,
            schema_variant_id,
            &PropPath::new(prop_path.split("/")),
        )
        .await?
        .ok_or(FsError::PropNotFound(prop_path.clone()))?;

        let output_location = AttributeFuncDestination::Prop(prop_id);

        let (func_id, prototype_arguments) = match prop_binding {
            PropIdentityBinding::Prop(input_prop_path) => {
                let input_prop_id = Prop::find_prop_id_by_path_opt(
                    &ctx,
                    schema_variant_id,
                    &PropPath::new(input_prop_path.split("/")),
                )
                .await?
                .ok_or(FsError::PropNotFound(input_prop_path))?;

                let arguments = vec![AttributeArgumentBinding {
                    func_argument_id: identity_func_arg.id,
                    attribute_prototype_argument_id: None,
                    attribute_func_input_location: AttributeFuncArgumentSource::Prop(input_prop_id),
                }];
                (identity_func_id, arguments)
            }
            PropIdentityBinding::InputSocket(input_socket_name) => {
                let input_socket = InputSocket::find_with_name_or_error(
                    &ctx,
                    input_socket_name,
                    schema_variant_id,
                )
                .await?;

                let arguments = vec![AttributeArgumentBinding {
                    func_argument_id: identity_func_arg.id,
                    attribute_prototype_argument_id: None,
                    attribute_func_input_location: AttributeFuncArgumentSource::InputSocket(
                        input_socket.id(),
                    ),
                }];
                (identity_func_id, arguments)
            }
            PropIdentityBinding::Unset => (unset_func_id, vec![]),
        };

        AttributeBinding::upsert_attribute_binding(
            &ctx,
            func_id,
            Some(EventualParent::SchemaVariant(schema_variant_id)),
            output_location,
            prototype_arguments,
        )
        .await?;
    }

    for func_id in [identity_func_id, unset_func_id] {
        let func_summary = Func::get_by_id(&ctx, func_id)
            .await?
            .into_frontend_type(&ctx)
            .await?;
        WsEvent::func_updated(&ctx, func_summary.clone(), None)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    Ok(())
}
