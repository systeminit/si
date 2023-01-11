use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::num::{ParseFloatError, ParseIntError};
use strum_macros::{AsRefStr, Display, EnumString};
use thiserror::Error;

use crate::diagram::connection::{Connection, DiagramEdgeView};
use crate::diagram::node::DiagramNodeView;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::socket::SocketError;
use crate::{
    AttributeContextBuilderError, AttributePrototypeArgumentError, AttributeValueError,
    ComponentError, DalContext, EdgeError, Node, NodeError, NodeKind, NodePosition,
    NodePositionError, PropError, ReadTenancyError, SchemaError, SocketId, StandardModel,
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
    #[error("component not found")]
    ComponentNotFound,
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
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
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
    nodes: Vec<DiagramNodeView>,
    /// The shape of assembled [`Edge`](crate::Edge) information to render graphical/visual edges.
    edges: Vec<DiagramEdgeView>,
}

impl Diagram {
    /// Assemble a [`Diagram`](Self) based on existing [`Nodes`](crate::Node) and
    /// [`Connections`](crate::Connection).
    pub async fn assemble(ctx: &DalContext) -> DiagramResult<Self> {
        let connections = Connection::list(ctx).await?;
        let nodes = Node::list(ctx).await?;

        let mut node_views = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let schema_variant = match node.kind() {
                NodeKind::Configuration => {
                    let component = node
                        .component(ctx)
                        .await?
                        .ok_or(DiagramError::ComponentNotFound)?;
                    component
                        .schema_variant(ctx)
                        .await?
                        .ok_or(DiagramError::SchemaVariantNotFound)?
                }
            };

            let positions = NodePosition::list_for_node(ctx, *node.id()).await?;

            // FIXME(nick): handle nodes with no position. Perhaps, we should generate one
            // automatically that is close to the origin, but does not share the same position as
            // another node.
            let position = match positions.into_iter().next() {
                Some(pos) => pos,
                None => continue, // Note: do we want to ignore things with no position?
            };
            let view = DiagramNodeView::new(ctx, node, &position, &schema_variant).await?;
            node_views.push(view);
        }

        Ok(Self {
            edges: connections
                .into_iter()
                .filter(|conn| {
                    node_views.iter().any(|n| {
                        let source_node_id = conn.source.node_id.to_string();
                        source_node_id == n.id()
                    })
                })
                .map(DiagramEdgeView::from)
                .collect(),
            nodes: node_views,
        })
    }

    pub fn nodes(&self) -> &[DiagramNodeView] {
        &self.nodes
    }

    pub fn edges(&self) -> &[DiagramEdgeView] {
        &self.edges
    }
}
