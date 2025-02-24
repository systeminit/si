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
use si_id::ViewId;

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
pub struct DefaultViewSocketRequest {
    pub component_id: ComponentId,
    pub socket_id: AttributeValueId,
    pub view_id: ViewId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AutoconnectComponentResponse {
    pub is_default: bool,
}

pub async fn default_view_socket(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    PosthogClient(_posthog_client): PosthogClient,
    Json(DefaultViewSocketRequest {
        component_id,
        socket_id,
        view_id,
        visibility,
    }): Json<DefaultViewSocketRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let socket_id: Ulid = socket_id.into();
    // first get the workspace defaults
    let default_connections =
        DefaultConnection::get_default_connections_for_view(&ctx, view_id).await?;
    // see if there's one for this component/socket. If there is, remove it. if not, add it
    let maybe: Vec<DefaultConnection> = default_connections
        .iter()
        .filter_map(|conn| match conn {
            DefaultConnection::View(component_socket) => {
                if component_socket.get_component_id() == component_id
                    && component_socket.get_socket_id() == socket_id
                {
                    Some(*conn)
                } else {
                    None
                }
            }
            DefaultConnection::Workspace(_) | DefaultConnection::Frame(_) => None,
        })
        .collect();

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
            DefaultConnection::set_default_connection_for_view(&ctx, attribute_value_id, view_id)
                .await?;
        } else {
            DefaultConnection::remove_for_view(&ctx, attribute_value_id, view_id).await?;
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

    // let component = Component::get_by_id(&ctx, component_id).await?;

    ctx.commit().await?;

    // let the front end know if nothing was created so we can tell the user
    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
