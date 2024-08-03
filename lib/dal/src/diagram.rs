use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::collections::{hash_map, HashMap, HashSet};
use std::num::{ParseFloatError, ParseIntError};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::actor_view::ActorView;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::value::AttributeValueError;
use crate::change_status::ChangeStatus;
use crate::component::{ComponentError, IncomingConnection, InferredIncomingConnection};
use crate::history_event::HistoryEventMetadata;
use crate::schema::variant::SchemaVariantError;
use crate::socket::connection_annotation::ConnectionAnnotation;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    AttributePrototypeId, ChangeSetError, Component, ComponentId, DalContext, HistoryEventError,
    InputSocketId, OutputSocketId, SchemaId, SchemaVariant, SchemaVariantId, SocketArity,
    StandardModelError, TransactionsError, Workspace, WorkspaceError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype not found")]
    AttributePrototypeNotFound,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found")]
    AttributeValueNotFound,
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("component status not found for component: {0}")]
    ComponentStatusNotFound(ComponentId),
    #[error("deletion timestamp not found")]
    DeletionTimeStamp,
    #[error("destination attribute prototype not found for inter component attribute prototype argument: {0}")]
    DestinationAttributePrototypeNotFound(AttributePrototypeArgumentId),
    #[error("destination input socket not found for attribute prototype ({0}) and inter component attribute prototype argument ({1})")]
    DestinationInputSocketNotFound(AttributePrototypeId, AttributePrototypeArgumentId),
    #[error("edge not found")]
    EdgeNotFound,
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("node not found")]
    NodeNotFound,
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("position not found")]
    PositionNotFound,
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("socket not found")]
    SocketNotFound,
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("Workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type NodeId = ComponentId;
pub type EdgeId = AttributePrototypeArgumentId;

pub type DiagramResult<T> = Result<T, DiagramError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint {
    pub x: isize,
    pub y: isize,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Size2D {
    pub width: isize,
    pub height: isize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramComponent {
    pub id: ComponentId,
    pub component_id: ComponentId,
    pub schema_name: String,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub schema_variant_name: String,
    pub schema_category: String,
    pub sockets: serde_json::Value,
    pub display_name: String,
    pub position: GridPoint,
    pub size: Size2D,
    pub color: String,
    pub component_type: String,
    pub change_status: ChangeStatus,
    pub has_resource: bool,
    pub parent_id: Option<ComponentId>,
    pub created_info: serde_json::Value,
    pub updated_info: serde_json::Value,
    pub deleted_info: serde_json::Value,
    pub to_delete: bool,
    pub can_be_upgraded: bool,
    pub from_base_change_set: bool,
}

impl SummaryDiagramComponent {
    pub async fn assemble(
        ctx: &DalContext,
        component: &Component,
        change_status: ChangeStatus,
    ) -> DiagramResult<Self> {
        let mut diagram_sockets: HashMap<SchemaVariantId, serde_json::Value> = HashMap::new();
        let schema_variant = component.schema_variant(ctx).await?;

        let sockets = match diagram_sockets.entry(schema_variant.id()) {
            hash_map::Entry::Vacant(entry) => {
                let (output_sockets, input_sockets) =
                    SchemaVariant::list_all_sockets(ctx, schema_variant.id()).await?;

                let mut sockets = vec![];

                for socket in input_sockets {
                    sockets.push(DiagramSocket {
                        id: socket.id().to_string(),
                        label: socket.name().to_string(),
                        connection_annotations: socket.connection_annotations(),
                        direction: DiagramSocketDirection::Input,
                        max_connections: match socket.arity() {
                            SocketArity::Many => None,
                            SocketArity::One => Some(1),
                        },
                        is_required: Some(false),
                        node_side: DiagramSocketNodeSide::Left,
                    });
                }

                for socket in output_sockets {
                    sockets.push(DiagramSocket {
                        id: socket.id().to_string(),
                        label: socket.name().to_string(),
                        connection_annotations: socket.connection_annotations(),
                        direction: DiagramSocketDirection::Output,
                        max_connections: match socket.arity() {
                            SocketArity::Many => None,
                            SocketArity::One => Some(1),
                        },
                        is_required: Some(false),
                        node_side: DiagramSocketNodeSide::Right,
                    });
                }

                let socket_value = serde_json::to_value(sockets)?;

                entry.insert(socket_value.to_owned());

                socket_value
            }
            hash_map::Entry::Occupied(entry) => entry.get().to_owned(),
        };
        let schema = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant.id()).await?;

        let default_schema_variant =
            SchemaVariant::get_default_for_schema(ctx, schema.id()).await?;

        let position = GridPoint {
            x: component.x().parse::<f64>()?.round() as isize,
            y: component.y().parse::<f64>()?.round() as isize,
        };
        let size = match (component.width(), component.height()) {
            (Some(w), Some(h)) => Size2D {
                height: h.parse()?,
                width: w.parse()?,
            },
            _ => Size2D {
                height: 500,
                width: 500,
            },
        };

        let updated_info = {
            let history_actor = ctx.history_actor();
            let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
            serde_json::to_value(HistoryEventMetadata {
                actor,
                timestamp: component.timestamp().updated_at,
            })?
        };

        let created_info = {
            let history_actor = ctx.history_actor();
            let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
            serde_json::to_value(HistoryEventMetadata {
                actor,
                timestamp: component.timestamp().created_at,
            })?
        };

        let can_be_upgraded = if let Some(unlocked_schema_variant) =
            SchemaVariant::get_unlocked_for_schema(ctx, schema.id()).await?
        {
            unlocked_schema_variant.id() != schema_variant.id()
        } else {
            default_schema_variant.id() != schema_variant.id()
        };

        Ok(SummaryDiagramComponent {
            id: component.id(),
            component_id: component.id(),
            schema_name: schema.name().to_owned(),
            schema_id: schema.id(),
            schema_variant_id: schema_variant.id(),
            schema_variant_name: schema_variant.version().to_owned(),
            schema_category: schema_variant.category().to_owned(),
            display_name: component.name(ctx).await?,
            position,
            size,
            component_type: component.get_type(ctx).await?.to_string(),
            color: component.color(ctx).await?.unwrap_or("#111111".into()),
            change_status,
            has_resource: component.resource(ctx).await?.is_some(),
            sockets,
            parent_id: component.parent(ctx).await?,
            updated_info,
            created_info,
            deleted_info: serde_json::Value::Null,
            to_delete: component.to_delete(),
            can_be_upgraded,
            from_base_change_set: false,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramEdge {
    pub from_component_id: ComponentId,
    pub from_socket_id: OutputSocketId,
    pub to_component_id: ComponentId,
    pub to_socket_id: InputSocketId,
    pub change_status: ChangeStatus,
    pub created_info: serde_json::Value,
    pub deleted_info: serde_json::Value,
    pub to_delete: bool,
    pub from_base_change_set: bool,
}

impl SummaryDiagramEdge {
    pub fn assemble(
        incoming_connection: IncomingConnection,
        from_component: Component,
        to_component: &Component,
        change_status: ChangeStatus,
    ) -> DiagramResult<Self> {
        Ok(SummaryDiagramEdge {
            from_component_id: incoming_connection.from_component_id,
            from_socket_id: incoming_connection.from_output_socket_id,
            to_component_id: incoming_connection.to_component_id,
            to_socket_id: incoming_connection.to_input_socket_id,
            change_status,
            created_info: serde_json::to_value(incoming_connection.created_info)?,
            deleted_info: serde_json::to_value(incoming_connection.deleted_info)?,
            to_delete: from_component.to_delete() || to_component.to_delete(),
            from_base_change_set: false,
        })
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramInferredEdge {
    pub from_component_id: ComponentId,
    pub from_socket_id: OutputSocketId,
    pub to_component_id: ComponentId,
    pub to_socket_id: InputSocketId,
    // this is inferred by if either the to or from component is marked to_delete
    pub to_delete: bool,
}

impl SummaryDiagramInferredEdge {
    pub fn assemble(
        inferred_incoming_connection: InferredIncomingConnection,
    ) -> DiagramResult<Self> {
        Ok(SummaryDiagramInferredEdge {
            from_component_id: inferred_incoming_connection.from_component_id,
            from_socket_id: inferred_incoming_connection.from_output_socket_id,
            to_component_id: inferred_incoming_connection.to_component_id,
            to_socket_id: inferred_incoming_connection.to_input_socket_id,
            to_delete: inferred_incoming_connection.to_delete,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramSocket {
    pub id: String,
    pub label: String,
    pub connection_annotations: Vec<ConnectionAnnotation>,
    pub direction: DiagramSocketDirection,
    pub max_connections: Option<usize>,
    pub is_required: Option<bool>,
    pub node_side: DiagramSocketNodeSide,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum DiagramSocketDirection {
    Bidirectional,
    Input,
    Output,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum DiagramSocketNodeSide {
    Left,
    Right,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagram {
    pub components: Vec<SummaryDiagramComponent>,
    pub edges: Vec<SummaryDiagramEdge>,
    pub inferred_edges: Vec<SummaryDiagramInferredEdge>,
}

impl Diagram {
    /// Assemble a [`Diagram`](Self) based on existing [`Nodes`](crate::Node) and
    /// [`Connections`](crate::Connection).
    #[instrument(level = "debug", skip(ctx))]
    pub async fn assemble(ctx: &DalContext) -> DiagramResult<Self> {
        let mut diagram_edges: Vec<SummaryDiagramEdge> = vec![];
        let mut diagram_inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];

        let components = Component::list(ctx).await?;

        let mut virtual_and_real_components_by_id: HashMap<ComponentId, Component> =
            components.iter().cloned().map(|c| (c.id(), c)).collect();
        let mut component_views = Vec::with_capacity(components.len());
        let new_component_ids: HashSet<ComponentId> = ctx
            .workspace_snapshot()?
            .components_added_relative_to_base(ctx)
            .await?
            .iter()
            .copied()
            .collect();

        // let new_attribute_prototype_argument_ids: HashSet<AttributePrototypeArgumentId> = ctx
        //     .workspace_snapshot()?
        //     .socket_edges_added_relative_to_base(ctx)
        //     .await?
        //     .iter()
        //     .copied()
        //     .collect();

        let new_attribute_prototype_argument_ids: HashSet<AttributePrototypeArgumentId> =
            HashSet::new();

        for component in &components {
            let component_change_status = if new_component_ids.contains(&component.id()) {
                ChangeStatus::Added
            } else if component.to_delete() {
                ChangeStatus::Deleted
            } else {
                // TODO: Eventually, we'll want to also handle detecting ChangeStatus::Modified
                ChangeStatus::Unmodified
            };

            for incoming_connection in component.incoming_connections(ctx).await? {
                let from_component =
                    Component::get_by_id(ctx, incoming_connection.from_component_id).await?;
                let edge_change_status = if component_change_status == ChangeStatus::Added
                    || new_component_ids.contains(&from_component.id())
                    || new_attribute_prototype_argument_ids
                        .contains(&incoming_connection.attribute_prototype_argument_id)
                {
                    ChangeStatus::Added
                } else {
                    ChangeStatus::Unmodified
                };

                diagram_edges.push(SummaryDiagramEdge::assemble(
                    incoming_connection,
                    from_component,
                    component,
                    edge_change_status,
                )?);
            }
            for inferred_incoming_connection in component.inferred_incoming_connections(ctx).await?
            {
                diagram_inferred_edges.push(SummaryDiagramInferredEdge::assemble(
                    inferred_incoming_connection,
                )?)
            }

            component_views.push(
                SummaryDiagramComponent::assemble(ctx, component, component_change_status).await?,
            );
        }

        // Even though the default change set for a workspace can have a base change set, we don't
        // want to consider anything as new/modified/removed when looking at the default change
        // set.
        let workspace = Workspace::get_by_pk_or_error(
            ctx,
            &ctx.tenancy()
                .workspace_pk()
                .ok_or(WorkspaceSnapshotError::WorkspaceMissing)?,
        )
        .await?;
        if workspace.default_change_set_id() != ctx.change_set_id()
            && ctx.change_set()?.base_change_set_id.is_some()
        {
            // We also want to display the edges & components that would be actively removed when we
            // merge this change set into its base change set.
            let removed_component_ids = ctx
                .workspace_snapshot()?
                .components_removed_relative_to_base(ctx)
                .await?;

            // We now need to retrieve these Components from the base change set, and build
            // SummaryDiagramComponents for them with from_base_change_set true, and change_status
            // ChangeStatus::Removed
            let base_change_set_ctx = ctx.clone_with_base().await?;
            let base_change_set_ctx = &base_change_set_ctx;

            for removed_component_id in removed_component_ids {
                // We don't need to worry about these duplicating SummaryDiagramComponents that
                // have already been generated as they're generated from the list of Components
                // still in the change set's snapshot, while the list of Component IDs we're
                // working on here are explicitly ones that do not exist in that snapshot anymore.
                let base_change_set_component =
                    Component::get_by_id(base_change_set_ctx, removed_component_id).await?;
                let mut summary_diagram_component = SummaryDiagramComponent::assemble(
                    base_change_set_ctx,
                    &base_change_set_component,
                    ChangeStatus::Deleted,
                )
                .await?;
                summary_diagram_component.from_base_change_set = true;
                virtual_and_real_components_by_id
                    .insert(removed_component_id, base_change_set_component);

                component_views.push(summary_diagram_component);
            }

            // We need to bring in any AttributePrototypeArguments for incoming & outgoing
            // connections that have been removed.
            let removed_attribute_prototype_argument_ids: Vec<AttributePrototypeArgumentId> =
                vec![];
            //  = ctx
            //     .workspace_snapshot()?
            //     .socket_edges_removed_relative_to_base(ctx)
            //     .await?;
            let mut incoming_connections_by_component_id: HashMap<
                ComponentId,
                Vec<IncomingConnection>,
            > = HashMap::new();

            for removed_attribute_prototype_argument_id in &removed_attribute_prototype_argument_ids
            {
                let attribute_prototype_argument = AttributePrototypeArgument::get_by_id(
                    base_change_set_ctx,
                    *removed_attribute_prototype_argument_id,
                )
                .await?;

                // This should always be Some as
                // `WorkspaceSnapshot::socket_edges_removed_relative_to_base` only returns the
                // IDs of arguments that have targets.
                if let Some(targets) = attribute_prototype_argument.targets() {
                    if let hash_map::Entry::Vacant(vacant_entry) =
                        incoming_connections_by_component_id.entry(targets.destination_component_id)
                    {
                        let removed_component = Component::get_by_id(
                            base_change_set_ctx,
                            targets.destination_component_id,
                        )
                        .await?;
                        let mut incoming_connections = removed_component
                            .incoming_connections(base_change_set_ctx)
                            .await?;
                        // We only care about connections going to the Component in the base that
                        // are *ALSO* ones that we are considered to have removed.
                        incoming_connections.retain(|connection| {
                            removed_attribute_prototype_argument_ids
                                .contains(&connection.attribute_prototype_argument_id)
                        });
                        vacant_entry.insert(incoming_connections);
                    }
                }
            }

            for incoming_connections in incoming_connections_by_component_id.values() {
                for incoming_connection in incoming_connections {
                    let from_component = virtual_and_real_components_by_id
                        .get(&incoming_connection.from_component_id)
                        .cloned()
                        .ok_or(ComponentError::NotFound(
                            incoming_connection.from_component_id,
                        ))?;
                    let to_component = virtual_and_real_components_by_id
                        .get(&incoming_connection.to_component_id)
                        .ok_or(ComponentError::NotFound(
                            incoming_connection.to_component_id,
                        ))?;
                    let mut summary_diagram_edge = SummaryDiagramEdge::assemble(
                        incoming_connection.clone(),
                        from_component,
                        to_component,
                        ChangeStatus::Deleted,
                    )?;
                    summary_diagram_edge.from_base_change_set = true;

                    diagram_edges.push(summary_diagram_edge);
                }
            }
        }

        Ok(Self {
            edges: diagram_edges,
            components: component_views,
            inferred_edges: diagram_inferred_edges,
        })
    }
}
