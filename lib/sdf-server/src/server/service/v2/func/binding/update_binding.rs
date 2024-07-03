use axum::{
    extract::{OriginalUri, Path},
    response::IntoResponse,
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

use crate::{
    server::{
        extract::{AccessBuilder, HandlerContext, PosthogClient},
        tracking::track,
    },
    service::v2::func::{get_types, FuncAPIError, FuncAPIResult},
};

pub async fn update_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncBindings>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id_or_error(&ctx, func_id).await?;
    match func.kind {
        dal::func::FuncKind::Attribute => {
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
                                    AttributeArgumentBinding::assemble_attribute_input_location(
                                        arg_binding.prop_id,
                                        arg_binding.input_socket_id,
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
    let binding = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?
        .bindings;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "update_binding",
        serde_json::json!({
            "how": "/func/update_binding",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );
    let types = get_types(&ctx, func_id).await?;
    WsEvent::func_bindings_updated(&ctx, binding.clone(), types)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&binding)?)?)
}
