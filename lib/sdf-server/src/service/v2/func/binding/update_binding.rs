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
    DalContext,
    Func,
    FuncId,
    WorkspacePk,
    WsEvent,
    func::{
        FuncKind,
        binding::{
            AttributeArgumentBinding,
            action::ActionBinding,
            attribute::AttributeBinding,
            leaf::LeafBinding,
        },
    },
};
use si_frontend_types::{
    self as frontend_types,
    FuncBinding,
};

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

pub async fn update_attribute_func_bindings(
    ctx: &DalContext,
    bindings: Vec<FuncBinding>,
) -> FuncAPIResult<()> {
    for binding in bindings {
        let frontend_types::FuncBinding::Attribute {
            argument_bindings,
            attribute_prototype_id,
            ..
        } = binding
        else {
            continue;
        };

        let attribute_prototype_id =
            attribute_prototype_id.ok_or(FuncAPIError::MissingPrototypeId)?;
        let mut arguments: Vec<AttributeArgumentBinding> = vec![];
        for arg_binding in argument_bindings {
            let input_location = AttributeBinding::assemble_attribute_input_location(
                arg_binding.prop_id,
                arg_binding.input_socket_id,
                arg_binding.static_value,
            )?;
            arguments.push(AttributeArgumentBinding {
                func_argument_id: arg_binding.func_argument_id.into_raw_id().into(),
                attribute_func_input_location: input_location,
                attribute_prototype_argument_id: None, // when creating a new prototype,
                                                       // we don't have the attribute prototype arguments yet
            });
        }

        AttributeBinding::update_attribute_binding_arguments(
            ctx,
            attribute_prototype_id.into_raw_id().into(),
            arguments,
        )
        .await?;
    }

    Ok(())
}

pub async fn update_action_func_bindings(
    ctx: &DalContext,
    bindings: Vec<FuncBinding>,
) -> FuncAPIResult<()> {
    for binding in bindings {
        let frontend_types::FuncBinding::Action {
            action_prototype_id,
            kind,
            ..
        } = binding
        else {
            continue;
        };

        let (action_prototype_id, kind) = action_prototype_id
            .zip(kind)
            .ok_or(FuncAPIError::MissingActionKindForActionFunc)?;

        ActionBinding::update_action_binding(
            ctx,
            action_prototype_id.into_raw_id().into(),
            kind.into(),
        )
        .await?;
    }

    Ok(())
}

pub async fn update_leaf_func_bindings(
    ctx: &DalContext,
    bindings: Vec<FuncBinding>,
) -> FuncAPIResult<()> {
    for binding in bindings {
        let inputs: Vec<_> = binding
            .leaf_inputs()
            .ok_or(FuncAPIError::MissingInputLocationForLeafFunc)?
            .iter()
            .copied()
            .map(Into::into)
            .collect();

        let leaf_binding_proto = binding
            .leaf_binding_prototype()
            .ok_or(FuncAPIError::MissingPrototypeId)?;

        match binding {
            FuncBinding::CodeGeneration { .. } => {
                LeafBinding::update_leaf_func_binding(ctx, leaf_binding_proto, &inputs).await?;
            }
            FuncBinding::Qualification { .. } => {
                LeafBinding::update_leaf_func_binding(ctx, leaf_binding_proto, &inputs).await?;
            }

            _ => return Err(FuncAPIError::WrongFunctionKindForBinding),
        }
    }

    Ok(())
}

pub async fn update_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncBindings>,
) -> FuncAPIResult<ForceChangeSetResponse<Vec<frontend_types::FuncBinding>>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id(&ctx, func_id).await?;
    // add cycle check so we don't end up with a cycle as a result of updating this binding
    let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;

    if func.is_transformation {
        return Err(FuncAPIError::WrongFunctionKindForBinding);
    }

    match func.kind {
        FuncKind::Attribute | FuncKind::Intrinsic => {
            update_attribute_func_bindings(&ctx, request.bindings).await?;
        }
        FuncKind::Action => {
            update_action_func_bindings(&ctx, request.bindings).await?;
        }
        FuncKind::CodeGeneration | FuncKind::Qualification => {
            update_leaf_func_bindings(&ctx, request.bindings).await?;
        }
        _ => return Err(FuncAPIError::WrongFunctionKindForBinding),
    }

    drop(cycle_check_guard);

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "update_binding",
        serde_json::json!({
            "how": "/func/update_binding",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );
    let binding = Func::get_by_id(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?
        .bindings;
    let func_summary = Func::get_by_id(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;
    WsEvent::func_updated(&ctx, func_summary.clone(), None)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, binding))
}
