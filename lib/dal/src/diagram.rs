use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::num::{ParseFloatError, ParseIntError};
use strum::{AsRefStr, Display, EnumString};
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
    ActionPrototypeError, AttributeContextBuilderError, AttributePrototypeArgumentError,
    AttributeValueError, ChangeSetPk, ComponentError, ComponentId, DalContext, Edge, EdgeError,
    Node, NodeError, NodeId, NodeKind, PropError, SchemaError, SocketId, StandardModel,
    StandardModelError,
};

pub mod connection;
pub mod node;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("action prototype: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute context error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype not found")]
    AttributePrototypeNotFound,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found")]
    AttributeValueNotFound,
    #[error("change status error: {0}")]
    ChangeStatus(#[from] ChangeStatusError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("component status not found for component: {0}")]
    ComponentStatusNotFound(ComponentId),
    #[error("deletion timestamp not found")]
    DeletionTimeStamp,
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("edge not found")]
    EdgeNotFound,
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("external provider not found for socket id: {0}")]
    ExternalProviderNotFoundForSocket(SocketId),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("internal provider not found for socket id: {0}")]
    InternalProviderNotFoundForSocket(SocketId),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node not found")]
    NodeNotFound,
    #[error("no node positions found for node ({0}) and kind ({1})")]
    NoNodePositionsFound(NodeId, NodeKind),
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("position not found")]
    PositionNotFound,
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("socket not found")]
    SocketNotFound,
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
}

pub type DiagramResult<T> = Result<T, DiagramError>;

/// The kinds of [`Diagrams`](Diagram) available to choose between for rendering.
#[remain::sorted]
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

        let deleted_diagram_edges = ctx
            .run_with_deleted_visibility(|ctx_with_deleted| async move {
                let mut deleted_diagram_edges = Vec::new();

                for deleted_edge in deleted_edges {
                    if *deleted_edge.kind() == EdgeKind::Configuration {
                        let conn = Connection::from_edge(&deleted_edge);
                        let mut diagram_edge_view =
                            DiagramEdgeView::from_with_change_status(conn, ChangeStatus::Deleted);
                        diagram_edge_view
                            .set_actor_details(&ctx_with_deleted, &deleted_edge)
                            .await?;
                        deleted_diagram_edges.push(diagram_edge_view);
                    }
                }

                Ok::<_, DiagramError>(deleted_diagram_edges)
            })
            .await?;

        diagram_edges.extend(deleted_diagram_edges);

        let nodes = ctx
            .run_with_deleted_visibility(|ctx_with_deleted| async move {
                Node::list_live(&ctx_with_deleted, NodeKind::Configuration).await
            })
            .await?;

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

            let is_modified = modified
                .clone()
                .iter()
                .any(|s| s.component_id == *component.id());

            // Get Parent Id
            let sockets = SocketView::list(ctx, &schema_variant).await?;
            let maybe_socket_to_parent = sockets.iter().find(|socket| {
                socket.label == "Frame" && socket.direction == SocketDirection::Output
            });

            let edges_with_deleted =
                Edge::list_for_component(ctx_with_deleted, *component.id()).await?;

            let mut parent_node_id = None;

            if let Some(socket_to_parent) = maybe_socket_to_parent {
                for edge in &edges_with_deleted {
                    if edge.tail_node_id() == *node.id()
                        && edge.tail_socket_id().to_string() == socket_to_parent.id
                        && (edge.visibility().deleted_at.is_none() || edge.deleted_implicitly)
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
                for edge in &edges_with_deleted {
                    if edge.head_node_id() == *node.id()
                        && edge.head_socket_id().to_string() == socket_from_children.id
                        && (edge.visibility().deleted_at.is_none()
                            || (edge.deleted_implicitly && edge.visibility().in_change_set()))
                    {
                        let child_node = Node::get_by_id(ctx_with_deleted, &edge.tail_node_id())
                            .await?
                            .ok_or(DiagramError::NodeNotFound)?;

                        // This is a node in the current changeset and it is not deleted
                        if child_node.visibility().in_change_set()
                            && child_node.visibility().deleted_at.is_none()
                        {
                            child_node_ids.push(edge.tail_node_id());
                            continue;
                        }

                        // this is a node in the current changeset that has been marked as deleted
                        // now we need to check to see if it is exists in head
                        // if it does, then it's a ghosted node and should be included as a child
                        if child_node.visibility().in_change_set()
                            && child_node.visibility().deleted_at.is_some()
                        {
                            let head_ctx = &ctx.clone_with_head();
                            let head_node = Node::get_by_id(head_ctx, &edge.tail_node_id()).await?;
                            if head_node.is_some() {
                                child_node_ids.push(edge.tail_node_id());
                                continue;
                            }
                        }

                        // if the node is in head, doesn't exist directly on the changeset
                        // and not marked as deleted in head, then it's also a valid child
                        // *Remember*: a node won't exist in the changeset until a change is
                        // made to a node!!
                        if child_node.visibility().is_head()
                            && child_node.visibility().deleted_at.is_none()
                        {
                            child_node_ids.push(edge.tail_node_id());
                            continue;
                        }
                    }
                }
            };

            let view = DiagramComponentView::new(
                ctx_with_deleted,
                &component,
                node,
                parent_node_id,
                child_node_ids,
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
