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
        binding::{
            attribute_argument::AttributeArgumentBinding, AttributeArgumentBinding, FuncBinding,
            FuncBindings,
        },
    },
    schema::variant,
    AttributePrototypeId, ChangeSet, ChangeSetId, FuncId, Schema, SchemaVariant, SchemaVariantId,
    WorkspacePk,
};
use si_frontend_types as frontend_types;
use thiserror::Error;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    state::AppState,
    tracking::track,
};

use super::{ApiError, FuncAPIResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttributeBindingRequest {
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
    component_id: Option<ComponentId>,
    prop_id: Option<PropId>,
    output_socket_id: Option<OutputSocketId>,
    argument_bindings: Vec<AttributeArgumentBinding>,
    #[serde(flatten)]
    pub visibility: Visibility,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAttributeBindingRequest {
    func_id: FuncId,
    attribute_prototype_id: AttributePrototypeId,
    prop_id: Option<PropId>,
    output_socket_id: Option<OutputSocketId>,
    argument_bindings: Vec<AttributeArgumentBinding>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn create_attribute_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<CreateAttributeBindingRequest>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::create_attribute_prototype(
        &ctx,
        func_id,
        request.schema_variant_id,
        None,
        request.prop_id,
        request.output_socket_id,
        request.argument_bindings,
    )
    .await?;

    //todo wsevent

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}

pub async fn update_attribute_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<UpdateAttributeBindingRequest>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::create_attribute_prototype(
        &ctx,
        func_id,
        request.attribute_prototype_id,
        request.prop_id,
        request.output_socket_id,
        request.argument_bindings,
    )
    .await?;

    //todo wsevent

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
pub async fn reset_attribute_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<AttributePrototypeId>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::remove_attribute_prototype(&ctx, func_id, request).await?;

    //todo wsevent

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
