use axum::{
    extract::{OriginalUri, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::{
    func::{
        authoring::FuncAuthoringClient,
        binding::{FuncBinding, FuncBindings},
    },
    schema::variant,
    ChangeSetId, Func, FuncId, Schema, SchemaVariant, SchemaVariantId, WorkspacePk,
};
use si_frontend_types as frontend_types;
use thiserror::Error;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    state::AppState,
    tracking::track,
};

use super::{ApiError, FuncAPIError, FuncAPIResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRequest {
    pub func_ids: Vec<FuncId>,
}

pub async fn list_bindings(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<GetRequest>,
) -> FuncAPIResult<Json<Vec<frontend_types::FuncBindings>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let mut bindings = vec![];
    for func_id in request.func_ids {
        let binding = FuncBindings::from_func_id(&ctx, func_id)
            .await?
            .into_frontend_type();
        bindings.push(binding);
    }

    Ok(Json(bindings))
}

pub async fn create_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncBindings>,
) -> FuncAPIResult<Json<frontend_types::FuncBindings>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id_or_error(ctx, func_id).await?;
    match func.kind {
        dal::func::FuncKind::Action => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Action {
                    schema_variant_id,
                    action_prototype_id,
                    func_id,
                    kind,
                } = binding
                {
                    if let Some(aciton_prototype_id) = action_prototype_id {
                        FuncBindings::action::delete_action_binding(action_prototype_id, kind)
                            .await?;
                    } else {
                        return Err(FuncAPIError::MalformedRequest);
                    }
                } else {
                    return Err(FuncAPIError::MalformedRequest);
                }
            }
        }
        dal::func::FuncKind::CodeGeneration | dal::func::FuncKind::Qualification => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::CodeGeneration {
                    schema_variant_id,
                    component_id,
                    func_id,
                    attribute_prototype_id,
                    inputs,
                } = binding
                {
                    FuncBindings::leaf::delete_leaf_func_binding(
                        &ctx,
                        attribute_prototype_id,
                        inputs,
                    )
                    .await?;
                } else if let frontend_types::FuncBinding::Qualification {
                    schema_variant_id,
                    component_id,
                    func_id,
                    attribute_prototype_id,
                    inputs,
                } = binding
                {
                    FuncBindings::leaf::delete_leaf_func_binding(
                        &ctx,
                        attribute_prototype_id,
                        inputs,
                    )?;
                } else {
                    return Err(FuncAPIError::MalformedRequest);
                }
            }
        }
        _ => return Err(FuncAPIError::MalformedRequest),
    }
    let binding = FuncBindings::from_func_id(&ctx, func_id)
        .await?
        .into_frontend_type();
    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(Json(binding))?)
}
pub async fn update_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncBindings>,
) -> FuncAPIResult<Json<frontend_types::FuncBindings>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id_or_error(ctx, func_id).await?;
    match func.kind {
        dal::func::FuncKind::Action => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Action {
                    schema_variant_id,
                    action_prototype_id,
                    func_id,
                    kind,
                } = binding
                {
                    if let Some(aciton_prototype_id) = action_prototype_id {
                        FuncBindings::action::update_action_binding(action_prototype_id, kind)
                            .await?;
                    } else {
                        return Err(FuncAPIError::MalformedRequest);
                    }
                } else {
                    return Err(FuncAPIError::MalformedRequest);
                }
            }
        }
        dal::func::FuncKind::CodeGeneration | dal::func::FuncKind::Qualification => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::CodeGeneration {
                    schema_variant_id,
                    component_id,
                    func_id,
                    attribute_prototype_id,
                    inputs,
                } = binding
                {
                    FuncBindings::leaf::update_leaf_func_binding(
                        &ctx,
                        attribute_prototype_id,
                        inputs,
                    )
                    .await?;
                } else if let frontend_types::FuncBinding::Qualification {
                    schema_variant_id,
                    component_id,
                    func_id,
                    attribute_prototype_id,
                    inputs,
                } = binding
                {
                    FuncBindings::leaf::update_leaf_func_binding(
                        &ctx,
                        attribute_prototype_id,
                        inputs,
                    )?;
                } else {
                    return Err(FuncAPIError::MalformedRequest);
                }
            }
        }
        _ => return Err(FuncAPIError::MalformedRequest),
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(Json(binding))?)
}
pub async fn delete_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<FuncBinding>,
) -> FuncAPIResult<Json<frontend_types::FuncBindings>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let func = Func::get_by_id_or_error(ctx, func_id).await?;
    match func.kind {
        dal::func::FuncKind::Action => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Action {
                    schema_variant_id,
                    action_prototype_id,
                    func_id,
                    kind,
                } = binding
                {
                    if let Some(aciton_prototype_id) = action_prototype_id {
                        FuncBindings::action::delete_action_binding(action_prototype_id).await?;
                    } else {
                        return Err(FuncAPIError::MalformedRequest);
                    }
                } else {
                    return Err(FuncAPIError::MalformedRequest);
                }
            }
        }
        dal::func::FuncKind::CodeGeneration | dal::func::FuncKind::Qualification => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::CodeGeneration {
                    schema_variant_id,
                    component_id,
                    func_id,
                    attribute_prototype_id,
                    inputs,
                } = binding
                {
                    FuncBindings::leaf::delete_leaf_func_binding(
                        &ctx,
                        attribute_prototype_id,
                        inputs,
                    )
                    .await?;
                } else if let frontend_types::FuncBinding::Qualification {
                    schema_variant_id,
                    component_id,
                    func_id,
                    attribute_prototype_id,
                    inputs,
                } = binding
                {
                    FuncBindings::leaf::delete_leaf_func_binding(
                        &ctx,
                        attribute_prototype_id,
                        inputs,
                    )?;
                } else {
                    return Err(FuncAPIError::MalformedRequest);
                }
            }
        }
        _ => return Err(FuncAPIError::MalformedRequest),
    }
    let binding = FuncBindings::from_func_id(&ctx, func_id)
        .await?
        .into_frontend_type();

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(Json(binding))?)
}
