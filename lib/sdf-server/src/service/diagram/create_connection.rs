use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    change_status::ChangeStatus,
    diagram::{SummaryDiagramEdge, SummaryDiagramInferredEdge},
    ChangeSet, Component, ComponentId, InputSocket, InputSocketId, OutputSocket, OutputSocketId,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    routes::AppError,
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

use super::DiagramError;

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
) -> Result<ForceChangeSetResponse<()>, AppError> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // Register the inferred edges that are going away before they become inaccessible
    {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut component_tree = workspace_snapshot.inferred_connection_graph(&ctx).await?;

        let inferred_edges = component_tree
            .inferred_incoming_connections_for_component(&ctx, request.to_component_id)
            .await?
            .iter()
            .filter_map(|connection| {
                if connection.input_socket_id != request.to_socket_id {
                    return None;
                }
                Some(SummaryDiagramInferredEdge {
                    from_component_id: connection.source_component_id,
                    from_socket_id: connection.output_socket_id,
                    to_component_id: connection.destination_component_id,
                    to_socket_id: connection.input_socket_id,
                    to_delete: false,
                })
            })
            .collect();

        WsEvent::remove_inferred_edges(&ctx, inferred_edges)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

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
    let to_component_name = to_component.name(&ctx).await?;
    let to_socket_name = InputSocket::get_by_id(&ctx, request.to_socket_id)
        .await?
        .name()
        .to_string();
    ctx.write_audit_log(
        AuditLogKind::CreateConnection {
            from_component_id: request.from_component_id,
            from_component_name: from_component.name(&ctx).await?,
            from_socket_id: request.from_socket_id,
            from_socket_name: OutputSocket::get_by_id(&ctx, request.from_socket_id)
                .await?
                .name()
                .to_string(),
            to_component_id: request.to_component_id,
            to_component_name: to_component_name.clone(),
            to_socket_id: request.to_socket_id,
            to_socket_name: to_socket_name.clone(),
        },
        format!("{to_component_name} --- {to_socket_name}"),
    )
    .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
