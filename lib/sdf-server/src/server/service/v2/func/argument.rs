use axum::{
    extract::{OriginalUri, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::{schema::variant, ChangeSetId, Schema, SchemaVariant, SchemaVariantId, WorkspacePk};
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
pub struct FuncArgumentRequest {
    name: String,
    kind: FuncArgumentKind,
    element_kind: Option<FuncArgumentKind>,
}

pub async fn list_arguments(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
) -> FuncAPIResult<Json<Vec<FuncArgument>>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let func_arguments = FuncArgument::list_for_func(&ctx, func_id).await?;

    Ok(Json(func_arguments))
}

pub async fn create_func_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<FuncArgumentRequest>,
) -> FuncAPIResult<Json<Vec<FuncArgument>>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::create_func_argument(
        &ctx,
        func_id,
        request.name,
        request.kind,
        request.element_kind,
    )
    .await?;
    WsEvent::func_arguments_saved(&ctx, request.func_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}

pub async fn update_func_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id, func_argument_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        FuncId,
        FuncArgumentId,
    )>,
    Json(request): Json<FuncArgumentRequest>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::update_func_argument(
        &ctx,
        func_arg_id,
        request.name,
        request.kind,
        request.element_kind,
    )
    .await?;

    WsEvent::func_arguments_saved(&ctx, request.func_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}

pub async fn delete_func_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id, func_argument_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        FuncId,
        FuncArgumentId,
    )>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncAuthoringClient::delete_func_argument(&ctx, func_arg_id).await?;

    WsEvent::func_arguments_saved(&ctx, request.func_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
