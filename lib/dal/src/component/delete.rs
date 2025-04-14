use std::collections::{HashMap, HashSet};

use si_events::audit_log::AuditLogKind;
use si_id::ComponentId;

use crate::{change_status::ChangeStatus, diagram::SummaryDiagramEdge, DalContext, WsEvent};

use super::{Component, ComponentResult};

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
        Component::exists_on_head(ctx, component_ids).await?;
    let mut result = HashMap::new();

    let mut socket_map = HashMap::new();
    let mut socket_map_head = HashMap::new();
    let base_change_set_ctx = ctx.clone_with_base().await?;

    for &component_id in component_ids {
        let component = Component::get_by_id(ctx, component_id).await?;

        let incoming_connections = component.incoming_connections(ctx).await?;
        let outgoing_connections = component.outgoing_connections(ctx).await?;

        dbg!("deleting");
        let status = delete_component(ctx, &component, force_erase, &head_components).await?;
        dbg!("deleted");

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

        dbg!("0");

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

        dbg!("2");

        match status {
            ComponentDeletionStatus::MarkedForDeletion => {
                dbg!("3");
                let payload = component
                    .into_frontend_type(ctx, None, ChangeStatus::Deleted, &mut socket_map)
                    .await?;
                WsEvent::component_updated(ctx, payload)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            }
            ComponentDeletionStatus::StillExistsOnHead => {
                dbg!("4");
                let component: Component =
                    Component::get_by_id(&base_change_set_ctx, component_id).await?;
                dbg!("5");
                let payload = component
                    .into_frontend_type(
                        &base_change_set_ctx,
                        None,
                        ChangeStatus::Deleted,
                        &mut socket_map_head,
                    )
                    .await?;
                dbg!("6");
                WsEvent::component_updated(ctx, payload)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            }
            ComponentDeletionStatus::Deleted => {
                dbg!("7");
                WsEvent::component_deleted(ctx, component_id)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            }
        }

        result.insert(component_id, status);
    }

    Ok(result)
}

async fn delete_component(
    ctx: &DalContext,
    component: &Component,
    force_erase: bool,
    head_components: &HashSet<ComponentId>,
) -> ComponentResult<ComponentDeletionStatus> {
    dbg!("delete_component");
    let component_id = component.id();
    dbg!("name");
    let component_name = component.name(ctx).await?;
    dbg!("asking for sv");
    let component_schema_variant = component.schema_variant(ctx).await?;
    dbg!("got sv");

    let still_exists_on_head = head_components.contains(&component_id);

    let mut status = if force_erase {
        dbg!("remove");
        Component::remove(ctx, component_id).await?;
        ComponentDeletionStatus::Deleted
    } else {
        dbg!("mark for deletion");
        // the move semantics here feel strange
        match dbg!(component.clone().delete(ctx).await)? {
            Some(_) => ComponentDeletionStatus::MarkedForDeletion,
            None => ComponentDeletionStatus::Deleted,
        }
    };

    if matches!(status, ComponentDeletionStatus::Deleted) && still_exists_on_head {
        status = ComponentDeletionStatus::StillExistsOnHead;
    }

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

    dbg!("audit log written");

    ctx.workspace_snapshot()?.cleanup().await?;
    dbg!("deleted and cleaned up");

    Ok(status)
}
