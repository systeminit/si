use crate::{
    node::NodeId, ComponentError, Node, NodeError, NodeKind, NodePosition, NodePositionError,
    NodeTemplate, NodeView, StandardModel, StandardModelError, SystemId, Tenancy, Visibility,
};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

use si_data::{PgError, PgTxn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchematicError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("position not found")]
    PositionNotFound,
    #[error("component not foundl")]
    ComponentNotFound,
    #[error("schema not foundl")]
    SchemaNotFound,
}

pub type SchematicResult<T> = Result<T, SchematicError>;

#[derive(
    AsRefStr, Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SchematicKind {
    Component,
    Deployment,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Schematic {
    nodes: Vec<NodeView>,
    // Dummy type, we should actually have a Connection struct defined somewhere
    connections: Vec<String>,
}

impl Schematic {
    pub async fn find(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        system_id: Option<SystemId>,
        root_node_id: NodeId,
    ) -> SchematicResult<Self> {
        let nodes: Vec<Node> = Node::list(txn, tenancy, visibility).await?;

        let mut node_views = Vec::with_capacity(nodes.len());
        for node in nodes {
            let (schema, name, schematic_kind) = match node.kind() {
                NodeKind::Component => {
                    let component = node
                        .component(txn, visibility)
                        .await?
                        .ok_or(SchematicError::ComponentNotFound)?;
                    let mut tenancy = tenancy.clone();
                    tenancy.universal = true;
                    let schema = component
                        .schema_with_tenancy(txn, &tenancy, visibility)
                        .await?
                        .ok_or(SchematicError::SchemaNotFound)?;
                    (
                        schema,
                        component.name().to_owned(),
                        SchematicKind::Component,
                    )
                }
            };

            let position = NodePosition::find_by_node_id(
                txn,
                &tenancy,
                visibility,
                schematic_kind,
                &system_id,
                root_node_id,
                *node.id(),
            )
            .await?;
            let template =
                NodeTemplate::new_from_schema_id(txn, &tenancy, visibility, *schema.id()).await?;
            let view = NodeView::new(name, node, position.map_or(vec![], |p| vec![p]), template);
            node_views.push(view);
        }
        let connections = vec![]; // TODO: retrieve actual connections (they don't exist yet in the backend)
        Ok(Self {
            nodes: node_views,
            connections,
        })
    }
}
