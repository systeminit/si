use dal::{
    Component,
    ComponentId,
    InputSocket,
    OutputSocket,
    SchemaVariantId,
    WsEvent,
    change_status::ChangeStatus,
    diagram::SummaryDiagramEdge,
};
use serde::{
    Deserialize,
    Serialize,
};
use utoipa::ToSchema;

use super::{
    ComponentsError,
    ComponentsResult,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({"component": "ComponentName", "socketName": "InputSocketName"}))]
#[schema(example = json!({"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "OutputSocketName"}))]
pub struct ConnectionPoint {
    #[serde(flatten)]
    pub component_ref: ComponentReference,
    pub socket_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[schema(example = json!({"component": "ComponentName"}))]
#[schema(example = json!({"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY"}))]
pub enum ComponentReference {
    ByName {
        component: String,
    },
    #[serde(rename_all = "camelCase")]
    ById {
        #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
        component_id: ComponentId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Connection {
    #[schema(example = json!({"from": {"component": "OtherComponentName", "socketName": "output"}, "to": "ThisComponentInputSocketName"}))]
    #[schema(example = json!({"from": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "output"}, "to": "ThisComponentInputSocketName"}))]
    Incoming { from: ConnectionPoint, to: String },
    #[schema(example = json!({"from": "ThisComponentOutputSocketName", "to": {"component": "OtherComponentName", "socketName": "InputSocketName"}}))]
    #[schema(example = json!({"from": "ThisComponentOutputSocketName", "to": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "InputSocketName"}}))]
    Outgoing { from: String, to: ConnectionPoint },
}

/// Returns the component ID if found, or appropriate error if not found. If a duplicate exists, it
/// picks the "first" one it sees.
pub async fn find_component_id_by_name(
    ctx: &dal::DalContext,
    component_list: &[ComponentId],
    component_name: &str,
) -> Result<ComponentId, ComponentsError> {
    for component_id in component_list {
        let name = Component::name_by_id(ctx, *component_id).await?;
        if name == component_name {
            return Ok(*component_id);
        }
    }
    Err(ComponentsError::ComponentNotFound(
        component_name.to_string(),
    ))
}

/// Helper function to resolve a component reference to a component ID
pub async fn resolve_component_reference(
    ctx: &dal::DalContext,
    component_ref: &ComponentReference,
    component_list: &[ComponentId],
) -> Result<ComponentId, ComponentsError> {
    match component_ref {
        ComponentReference::ById { component_id } => Ok(*component_id),
        ComponentReference::ByName { component } => {
            find_component_id_by_name(ctx, component_list, component).await
        }
    }
}

/// Helper function to find an input socket ID by name for a given schema variant
pub async fn find_input_socket_id(
    ctx: &dal::DalContext,
    socket_name: &str,
    schema_variant_id: dal::SchemaVariantId,
) -> Result<si_id::InputSocketId, ComponentsError> {
    let input_socket =
        InputSocket::find_with_name_or_error(ctx, socket_name, schema_variant_id).await?;
    Ok(input_socket.id())
}

/// Helper function to find an output socket ID by name for a given schema variant
pub async fn find_output_socket_id(
    ctx: &dal::DalContext,
    socket_name: &str,
    schema_variant_id: dal::SchemaVariantId,
) -> Result<si_id::OutputSocketId, ComponentsError> {
    let output_socket =
        OutputSocket::find_with_name_or_error(ctx, socket_name, schema_variant_id).await?;
    Ok(output_socket.id())
}

/// Helper function to find an input socket ID by name for a specific component
pub async fn find_component_input_socket_id(
    ctx: &dal::DalContext,
    component_id: ComponentId,
    socket_name: &str,
) -> Result<si_id::InputSocketId, ComponentsError> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let schema_variant = component.schema_variant(ctx).await?;
    let variant_id = schema_variant.id();

    find_input_socket_id(ctx, socket_name, variant_id).await
}

pub async fn find_component_output_socket_id(
    ctx: &dal::DalContext,
    component_id: ComponentId,
    socket_name: &str,
) -> Result<si_id::OutputSocketId, ComponentsError> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let schema_variant = component.schema_variant(ctx).await?;
    let variant_id = schema_variant.id();

    find_output_socket_id(ctx, socket_name, variant_id).await
}

pub async fn handle_connection(
    ctx: &dal::DalContext,
    connection: &Connection,
    component_id: ComponentId,
    variant_id: SchemaVariantId,
    component_list: &[ComponentId],
    is_add: bool,
) -> ComponentsResult<()> {
    match connection {
        Connection::Incoming {
            from,
            to: to_socket_name,
        } => {
            let source_component_id =
                resolve_component_reference(ctx, &from.component_ref, component_list).await?;
            let target_socket_id = find_input_socket_id(ctx, to_socket_name, variant_id).await?;
            let source_socket_id =
                find_component_output_socket_id(ctx, source_component_id, &from.socket_name)
                    .await?;

            if is_add {
                Component::connect(
                    ctx,
                    source_component_id,
                    source_socket_id,
                    component_id,
                    target_socket_id,
                )
                .await?;
                let from_component = Component::get_by_id(ctx, source_component_id).await?;
                let to_component = Component::get_by_id(ctx, component_id).await?;
                for incoming_connection in to_component.incoming_connections(ctx).await? {
                    if incoming_connection.to_input_socket_id == target_socket_id
                        && incoming_connection.from_component_id == from_component.id()
                        && incoming_connection.to_component_id == to_component.id()
                    {
                        let edge = SummaryDiagramEdge::assemble(
                            incoming_connection,
                            &from_component,
                            &to_component,
                            ChangeStatus::Added,
                        )?;
                        WsEvent::connection_upserted(ctx, edge.into())
                            .await?
                            .publish_on_commit(ctx)
                            .await?;
                    }
                }
            } else {
                Component::remove_connection(
                    ctx,
                    source_component_id,
                    source_socket_id,
                    component_id,
                    target_socket_id,
                )
                .await?;
                WsEvent::connection_deleted(
                    ctx,
                    source_component_id,
                    component_id,
                    source_socket_id,
                    target_socket_id,
                )
                .await?
                .publish_on_commit(ctx)
                .await?;
            }
        }
        Connection::Outgoing {
            from: from_socket_name,
            to,
        } => {
            let target_component_id =
                resolve_component_reference(ctx, &to.component_ref, component_list).await?;
            let source_socket_id = find_output_socket_id(ctx, from_socket_name, variant_id).await?;
            let target_socket_id =
                find_component_input_socket_id(ctx, target_component_id, &to.socket_name).await?;

            if is_add {
                Component::connect(
                    ctx,
                    component_id,
                    source_socket_id,
                    target_component_id,
                    target_socket_id,
                )
                .await?;
                let from_component = Component::get_by_id(ctx, component_id).await?;
                let to_component = Component::get_by_id(ctx, target_component_id).await?;
                for incoming_connection in to_component.incoming_connections(ctx).await? {
                    if incoming_connection.to_input_socket_id == target_socket_id
                        && incoming_connection.from_component_id == from_component.id()
                        && incoming_connection.to_component_id == to_component.id()
                    {
                        let edge = SummaryDiagramEdge::assemble(
                            incoming_connection,
                            &from_component,
                            &to_component,
                            ChangeStatus::Added,
                        )?;
                        WsEvent::connection_upserted(ctx, edge.into())
                            .await?
                            .publish_on_commit(ctx)
                            .await?;
                    }
                }
            } else {
                Component::remove_connection(
                    ctx,
                    component_id,
                    source_socket_id,
                    target_component_id,
                    target_socket_id,
                )
                .await?;
                WsEvent::connection_deleted(
                    ctx,
                    component_id,
                    target_component_id,
                    source_socket_id,
                    target_socket_id,
                )
                .await?
                .publish_on_commit(ctx)
                .await?;
            }
        }
    }

    Ok(())
}
