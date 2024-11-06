use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{
    func::binding::{
        action::ActionBinding, attribute::AttributeBinding, leaf::LeafBinding,
        AttributeArgumentBinding,
    },
    schema::variant::leaves::LeafInputLocation,
    ChangeSet, ChangeSetId, Func, FuncId, WorkspacePk, WsEvent,
};
use si_frontend_types as frontend_types;

use crate::{FuncAPIError, FuncAPIResult};
use axum_util::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

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
    let func = Func::get_by_id_or_error(&ctx, func_id).await?;
    // add cycle check so we don't end up with a cycle as a result of updating this binding
    let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
    match func.kind {
        dal::func::FuncKind::Attribute | dal::func::FuncKind::Intrinsic => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Attribute {
                    argument_bindings,
                    attribute_prototype_id,
                    ..
                } = binding
                {
                    match attribute_prototype_id {
                        Some(attribute_prototype_id) => {
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

                            AttributeBinding::update_attribute_binding_arguments(
                                &ctx,
                                attribute_prototype_id.into_raw_id().into(),
                                arguments,
                            )
                            .await?;
                        }
                        None => return Err(FuncAPIError::MissingPrototypeId),
                    }
                }
            }
        }
        dal::func::FuncKind::Action => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Action {
                    action_prototype_id,
                    kind,
                    ..
                } = binding
                {
                    match (action_prototype_id, kind) {
                        (Some(action_prototype_id), Some(kind)) => {
                            ActionBinding::update_action_binding(
                                &ctx,
                                action_prototype_id.into_raw_id().into(),
                                kind.into(),
                            )
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
                    attribute_prototype_id,
                    inputs,
                    ..
                } = binding
                {
                    match attribute_prototype_id {
                        Some(attribute_prototype_id) => {
                            let inputs: Vec<LeafInputLocation> =
                                inputs.into_iter().map(|input| input.into()).collect();
                            LeafBinding::update_leaf_func_binding(
                                &ctx,
                                attribute_prototype_id.into_raw_id().into(),
                                &inputs,
                            )
                            .await?;
                        }
                        None => {
                            return Err(FuncAPIError::MissingPrototypeId);
                        }
                    }
                } else if let frontend_types::FuncBinding::Qualification {
                    attribute_prototype_id,
                    inputs,
                    ..
                } = binding
                {
                    match attribute_prototype_id {
                        Some(attribute_prototype_id) => {
                            let inputs: Vec<LeafInputLocation> =
                                inputs.into_iter().map(|input| input.into()).collect();
                            LeafBinding::update_leaf_func_binding(
                                &ctx,
                                attribute_prototype_id.into_raw_id().into(),
                                &inputs,
                            )
                            .await?;
                        }
                        None => {
                            return Err(FuncAPIError::MissingPrototypeId);
                        }
                    }
                } else {
                    return Err(FuncAPIError::WrongFunctionKindForBinding);
                }
            }
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
    let binding = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?
        .bindings;
    let func_summary = Func::get_by_id_or_error(&ctx, func_id)
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
