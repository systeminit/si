use std::collections::HashMap;
use telemetry::prelude::*;

use super::{ViewError, ViewResult};
use crate::{
    extract::{HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    service::v2::AccessBuilder,
    track,
};
use axum::extract::Path;
use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::diagram::view::ViewId;
use dal::{
    change_status::ChangeStatus, component::frame::Frame, diagram::SummaryDiagramEdge, ChangeSet,
    ChangeSetId, Component, ComponentId, WorkspacePk, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_frontend_types::StringGeometry;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PasteSingleComponentPayload {
    id: ComponentId,
    component_geometry: StringGeometry,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PasteComponentsRequest {
    pub components: Vec<PasteSingleComponentPayload>,
    pub new_parent_node_id: Option<ComponentId>,
}

/// Paste a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn paste_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<PasteComponentsRequest>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut pasted_components_by_original = HashMap::new();
    for component_payload in &request.components {
        let component_id = component_payload.id;

        let original_comp = Component::get_by_id(&ctx, component_id).await?;
        let pasted_comp = original_comp
            .create_copy(
                &ctx,
                view_id,
                component_payload.component_geometry.clone().try_into()?,
            )
            .await?;

        let schema = pasted_comp.schema(&ctx).await?;
        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "paste_component",
            serde_json::json!({
                "how": "/v2/view/paste_component",
                "component_id": pasted_comp.id(),
                "component_schema_name": schema.name(),
            }),
        );

        pasted_components_by_original.insert(component_id, pasted_comp);
    }

    for component_payload in &request.components {
        let component_id = component_payload.id;

        let pasted_component = pasted_components_by_original
            .get(&component_id)
            .ok_or(ViewError::Paste)?;
        let original_component = Component::get_by_id(&ctx, component_id).await?;

        // If component parent was also pasted on this batch, keep relationship between new components

        if let Some(pasted_parent) = original_component
            .parent(&ctx)
            .await?
            .and_then(|parent_id| pasted_components_by_original.get(&parent_id))
        {
            Frame::upsert_parent(&ctx, pasted_component.id(), pasted_parent.id()).await?;
        }

        // If the pasted component didn't get a parent already, set the new parent
        if pasted_component.parent(&ctx).await?.is_none() {
            if let Some(parent_id) = request.new_parent_node_id {
                Frame::upsert_parent(&ctx, pasted_component.id(), parent_id).await?;
            }
        }

        // re-fetch component with possible parentage
        let pasted_component = Component::get_by_id(&ctx, pasted_component.id()).await?;
        let mut diagram_sockets = HashMap::new();
        let geo = pasted_component.geometry(&ctx, view_id).await?;
        let payload = pasted_component
            .into_frontend_type(&ctx, Some(&geo), ChangeStatus::Added, &mut diagram_sockets)
            .await?;
        WsEvent::component_created(&ctx, payload)
            .await?
            .publish_on_commit(&ctx)
            .await?;

        for original_manager_id in original_component.managers(&ctx).await? {
            let Some(pasted_manager) = pasted_components_by_original.get(&original_manager_id)
            else {
                continue;
            };

            match Component::manage_component(&ctx, pasted_manager.id(), pasted_component.id())
                .await
            {
                Ok(edge) => {
                    WsEvent::connection_upserted(&ctx, edge.into())
                        .await?
                        .publish_on_commit(&ctx)
                        .await?;
                }
                Err(dal::ComponentError::ComponentNotManagedSchema(_, _, _)) => {
                    // This error should not occur, but we also don't want to
                    // fail the paste just because the managed schemas are out
                    // of sync
                    error!("Could not manage pasted component, but continuing paste");
                }
                Err(err) => {
                    return Err(err)?;
                }
            };
        }

        // Create on pasted components copies of edges that existed between original components
        for connection in original_component.incoming_connections(&ctx).await? {
            if let Some(from_component) =
                pasted_components_by_original.get(&connection.from_component_id)
            {
                Component::connect(
                    &ctx,
                    from_component.id(),
                    connection.from_output_socket_id,
                    pasted_component.id(),
                    connection.to_input_socket_id,
                )
                .await?;

                let edge = SummaryDiagramEdge {
                    from_component_id: from_component.id(),
                    from_socket_id: connection.from_output_socket_id,
                    to_component_id: pasted_component.id(),
                    to_socket_id: connection.to_input_socket_id,
                    change_status: ChangeStatus::Added,
                    created_info: serde_json::to_value(connection.created_info)?,
                    deleted_info: serde_json::to_value(connection.deleted_info)?,
                    to_delete: false,
                    from_base_change_set: false,
                };
                WsEvent::connection_upserted(&ctx, edge.into())
                    .await?
                    .publish_on_commit(&ctx)
                    .await?;
            }
        }
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
