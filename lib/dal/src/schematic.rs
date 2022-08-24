use serde::{Deserialize, Serialize};
use si_data::PgError;
use std::num::ParseIntError;
use strum_macros::{AsRefStr, Display, EnumString};
use thiserror::Error;

use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::schematic::connection::{Connection, SchematicEdgeView};
use crate::schematic::node::SchematicNodeView;

use crate::{
    AttributePrototypeArgumentError, ComponentError, DalContext, EdgeError, Node, NodeError,
    NodeKind, NodePosition, NodePositionError, PropError, ReadTenancyError, StandardModel,
    StandardModelError, SystemError, SystemId,
};

pub mod connection;
pub mod node;

#[derive(Error, Debug)]
pub enum SchematicError {
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

pub type SchematicResult<T> = Result<T, SchematicError>;

#[derive(
    AsRefStr, Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SchematicKind {
    /// Only shows SchemaKind::Concrete and SchemaKind::Implementation
    /// They all have implementation input socket tied to a service output socket (?)
    Component,
    /// Only shows SchemaKind::Concept
    Deployment,
    System,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Schematic {
    nodes: Vec<SchematicNodeView>,
    edges: Vec<SchematicEdgeView>,
}

impl Schematic {
    pub async fn find(
        ctx: &DalContext<'_, '_>,
        system_id: Option<SystemId>,
    ) -> SchematicResult<Self> {
        let connections = Connection::list(ctx).await?;
        let nodes = Node::list(ctx).await?;

        let mut node_views = Vec::with_capacity(nodes.len());
        for node in &nodes {
            // TODO: we have to filter the components here by system

            // hide deployment nodes from the diagram since we'll likely remove it completely
            if *node.kind() == NodeKind::Deployment {
                continue;
            }

            let schema_variant = match node.kind() {
                NodeKind::Deployment | NodeKind::Component => {
                    let component = node
                        .component(ctx)
                        .await?
                        .ok_or(SchematicError::ComponentNotFound)?;
                    component
                        .schema_variant(ctx)
                        .await?
                        .ok_or(SchematicError::SchemaVariantNotFound)?
                }
                NodeKind::System => {
                    // We're going to skip all `NodeKind::System` nodes
                    continue;

                    // TODO(fnichol): We were failing in `node.system()` with an `Error: dal
                    // schematic error: system not found` error. For the moment we're going to
                    // filter out system-backed nodes, but ultimately we might want to return all
                    // node kinds back to the frontend for use.
                    //
                    // let system = node
                    //     .system(ctx)
                    //     .await?
                    //     .ok_or(SchematicError::SystemNotFound)?;
                    // let mut tenancy = tenancy.clone();
                    // tenancy.universal = true;
                    // let schema = system
                    //     .schema(ctx)
                    //     .await?
                    //     .ok_or(SchematicError::SchemaNotFound)?;

                    // (schema, system.name().to_owned())
                }
            };

            let positions = NodePosition::find_by_node_id(ctx, system_id, *node.id()).await?;
            let position = match positions.into_iter().next() {
                Some(pos) => pos,
                None => continue, // Note: do we want to ignore things with no position?
            };
            let view = SchematicNodeView::new(ctx, node, &position, &schema_variant).await?;
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
                .map(SchematicEdgeView::from)
                .collect(),
            nodes: node_views,
        })
    }

    pub fn nodes(&self) -> &[SchematicNodeView] {
        &self.nodes
    }

    pub fn edges(&self) -> &[SchematicEdgeView] {
        &self.edges
    }
}
