use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    change_status::ChangeStatus, diagram::SummaryDiagramEdge, ChangeSet, Component, ComponentId,
    InputSocket, OutputSocket, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

use super::ComponentResult;
use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoconnectComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoconnectComponentResponse {
    pub connections_created: usize,
}

pub async fn autoconnect(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    Json(AutoconnectComponentRequest {
        component_id,
        visibility,
    }): Json<AutoconnectComponentRequest>,
) -> ComponentResult<ForceChangeSetResponse<AutoconnectComponentResponse>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let input_sockets_connected = Component::autoconnect(&ctx, component_id).await?;

    // just look at all this paperwork!
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "auto_connect",
        serde_json::json!({
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    let component = Component::get_by_id(&ctx, component_id).await?;
    for incoming_connection in component.incoming_connections(&ctx).await? {
        let input_socket_id = incoming_connection.to_input_socket_id;
        if input_sockets_connected.contains(&input_socket_id) {
            let from_component_id = incoming_connection.from_component_id;
            let from_socket_id = incoming_connection.from_output_socket_id;
            let from_component = Component::get_by_id(&ctx, from_component_id).await?;
            let edge = SummaryDiagramEdge::assemble(
                incoming_connection,
                &from_component,
                &component,
                ChangeStatus::Added,
            )?;
            WsEvent::connection_upserted(&ctx, edge.into())
                .await?
                .publish_on_commit(&ctx)
                .await?;
            let to_component_name = component.name(&ctx).await?;
            let to_socket_name = InputSocket::get_by_id(&ctx, input_socket_id)
                .await?
                .name()
                .to_string();
            ctx.write_audit_log(
                AuditLogKind::CreateConnection {
                    from_component_id,
                    from_component_name: from_component.name(&ctx).await?,
                    from_socket_id,
                    from_socket_name: OutputSocket::get_by_id(&ctx, from_socket_id)
                        .await?
                        .name()
                        .to_string(),
                    to_component_id: component_id,
                    to_component_name: to_component_name.clone(),
                    to_socket_id: input_socket_id,
                    to_socket_name: to_socket_name.clone(),
                },
                format!("{to_component_name} --- {to_socket_name}"),
            )
            .await?;
        }
    }
    ctx.commit().await?;

    // let the front end know if nothing was created so we can tell the user
    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        AutoconnectComponentResponse {
            connections_created: input_sockets_connected.len(),
        },
    ))
}
