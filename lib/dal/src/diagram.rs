use serde::{Deserialize, Serialize};
use si_data::PgError;
use std::num::ParseIntError;
use strum_macros::{AsRefStr, Display, EnumString};
use thiserror::Error;

use crate::diagram::connection::{Connection, DiagramEdgeView};
use crate::diagram::node::DiagramNodeView;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::{
    AttributePrototypeArgumentError, ComponentError, DalContext, EdgeError, Node, NodeError,
    NodeKind, NodePosition, NodePositionError, PropError, ReadTenancyError, StandardModel,
    StandardModelError, SystemError, SystemId,
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
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("position not found")]
    PositionNotFound,
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("system error: {0}")]
    System(#[from] SystemError),
    #[error("system not found")]
    SystemNotFound,
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
    /// within a [`Workspace`](crate::Workspace) and [`System`](crate::System)... well, at least the
    /// [`Workspace`](crate::Workspace) part.
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
    pub async fn assemble(ctx: &DalContext, system_id: Option<SystemId>) -> DiagramResult<Self> {
        let connections = Connection::list(ctx).await?;
        let nodes = Node::list(ctx).await?;

        let mut node_views = Vec::with_capacity(nodes.len());
        for node in &nodes {
            // TODO: we have to filter the components here by system

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
                NodeKind::System => {
                    // We're going to skip all `NodeKind::System` nodes
                    continue;

                    // TODO(fnichol): We were failing in `node.system()` with an `Error: dal
                    // diagram error: system not found` error. For the moment we're going to
                    // filter out system-backed nodes, but ultimately we might want to return all
                    // node kinds back to the frontend for use.
                    //
                    // let system = node
                    //     .system(ctx)
                    //     .await?
                    //     .ok_or(DiagramError::SystemNotFound)?;
                    // let mut tenancy = tenancy.clone();
                    // tenancy.universal = true;
                    // let schema = system
                    //     .schema(ctx)
                    //     .await?
                    //     .ok_or(DiagramError::SchemaNotFound)?;

                    // (schema, system.name().to_owned())
                }
            };

            let positions = NodePosition::list_for_node(ctx, *node.id(), system_id).await?;

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
                        let source_node_id: i64 = conn.source.node_id.into();
                        source_node_id.to_string() == n.id()
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
