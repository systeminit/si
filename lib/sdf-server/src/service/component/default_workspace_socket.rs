use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    change_status::ChangeStatus, component::socket::DefaultConnection, diagram::SummaryDiagramEdge,
    AttributeValueId, ChangeSet, Component, ComponentId, InputSocket, InputSocketId, OutputSocket,
    OutputSocketId, Ulid, Visibility, WsEvent,
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
enum SocketId {
    InputSocket(InputSocketId),
    OutputSocket(OutputSocketId),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DefaultWorkspaceSocketRequest {
    pub component_id: ComponentId,
    pub socket_id: AttributeValueId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoconnectComponentResponse {
    pub is_default: bool,
}

pub async fn default_workspace_socket(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    PosthogClient(_posthog_client): PosthogClient,
    Json(DefaultWorkspaceSocketRequest {
        component_id,
        socket_id,
        visibility,
    }): Json<DefaultWorkspaceSocketRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let socket_id: Ulid = socket_id.into();
    // first get the workspace defaults
    let default_connections =
        DefaultConnection::get_default_connections_for_workspace(&ctx).await?;
    // see if there's one for this component/socket. If there is, remove it. if not, add it
    let maybe: Vec<DefaultConnection> = default_connections
        .iter()
        .filter_map(|conn| match conn {
            DefaultConnection::Workspace(component_socket) => {
                if component_socket.get_component_id() == component_id
                    && component_socket.get_socket_id() == socket_id
                {
                    Some(*conn)
                } else {
                    None
                }
            }
            DefaultConnection::View(_) | DefaultConnection::Frame(_) => None,
        })
        .collect();

    let mut is_workspace_default = false;

    if let Some(attribute_value_id) = if let Some(attribute_value) =
        OutputSocket::component_attribute_value_for_output_socket_id_opt(
            &ctx,
            socket_id.into(),
            component_id,
        )
        .await?
    {
        Some(attribute_value)
    } else if let Ok(attribute_value) = InputSocket::component_attribute_value_for_input_socket_id(
        &ctx,
        socket_id.into(),
        component_id,
    )
    .await
    {
        Some(attribute_value)
    } else {
        None
    } {
        if maybe.is_empty() {
            DefaultConnection::set_default_connection_for_workspace(&ctx, attribute_value_id)
                .await?;
            is_workspace_default = true;
        } else {
            DefaultConnection::remove_for_workspace(&ctx, attribute_value_id).await?;
        }
    };

    // // just look at all this paperwork!
    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     &host_name,
    //     "auto_connect",
    //     serde_json::json!({
    //         "component_id": component_id,
    //         "change_set_id": ctx.change_set_id(),
    //     }),
    // );

    ctx.commit().await?;

    // let the front end know if nothing was created so we can tell the user
    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
