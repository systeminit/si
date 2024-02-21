use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::attribute::prototype::argument::AttributePrototypeArgumentId;
use dal::{ChangeSet, Component, ComponentId, InputSocketId, OutputSocketId, User, Visibility};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub from_component_id: ComponentId,
    pub from_socket_id: OutputSocketId,
    pub to_component_id: ComponentId,
    pub to_socket_id: InputSocketId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionResponse {
    pub id: AttributePrototypeArgumentId,
    pub created_by: Option<User>,
    pub deleted_by: Option<User>,
}

pub async fn create_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Json(request): Json<CreateConnectionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    let attribute_prototype_argument_id = Component::connect(
        &ctx,
        request.from_component_id,
        request.from_socket_id,
        request.to_component_id,
        request.to_socket_id,
    )
    .await?;

    // TODO(nick): restore posthog, but with new, relevant fields.
    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "connection_created",
    //     serde_json::json!({
    //                 "from_node_id": request.from_node_id,
    //                 "from_node_schema_name": &from_component_schema.name(),
    //                 "from_socket_id": request.from_socket_id,
    //                 "from_socket_name": &from_socket.name(),
    //                 "to_node_id": request.to_node_id,
    //                 "to_node_schema_name": &to_component_schema.name(),
    //                 "to_socket_id": request.to_socket_id,
    //                 "to_socket_name":  &to_socket.name(),
    //     }),
    // );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response
        .header("content-type", "application/json")
        .body(serde_json::to_string(&CreateConnectionResponse {
            id: attribute_prototype_argument_id,
            // TODO(nick): figure out what to do with these fields that were left over from the "Connection" struct.
            created_by: None,
            deleted_by: None,
        })?)?)
}
