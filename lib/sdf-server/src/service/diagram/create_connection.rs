use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    change_status::ChangeStatus, diagram::SummaryDiagramEdge, ChangeSet, Component, ComponentId,
    InputSocketId, OutputSocketId, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

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

pub async fn create_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CreateConnectionRequest>,
) -> DiagramResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    Component::connect(
        &ctx,
        request.from_component_id,
        request.from_socket_id,
        request.to_component_id,
        request.to_socket_id,
    )
    .await?
    .ok_or(DiagramError::DuplicatedConnection)?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_connection",
        serde_json::json!({
            "how": "/diagram/create_connection",
            "from_component_id": request.from_component_id,
            "from_socket_id": request.from_socket_id,
            "to_component_id": request.to_component_id,
            "to_socket_id": request.to_socket_id,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    let from_component = Component::get_by_id(&ctx, request.from_component_id).await?;
    let to_component = Component::get_by_id(&ctx, request.to_component_id).await?;
    for incoming_connection in to_component.incoming_connections(&ctx).await? {
        if incoming_connection.to_input_socket_id == request.to_socket_id
            && incoming_connection.from_component_id == from_component.id()
            && incoming_connection.to_component_id == to_component.id()
        {
            let edge = SummaryDiagramEdge::assemble(
                incoming_connection,
                &from_component,
                &to_component,
                ChangeStatus::Added,
            )?;
            WsEvent::connection_upserted(&ctx, edge.into())
                .await?
                .publish_on_commit(&ctx)
                .await?;
        }
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
