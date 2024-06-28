use axum::{
    extract::{OriginalUri, Path},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use dal::{
    func::{
        authoring::FuncAuthoringClient,
        binding::{
            AttributeArgumentBinding, AttributeFuncArgumentSource, AttributeFuncDestination,
            EventualParent, FuncBindings,
        },
        FuncKind,
    },
    schema::variant::leaves::{LeafInputLocation, LeafKind},
    ChangeSet, ChangeSetId, Func, WorkspacePk, WsEvent,
};
use si_frontend_types::{self as frontend_types, FuncBinding, FuncCode, FuncSummary};

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    tracking::track,
};

use super::{get_code_response, get_types, FuncAPIError, FuncAPIResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncRequest {
    name: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    binding: frontend_types::FuncBinding,
    kind: FuncKind,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncResponse {
    summary: FuncSummary,
    code: FuncCode,
    binding: frontend_types::FuncBindings,
}

pub async fn create_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<CreateFuncRequest>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    if let Some(name) = request.name.as_deref() {
        if dal::func::is_intrinsic(name)
            || ["si:resourcePayloadToValue", "si:normalizeToArray"].contains(&name)
        {
            return Err(FuncAPIError::FuncNameReserved(name.into()));
        }
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let created_func_response = match request.kind {
        FuncKind::Action => {
            if let FuncBinding::Action {
                schema_variant_id: Some(schema_variant_id),
                kind: Some(kind),
                ..
            } = request.binding
            {
                let created_func = FuncAuthoringClient::create_new_action_func(
                    &ctx,
                    request.name,
                    kind.into(),
                    schema_variant_id.into(),
                )
                .await?;

                let func = Func::get_by_id_or_error(&ctx, created_func.id).await?;
                let binding = FuncBindings::from_func_id(&ctx, created_func.id)
                    .await?
                    .into_frontend_type();
                CreateFuncResponse {
                    summary: func.into_frontend_type(&ctx).await?,
                    code: get_code_response(&ctx, created_func.id).await?,
                    binding,
                }
            } else {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        }
        FuncKind::Attribute => {
            if let FuncBinding::Attribute {
                prop_id,
                output_socket_id,
                argument_bindings,
                component_id,
                ..
            } = request.binding
            {
                let output_location = if let Some(prop_id) = prop_id {
                    AttributeFuncDestination::Prop(prop_id.into())
                } else if let Some(output_socket_id) = output_socket_id {
                    AttributeFuncDestination::OutputSocket(output_socket_id.into())
                } else {
                    return Err(FuncAPIError::MissingOutputLocationForAttributeFunc);
                };
                let eventual_parent =
                    component_id.map(|component_id| EventualParent::Component(component_id.into()));
                let mut arg_bindings = vec![];
                for arg_binding in argument_bindings {
                    let input_location = if let Some(prop_id) = arg_binding.prop_id {
                        AttributeFuncArgumentSource::Prop(prop_id.into())
                    } else if let Some(input_socket_id) = arg_binding.input_socket_id {
                        AttributeFuncArgumentSource::InputSocket(input_socket_id.into())
                    } else {
                        return Err(FuncAPIError::MissingInputLocationForAttributeFunc);
                    };
                    arg_bindings.push(AttributeArgumentBinding {
                        func_argument_id: arg_binding.func_argument_id.into(),
                        attribute_prototype_argument_id: arg_binding
                            .attribute_prototype_argument_id
                            .map(|a| a.into()),
                        attribute_func_input_location: input_location,
                    });
                }

                let created_func = FuncAuthoringClient::create_new_attribute_func(
                    &ctx,
                    request.name,
                    eventual_parent,
                    output_location,
                    arg_bindings,
                )
                .await?;

                let binding = FuncBindings::from_func_id(&ctx, created_func.id)
                    .await?
                    .into_frontend_type();
                let func = Func::get_by_id_or_error(&ctx, created_func.id).await?;
                CreateFuncResponse {
                    summary: func.into_frontend_type(&ctx).await?,
                    code: get_code_response(&ctx, func.id).await?,
                    binding,
                }
            } else {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        }
        FuncKind::Authentication => {
            if let FuncBinding::Authentication {
                schema_variant_id,
                func_id: _,
            } = request.binding
            {
                let created_func = FuncAuthoringClient::create_new_auth_func(
                    &ctx,
                    request.name,
                    schema_variant_id.into(),
                )
                .await?;

                let binding = FuncBindings::from_func_id(&ctx, created_func.id)
                    .await?
                    .into_frontend_type();
                let new_func = Func::get_by_id_or_error(&ctx, created_func.id).await?;
                CreateFuncResponse {
                    summary: new_func.into_frontend_type(&ctx).await?,
                    code: get_code_response(&ctx, created_func.id).await?,
                    binding,
                }
            } else {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        }
        FuncKind::CodeGeneration => {
            if let FuncBinding::CodeGeneration {
                schema_variant_id: Some(schema_variant_id),
                inputs,
                ..
            } = request.binding
            {
                let inputs = if inputs.is_empty() {
                    vec![LeafInputLocation::Domain]
                } else {
                    inputs.into_iter().map(|input| input.into()).collect()
                };
                let created_func = FuncAuthoringClient::create_new_leaf_func(
                    &ctx,
                    request.name,
                    LeafKind::CodeGeneration,
                    EventualParent::SchemaVariant(schema_variant_id.into()),
                    &inputs,
                )
                .await?;
                let binding = FuncBindings::from_func_id(&ctx, created_func.id)
                    .await?
                    .into_frontend_type();
                let new_func = Func::get_by_id_or_error(&ctx, created_func.id).await?;
                CreateFuncResponse {
                    summary: new_func.into_frontend_type(&ctx).await?,
                    code: get_code_response(&ctx, created_func.id).await?,
                    binding,
                }
            } else {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        }
        FuncKind::Qualification => {
            if let FuncBinding::Qualification {
                schema_variant_id: Some(schema_variant_id),
                inputs,
                ..
            } = request.binding
            {
                let inputs = if inputs.is_empty() {
                    vec![LeafInputLocation::Domain, LeafInputLocation::Code]
                } else {
                    inputs.into_iter().map(|input| input.into()).collect()
                };

                let created_func = FuncAuthoringClient::create_new_leaf_func(
                    &ctx,
                    request.name,
                    LeafKind::Qualification,
                    EventualParent::SchemaVariant(schema_variant_id.into()),
                    &inputs,
                )
                .await?;
                let binding = FuncBindings::from_func_id(&ctx, created_func.id)
                    .await?
                    .into_frontend_type();
                let new_func = Func::get_by_id_or_error(&ctx, created_func.id)
                    .await?
                    .into_frontend_type(&ctx)
                    .await?;
                CreateFuncResponse {
                    summary: new_func,
                    code: get_code_response(&ctx, created_func.id).await?,
                    binding,
                }
            } else {
                return Err(FuncAPIError::WrongFunctionKindForBinding);
            }
        }
        _ => return Err(FuncAPIError::WrongFunctionKindForBinding),
    };
    let types = get_types(
        &ctx,
        created_func_response.summary.func_id.into_raw_id().into(),
    )
    .await?;

    WsEvent::func_created(&ctx, created_func_response.summary.clone(), types)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "created_func",
        serde_json::json!({
            "how": "/func/created_func",
            "func_id": created_func_response.summary.func_id,
            "func_name": created_func_response.summary.name.to_owned(),
            "func_kind": created_func_response.summary.kind,
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(&created_func_response)?)?)
}
