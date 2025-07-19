use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    ChangeSet,
    Component,
    ComponentId,
    InputSocket,
    InputSocketId,
    OutputSocket,
    OutputSocketId,
    WsEvent,
    change_status::ChangeStatus,
    diagram::{
        SummaryDiagramEdge,
        SummaryDiagramInferredEdge,
    },
};
use sdf_core::{
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;
use si_events::audit_log::AuditLogKind;

use super::DiagramResult;

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
    Host(host_name): Host,
    Json(request): Json<DeleteConnectionRequest>,
) -> DiagramResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    Component::remove_connection(
        &ctx,
        request.from_component_id,
        request.from_socket_id,
        request.to_component_id,
        request.to_socket_id,
    )
    .await?;

    let from_component = Component::get_by_id(&ctx, request.from_component_id).await?;

    let to_component = Component::get_by_id(&ctx, request.to_component_id).await?;

    let output_socket = OutputSocket::get_by_id(&ctx, request.from_socket_id).await?;
    let input_socket = InputSocket::get_by_id(&ctx, request.to_socket_id).await?;

    let base_change_set_ctx = ctx.clone_with_base().await?;

    let base_from_component =
        Component::try_get_by_id(&base_change_set_ctx, request.from_component_id).await?;
    let base_to_component =
        Component::try_get_by_id(&base_change_set_ctx, request.to_component_id).await?;

    let mut payload: Option<SummaryDiagramEdge> = None;
    if let Some((base_from, base_to)) = base_from_component.zip(base_to_component) {
        let incoming_edges = base_to
            .incoming_connections(&base_change_set_ctx)
            .await
            .ok();
        if let Some(edges) = incoming_edges {
            for incoming in edges {
                if incoming.from_output_socket_id == request.from_socket_id
                    && incoming.from_component_id == base_from.id()
                    && incoming.to_input_socket_id == request.to_socket_id
                {
                    payload = Some(SummaryDiagramEdge::assemble(
                        incoming,
                        &from_component,
                        &to_component,
                        ChangeStatus::Deleted,
                    )?);
                }
            }
        }
    }

    if let Some(edge) = payload {
        WsEvent::connection_upserted(&ctx, edge.into())
            .await?
            .publish_on_commit(&ctx)
            .await?;
    } else {
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
    }

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

    WsEvent::upsert_inferred_edges(&ctx, inferred_edges)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let from_component_schema =
        Component::schema_for_component_id(&ctx, request.from_component_id).await?;

    let to_component_schema =
        Component::schema_for_component_id(&ctx, request.to_component_id).await?;
    let to_component_name = to_component.name(&ctx).await?;
    let to_socket_name = InputSocket::get_by_id(&ctx, request.to_socket_id)
        .await?
        .name()
        .to_string();
    ctx.write_audit_log(
        AuditLogKind::DeleteConnection {
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
        format!("{to_component_name}-{to_socket_name}"),
    )
    .await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
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

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
