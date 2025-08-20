use std::collections::{
    HashMap,
    HashSet,
};

use si_events::audit_log::AuditLogKind;
use si_id::ComponentId;

use super::{
    Component,
    ComponentResult,
};
use crate::{
    DalContext,
    WsEvent,
    change_status::ChangeStatus,
    diagram::SummaryDiagramEdge,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentDeletionStatus {
    /// The component has a resource but will be deleted by the destroy action
    /// after applying to head
    MarkedForDeletion,
    /// The component was deleted in this changeset but still exists on head
    StillExistsOnHead,
    /// The component was deleted (either because it was safe to delete without
    /// an action, or becuase it was force erased)
    Deleted,
}

pub async fn delete_components(
    ctx: &DalContext,
    component_ids: &[ComponentId],
    force_erase: bool,
) -> ComponentResult<HashMap<ComponentId, ComponentDeletionStatus>> {
    let head_components: HashSet<ComponentId> =
        Component::exists_on_head_by_ids(ctx, component_ids).await?;
    let mut result = HashMap::new();

    let mut socket_map = HashMap::new();
    let mut socket_map_head = HashMap::new();
    let base_change_set_ctx = ctx.clone_with_base().await?;

    for &component_id in component_ids {
        let component = Component::get_by_id(ctx, component_id).await?;

        let incoming_connections = component.incoming_connections(ctx).await?;
        let outgoing_connections = component.outgoing_connections(ctx).await?;

        let status = delete_component(ctx, &component, force_erase, &head_components).await?;

        process_delete(
            ctx,
            &mut socket_map,
            &mut socket_map_head,
            &base_change_set_ctx,
            component_id,
            component,
            incoming_connections,
            outgoing_connections,
            status,
        )
        .await?;

        result.insert(component_id, status);
    }

    Ok(result)
}

/// Deletes a component (either removing or marking as to be deleted), and sends all the necessary WSEvents
/// TEMPORARILY EMBRACE THE MADNESS
pub async fn delete_and_process(
    ctx: &DalContext,
    force_erase: bool,
    head_components: &HashSet<ComponentId>,
    socket_map: &mut HashMap<si_id::SchemaVariantId, Vec<si_frontend_types::DiagramSocket>>,
    socket_map_head: &mut HashMap<si_id::SchemaVariantId, Vec<si_frontend_types::DiagramSocket>>,
    base_change_set_ctx: &DalContext,
    component_id: ComponentId,
) -> ComponentResult<ComponentDeletionStatus> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let incoming_connections = component.incoming_connections(ctx).await?;
    let outgoing_connections = component.outgoing_connections(ctx).await?;
    let status = delete_component(ctx, &component, force_erase, head_components).await?;
    process_delete(
        ctx,
        socket_map,
        socket_map_head,
        base_change_set_ctx,
        component_id,
        component,
        incoming_connections,
        outgoing_connections,
        status,
    )
    .await?;
    Ok(status)
}

#[allow(clippy::too_many_arguments)]
async fn process_delete(
    ctx: &DalContext,
    socket_map: &mut HashMap<si_id::SchemaVariantId, Vec<si_frontend_types::DiagramSocket>>,
    socket_map_head: &mut HashMap<si_id::SchemaVariantId, Vec<si_frontend_types::DiagramSocket>>,
    base_change_set_ctx: &DalContext,
    component_id: ComponentId,
    component: Component,
    incoming_connections: Vec<super::Connection>,
    outgoing_connections: Vec<super::Connection>,
    status: ComponentDeletionStatus,
) -> ComponentResult<ComponentDeletionStatus> {
    for incoming_connection in incoming_connections {
        let payload = SummaryDiagramEdge {
            from_component_id: incoming_connection.from_component_id,
            from_socket_id: incoming_connection.from_output_socket_id,
            to_component_id: incoming_connection.to_component_id,
            to_socket_id: incoming_connection.to_input_socket_id,
            change_status: ChangeStatus::Deleted,
            created_info: serde_json::to_value(incoming_connection.created_info)?,
            deleted_info: serde_json::to_value(incoming_connection.deleted_info)?,
            to_delete: true,
            from_base_change_set: false,
        };
        WsEvent::connection_upserted(ctx, payload.into())
            .await?
            .publish_on_commit(ctx)
            .await?;
    }
    for outgoing_connection in outgoing_connections {
        let payload = SummaryDiagramEdge {
            from_component_id: outgoing_connection.from_component_id,
            from_socket_id: outgoing_connection.from_output_socket_id,
            to_component_id: outgoing_connection.to_component_id,
            to_socket_id: outgoing_connection.to_input_socket_id,
            change_status: ChangeStatus::Deleted,
            created_info: serde_json::to_value(outgoing_connection.created_info)?,
            deleted_info: serde_json::to_value(outgoing_connection.deleted_info)?,
            to_delete: true,
            from_base_change_set: false,
        };
        WsEvent::connection_upserted(ctx, payload.into())
            .await?
            .publish_on_commit(ctx)
            .await?;
    }
    match status {
        ComponentDeletionStatus::MarkedForDeletion => {
            let payload = component
                .into_frontend_type(ctx, None, ChangeStatus::Deleted, socket_map)
                .await?;
            WsEvent::component_updated(ctx, payload)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
        ComponentDeletionStatus::StillExistsOnHead => {
            let component: Component =
                Component::get_by_id(base_change_set_ctx, component_id).await?;
            let payload = component
                .into_frontend_type(
                    base_change_set_ctx,
                    None,
                    ChangeStatus::Deleted,
                    socket_map_head,
                )
                .await?;
            WsEvent::component_updated(ctx, payload)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
        ComponentDeletionStatus::Deleted => {
            WsEvent::component_deleted(ctx, component_id)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }
    }
    Ok(status)
}

pub async fn delete_component(
    ctx: &DalContext,
    component: &Component,
    force_erase: bool,
    head_components: &HashSet<ComponentId>,
) -> ComponentResult<ComponentDeletionStatus> {
    let component_id = component.id();
    let component_name = component.name(ctx).await?;
    let component_schema_variant = component.schema_variant(ctx).await?;
    let still_exists_on_head = head_components.contains(&component_id);

    let mut status = if force_erase {
        Component::remove(ctx, component_id).await?;

        ComponentDeletionStatus::Deleted
    } else {
        ctx.write_audit_log(
            AuditLogKind::DeleteComponent {
                component_id,
                name: component_name.to_owned(),
                schema_variant_id: component_schema_variant.id(),
                schema_variant_name: component_schema_variant.display_name().to_string(),
            },
            component_name,
        )
        .await?;

        // the move semantics here feel strange
        match component.clone().delete(ctx).await? {
            Some(_) => ComponentDeletionStatus::MarkedForDeletion,
            None => ComponentDeletionStatus::Deleted,
        }
    };

    if matches!(status, ComponentDeletionStatus::Deleted) && still_exists_on_head {
        status = ComponentDeletionStatus::StillExistsOnHead;
    }

    ctx.workspace_snapshot()?.cleanup().await?;

    Ok(status)
}
