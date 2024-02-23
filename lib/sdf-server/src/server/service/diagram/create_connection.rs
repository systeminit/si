use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::edge::EdgeKind;
use dal::{
    job::definition::DependentValuesUpdate, socket::SocketId, AttributeReadContext, AttributeValue,
    ChangeSet, Component, ComponentId, Connection, InternalProvider, Socket, StandardModel,
    Visibility,
};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub from_component_id: ComponentId,
    pub from_socket_id: SocketId,
    pub to_component_id: ComponentId,
    pub to_socket_id: SocketId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionResponse {
    pub connection: Connection,
}

/// Create a [`Connection`] with a _to_ [`Socket`] and
/// [`Component`] and a _from_ [`Socket`] and [`Component`].
/// Creating change set if on head
pub async fn create_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateConnectionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    let connection = Connection::new(
        &ctx,
        request.from_component_id,
        request.from_socket_id,
        request.to_component_id,
        request.to_socket_id,
        EdgeKind::Configuration,
    )
    .await?;

    let from_component = Component::get_by_id(&ctx, &request.from_component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let from_component_schema = from_component
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let from_socket = Socket::get_by_id(&ctx, &request.from_socket_id)
        .await?
        .ok_or(DiagramError::SocketNotFound)?;

    let to_component = Component::get_by_id(&ctx, &request.to_component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let to_component_schema = to_component
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let to_socket = Socket::get_by_id(&ctx, &request.to_socket_id)
        .await?
        .ok_or(DiagramError::SocketNotFound)?;

    let to_socket_internal_provider =
        InternalProvider::find_explicit_for_socket(&ctx, request.to_socket_id)
            .await?
            .ok_or(DiagramError::InternalProviderNotFoundForSocket(
                request.to_socket_id,
            ))?;

    let to_attribute_value_context = AttributeReadContext {
        internal_provider_id: Some(*to_socket_internal_provider.id()),
        component_id: Some(*to_component.id()),
        ..Default::default()
    };
    let mut to_attribute_value = AttributeValue::find_for_context(&ctx, to_attribute_value_context)
        .await?
        .ok_or(DiagramError::AttributeValueNotFoundForContext(
            to_attribute_value_context,
        ))?;

    to_attribute_value
        .update_from_prototype_function(&ctx)
        .await?;

    ctx.enqueue_job(DependentValuesUpdate::new(
        ctx.access_builder(),
        *ctx.visibility(),
        vec![*to_attribute_value.id()],
    ))
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "connection_created",
        serde_json::json!({
                    "from_component_id": request.from_component_id,
                    "from_component_schema_name": &from_component_schema.name(),
                    "from_socket_id": request.from_socket_id,
                    "from_socket_name": &from_socket.name(),
                    "to_component_id": request.to_component_id,
                    "to_component_schema_name": &to_component_schema.name(),
                    "to_socket_id": request.to_socket_id,
                    "to_socket_name":  &to_socket.name(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response
        .header("content-type", "application/json")
        .body(serde_json::to_string(&CreateConnectionResponse {
            connection,
        })?)?)
}
