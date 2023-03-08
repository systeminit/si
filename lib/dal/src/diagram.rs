use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::num::{ParseFloatError, ParseIntError};
use strum_macros::{AsRefStr, Display, EnumString};
use telemetry::prelude::debug;
use thiserror::Error;

use crate::change_status::{
    ChangeStatus, ChangeStatusError, ComponentChangeStatus, EdgeChangeStatus,
};
use crate::diagram::connection::{Connection, DiagramEdgeView};
use crate::diagram::node::{DiagramComponentView, SocketDirection, SocketView};
use crate::edge::EdgeKind;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::socket::SocketError;
use crate::{
    AttributeContextBuilderError, AttributePrototypeArgumentError, AttributeValueError,
    ChangeSetPk, ComponentError, ComponentId, DalContext, Edge, EdgeError, Node, NodeError, NodeId,
    NodeKind, NodePosition, NodePositionError, PropError, SchemaError, SocketId, StandardModel,
    StandardModelError,
};

pub mod connection;
pub mod node;

#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("change status error: {0}")]
    ChangeStatus(#[from] ChangeStatusError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("component status not found for component: {0}")]
    ComponentStatusNotFound(ComponentId),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("external provider not found for socket id: {0}")]
    ExternalProviderNotFoundForSocket(SocketId),
    #[error("internal provider not found for socket id: {0}")]
    InternalProviderNotFoundForSocket(SocketId),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("position not found")]
    PositionNotFound,
    #[error("node not found")]
    NodeNotFound,
    #[error("edge not found")]
    EdgeNotFound,
    #[error("socket not found")]
    SocketNotFound,
    #[error("attribute prototype not found")]
    AttributePrototypeNotFound,
    #[error("attribute value not found")]
    AttributeValueNotFound,
    #[error("deletion timestamp not found")]
    DeletionTimeStamp,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("attribute context error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("no node positions found for node ({0}) and kind ({1})")]
    NoNodePositionsFound(NodeId, NodeKind),
}

pub type DiagramResult<T> = Result<T, DiagramError>;

/// The kinds of [`Diagrams`](Diagram) available to choose between for rendering.
#[derive(
    AsRefStr, Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum DiagramKind {
    /// Represents the collection of [`Components`](crate::Component) and connections between them
    /// within a [`Workspace`](crate::Workspace)
    Configuration,
}

/// The shape of assembled graph-related information required to render a graphical/visual diagram.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagram {
    /// The shape of assembled [`Node`](crate::Node) information to render graphical/visual nodes.
    components: Vec<DiagramComponentView>,
    /// The shape of assembled [`Edge`](crate::Edge) information to render graphical/visual edges.
    edges: Vec<DiagramEdgeView>,
}

impl Diagram {
    /// Assemble a [`Diagram`](Self) based on existing [`Nodes`](crate::Node) and
    /// [`Connections`](crate::Connection).
    pub async fn assemble(ctx: &DalContext) -> DiagramResult<Self> {
        let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        let modified = ComponentChangeStatus::list_modified(ctx).await?;
        debug!("modified component change status: {modified:#?}");

        let mut diagram_edges = Vec::new();
        let edges = Edge::list(ctx).await?;
        for edge in &edges {
            if *edge.kind() == EdgeKind::Configuration {
                let change_status = match edge.visibility().change_set_pk {
                    ChangeSetPk::NONE => ChangeStatus::Unmodified,
                    _ => ChangeStatus::Added,
                };
                let conn = Connection::from_edge(edge);
                let mut diagram_edge_view =
                    DiagramEdgeView::from_with_change_status(conn, change_status);
                diagram_edge_view.set_actor_details(ctx, edge).await?;
                diagram_edges.push(diagram_edge_view);
            }
        }

        let deleted_edges: Vec<Edge> = EdgeChangeStatus::list_deleted(ctx).await?;
        for deleted_edge in deleted_edges {
            if *deleted_edge.kind() == EdgeKind::Configuration {
                let conn = Connection::from_edge(&deleted_edge);
                let mut diagram_edge_view =
                    DiagramEdgeView::from_with_change_status(conn, ChangeStatus::Deleted);
                diagram_edge_view
                    .set_actor_details(ctx_with_deleted, &deleted_edge)
                    .await?;
                diagram_edges.push(diagram_edge_view);
            }
        }

        let mut nodes = Node::list(ctx).await?;
        nodes.extend(ComponentChangeStatus::list_deleted_nodes(ctx).await?);

        let mut component_views = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let component = node
                .component(ctx_with_deleted)
                .await?
                .ok_or(DiagramError::ComponentNotFound)?;

            let schema_variant = match node.kind() {
                NodeKind::Configuration => component
                    .schema_variant(ctx_with_deleted)
                    .await?
                    .ok_or(DiagramError::SchemaVariantNotFound)?,
            };

            let positions = NodePosition::list_for_node(ctx_with_deleted, *node.id()).await?;

            let mut maybe_position = None;

            for this_position in positions {
                maybe_position = Some(this_position.clone());
                // NOTE(victor): This is setup to get you either the head position, or the specific changeset position, with priority on the latter
                // but it's brittle since if the way we create positions changes it will just give you the last position on the list there's no specific one
                // The real fix for this would be to fix the query to return the most valid entry for visibility OR make node_position a single value per node
                // This second option feels more adequate, but we need to check with product if multiple node position contexts are coming back.
                if maybe_position.is_some()
                    && this_position.visibility().change_set_pk == ctx.visibility().change_set_pk
                {
                    break;
                }
            }

            let position = match maybe_position {
                Some(pos) => pos,
                None => return Err(DiagramError::NoNodePositionsFound(*node.id(), *node.kind())),
            };

            let is_modified = modified
                .clone()
                .iter()
                .any(|s| s.component_id == *component.id());

            // Get Parent Id
            let sockets = SocketView::list(ctx, &schema_variant).await?;
            let maybe_socket_to_parent = sockets.iter().find(|socket| {
                socket.label == "Frame" && socket.direction == SocketDirection::Output
            });

            let mut parent_node_id = None;

            if let Some(socket_to_parent) = maybe_socket_to_parent {
                for edge in &edges {
                    if edge.tail_node_id() == *node.id()
                        && edge.tail_socket_id().to_string() == socket_to_parent.id
                    {
                        parent_node_id = Some(edge.head_node_id());
                        break;
                    }
                }
            };

            // Get Child Ids
            let maybe_socket_from_children = sockets.iter().find(|socket| {
                socket.label == "Frame" && socket.direction == SocketDirection::Input
            });

            let mut child_node_ids = vec![];
            if let Some(socket_from_children) = maybe_socket_from_children {
                for edge in &edges {
                    if edge.head_node_id() == *node.id()
                        && edge.head_socket_id().to_string() == socket_from_children.id
                    {
                        child_node_ids.push(edge.tail_node_id());
                    }
                }
            };

            let view = DiagramComponentView::new(
                ctx_with_deleted,
                &component,
                node,
                parent_node_id,
                child_node_ids,
                &position,
                is_modified,
                &schema_variant,
            )
            .await?;
            component_views.push(view);
        }

        Ok(Self {
            edges: diagram_edges,
            components: component_views,
        })
    }

    pub fn components(&self) -> &[DiagramComponentView] {
        &self.components
    }

    pub fn edges(&self) -> &[DiagramEdgeView] {
        &self.edges
    }
}
