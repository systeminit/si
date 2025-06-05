use std::collections::HashMap;

use axum::{
    Json,
    extract::Path,
};
use dal::{
    ChangeSet,
    Component,
    ComponentId,
    WsEvent,
    change_status::ChangeStatus,
    diagram::{
        SummaryDiagramEdge,
        SummaryDiagramInferredEdge,
        SummaryDiagramManagementEdge,
    },
};
use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_types::StringGeometry;

use super::{
    ViewParam,
    ViewResult,
};
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::force_change_set_response::ForceChangeSetResponse,
};

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

/// Duplicate a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn duplicate_components(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ViewParam { view_id }): Path<ViewParam>,
    Json(request): Json<PasteComponentsRequest>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let pasted_component_ids = Component::batch_copy(
        ctx,
        view_id,
        request.new_parent_node_id,
        request
            .components
            .into_iter()
            .map(|p| p.component_geometry.try_into().map(|geo| (p.id, geo)))
            .try_collect()?,
    )
    .await?;

    // Emit WsEvents and posthog events
    for pasted_component_id in pasted_component_ids {
        // posthog paste component event
        let schema = Component::schema_for_component_id(ctx, pasted_component_id).await?;
        tracker.track(
            ctx,
            "paste_component",
            serde_json::json!({
                "how": "/v2/view/paste_component",
                "component_id": pasted_component_id,
                "component_schema_name": schema.name(),
            }),
        );

        // Component created event
        {
            let pasted = Component::get_by_id(ctx, pasted_component_id).await?;
            let mut diagram_sockets = HashMap::new();
            let geo = pasted.geometry(ctx, view_id).await?;
            let payload = pasted
                .into_frontend_type(ctx, Some(&geo), ChangeStatus::Added, &mut diagram_sockets)
                .await?;

            // Inferred connections (parent-child)
            let inferred_connections =
                Component::inferred_incoming_connections(ctx, pasted_component_id).await?;
            let inferred_edges = if !inferred_connections.is_empty() {
                Some(
                    inferred_connections
                        .into_iter()
                        .map(SummaryDiagramInferredEdge::assemble)
                        .collect(),
                )
            } else {
                None
            };

            WsEvent::component_created_with_inferred_edges(ctx, payload, inferred_edges)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }

        // Manager connections
        for manager_component_id in Component::managers_by_id(ctx, pasted_component_id).await? {
            let manager_schema =
                Component::schema_for_component_id(ctx, manager_component_id).await?;
            let pasted_schema =
                Component::schema_for_component_id(ctx, pasted_component_id).await?;
            let edge = SummaryDiagramManagementEdge::new(
                manager_schema.id(),
                pasted_schema.id(),
                manager_component_id,
                pasted_component_id,
            );
            WsEvent::connection_upserted(ctx, edge.into())
                .await?
                .publish_on_commit(ctx)
                .await?;
        }

        // Incoming socket connections
        for connection in Component::incoming_connections_for_id(ctx, pasted_component_id).await? {
            let edge = SummaryDiagramEdge {
                from_component_id: connection.from_component_id,
                from_socket_id: connection.from_output_socket_id,
                to_component_id: pasted_component_id,
                to_socket_id: connection.to_input_socket_id,
                change_status: ChangeStatus::Added,
                created_info: serde_json::to_value(connection.created_info)?,
                deleted_info: serde_json::to_value(connection.deleted_info)?,
                to_delete: false,
                from_base_change_set: false,
            };
            WsEvent::connection_upserted(ctx, edge.into())
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
