use axum::{
    extract::{OriginalUri, Path},
    response::IntoResponse,
    Json,
};
use dal::{
    func::binding::{
        action::ActionBinding, attribute::AttributeBinding, authentication::AuthBinding,
        leaf::LeafBinding, AttributeArgumentBinding, EventualParent,
    },
    schema::variant::leaves::{LeafInputLocation, LeafKind},
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

pub async fn create_binding(
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
                    AuthBinding::create_auth_binding(
                        &ctx,
                        func_id.into(),
                        schema_variant_id.into(),
                    )
                    .await?;
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
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "created_binding",
        serde_json::json!({
            "how": "/func/created_binding",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );

    let types = get_types(&ctx, func_id).await?;
    let binding = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?
        .bindings;

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
