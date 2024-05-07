use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::{
    ChangeSet, Component, ComponentId, InputSocket, InputSocketId, OutputSocket, OutputSocketId,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]

pub struct DeleteConnectionRequest {
    pub from_socket_id: OutputSocketId,
    pub from_component_id: ComponentId,
    pub to_component_id: ComponentId,
    pub to_socket_id: InputSocketId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a [`Connection`](dal::Connection) via its EdgeId. Creating change-set if on head.
pub async fn delete_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteConnectionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    Component::remove_connection(
        &ctx,
        request.to_socket_id,
        request.from_socket_id,
        request.to_component_id,
        request.from_component_id,
    )
    .await?;

    let from_component_schema =
        Component::schema_for_component_id(&ctx, request.from_component_id).await?;

    let to_component_schema =
        Component::schema_for_component_id(&ctx, request.to_component_id).await?;

    let output_socket = OutputSocket::get_by_id(&ctx, request.from_socket_id).await?;
    let input_socket = InputSocket::get_by_id(&ctx, request.to_socket_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "delete_connection",
        serde_json::json!({
            "how": "/diagram/delete_connection",
            "from_component_id": request.from_component_id,
            "from_component_schema_name": from_component_schema.name(),
            "from_socket_id": request.from_socket_id,
            "from_socket_name": &output_socket.name(),
            "to_component_id": request.to_component_id,
            "to_component_schema_name": to_component_schema.name(),
            "to_socket_id": request.to_socket_id,
            "to_socket_name":  &input_socket.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    WsEvent::connection_deleted(
        &ctx,
        request.from_component_id,
        request.to_component_id,
        request.from_socket_id,
        request.to_socket_id,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
