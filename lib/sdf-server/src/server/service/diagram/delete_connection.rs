use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::edge::EdgeId;
use dal::{ChangeSet, Component, Connection, Edge, Socket, Visibility};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::DiagramError;
use dal::standard_model::StandardModel;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteConnectionRequest {
    pub edge_id: EdgeId,
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

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;
    let edge = Edge::get_by_id(&ctx, &request.edge_id)
        .await?
        .ok_or(DiagramError::EdgeNotFound)?;

    let conn = Connection::from_edge(&edge);
    let from_component_schema = Component::get_by_id(&ctx, &conn.source.component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let from_socket = Socket::get_by_id(&ctx, &conn.source.socket_id)
        .await?
        .ok_or(DiagramError::SocketNotFound)?;

    let to_component_schema = Component::get_by_id(&ctx, &conn.destination.component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let to_socket = Socket::get_by_id(&ctx, &conn.destination.socket_id)
        .await?
        .ok_or(DiagramError::SocketNotFound)?;

    Connection::delete_for_edge(&ctx, request.edge_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "delete_connection",
        serde_json::json!({
            "from_component_id": conn.source.component_id,
            "from_component_schema_name": from_component_schema.name(),
            "from_socket_id": conn.source.socket_id,
            "from_socket_name": &from_socket.name(),
            "to_component_id": conn.destination.component_id,
            "to_component_schema_name": to_component_schema.name(),
            "to_socket_id": conn.destination.socket_id,
            "to_socket_name":  &to_socket.name(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
