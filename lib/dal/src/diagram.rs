use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::num::{ParseFloatError, ParseIntError};
use strum::{AsRefStr, Display, EnumString};

use thiserror::Error;

use crate::change_status::ChangeStatusError;

use crate::diagram::summary_diagram::{SummaryDiagramComponent, SummaryDiagramEdge};

use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::socket::SocketError;
use crate::{
    ActionPrototypeError, AttributeContextBuilderError, AttributePrototypeArgumentError,
    AttributeValueError, ComponentError, ComponentId, DalContext, EdgeError, NodeError, NodeId,
    NodeKind, PropError, SchemaError, SocketId, StandardModelError,
};

pub mod connection;
pub(crate) mod summary_diagram;
pub use summary_diagram::falsify_using_default_variant_for_components_of_schema;
pub use summary_diagram::{SummaryDiagramError, SummaryDiagramResult};

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
    #[error("summary diagram error: {0}")]
    SummaryDiagram(String),
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
    components: Vec<SummaryDiagramComponent>,
    /// The shape of assembled [`Edge`](crate::Edge) information to render graphical/visual edges.
    edges: Vec<SummaryDiagramEdge>,
}

impl Diagram {
    /// Assemble a [`Diagram`](Self) based on existing [`Nodes`](crate::Node) and
    /// [`Connections`](crate::Connection).
    pub async fn assemble(ctx: &DalContext) -> DiagramResult<Self> {
        let components = summary_diagram::component_list(ctx)
            .await
            .map_err(|e| DiagramError::SummaryDiagram(e.to_string()))?;
        let edges = summary_diagram::edge_list(ctx)
            .await
            .map_err(|e| DiagramError::SummaryDiagram(e.to_string()))?;

        Ok(Self { edges, components })
    }

    pub fn components(&self) -> &[SummaryDiagramComponent] {
        &self.components
    }

    pub fn edges(&self) -> &[SummaryDiagramEdge] {
        &self.edges
    }
}
