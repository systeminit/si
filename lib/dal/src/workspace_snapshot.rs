//! Mostly everything is a node or an edge!

// #![warn(
//     missing_debug_implementations,
//     missing_docs,
//     unreachable_pub,
//     bad_style,
//     dead_code,
//     improper_ctypes,
//     non_shorthand_field_patterns,
//     no_mangle_generic_items,
//     overflowing_literals,
//     path_statements,
//     patterns_in_fns_without_body,
//     private_in_public,
//     unconditional_recursion,
//     unused,
//     unused_allocation,
//     unused_comparisons,
//     unused_parens,
//     while_true,
//     clippy::missing_panics_doc
// )]

pub mod edge;
pub mod lamport_clock;
pub mod node;
pub mod vector_clock;

use petgraph::algo;
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use ulid::{Generator, Ulid};

use crate::workspace_snapshot::edge::SnapshotEdge;
use crate::workspace_snapshot::node::{SnapshotNode, SnapshotNodeKind};
use crate::{DalContext, StandardModelError, Timestamp, TransactionsError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceSnapshotError {
    #[error("si_data_pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot graph missing")]
    WorkspaceSnapshotGraphMissing,
}

pub type WorkspaceSnapshotResult<T> = Result<T, WorkspaceSnapshotError>;

// FIXME(nick): remove this in favor of the real one.
pub type ChangeSetId = Ulid;

// FIXME(nick): remove this in favor of the real one.
pub struct ChangeSet {
    pub id: ChangeSetId,
    pub generator: Arc<Mutex<Generator>>,
}

pub type WorkspaceSnapshotId = Ulid;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    id: WorkspaceSnapshotId,
    #[serde(flatten)]
    timestamp: Timestamp,
    snapshot: Value,
    #[serde(skip_serializing)]
    working_copy: Option<WorkspaceSnapshotGraph>,
}

impl WorkspaceSnapshot {
    pub async fn new(ctx: &DalContext) -> WorkspaceSnapshotResult<Self> {
        let snapshot = WorkspaceSnapshotGraph::new();
        let serialized_snapshot = serde_json::to_value(&snapshot)?;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT workspace_snapshot_create_v1($1) AS object",
                &[&serialized_snapshot],
            )
            .await?;
        let json: Value = row.try_get("object")?;
        let object: WorkspaceSnapshot = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn write(mut self, ctx: &DalContext) -> WorkspaceSnapshotResult<Self> {
        let working_copy = self.working_copy()?;
        working_copy.cleanup();

        let serialized_snapshot = serde_json::to_value(working_copy.clone())?;
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT workspace_snapshot_create_v1($1) AS object",
                &[&serialized_snapshot],
            )
            .await?;
        let json: Value = row.try_get("object")?;
        let object: WorkspaceSnapshot = serde_json::from_value(json)?;
        Ok(object)
    }

    fn working_copy(&mut self) -> WorkspaceSnapshotResult<&mut WorkspaceSnapshotGraph> {
        if self.working_copy.is_none() {
            self.working_copy = Some(serde_json::from_value(self.snapshot.clone())?);
        }
        self.working_copy
            .as_mut()
            .ok_or(WorkspaceSnapshotError::WorkspaceSnapshotGraphMissing)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraph {
    pub root_index: NodeIndex,
    pub graph: StableGraph<SnapshotNode, SnapshotEdge>,
}

impl WorkspaceSnapshotGraph {
    pub fn new() -> Self {
        let mut graph: StableGraph<SnapshotNode, SnapshotEdge> = StableGraph::with_capacity(1, 0);
        let root_index = graph.add_node(SnapshotNode::new(SnapshotNodeKind::Root));
        Self { graph, root_index }
    }

    pub fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    pub fn cleanup(&mut self) {
        self.graph.retain_nodes(|frozen_graph, current_node| {
            algo::has_path_connecting(&*frozen_graph, self.root_index, current_node, None)
        });
    }

    fn add_node(&mut self, node: SnapshotNode) -> NodeIndex {
        self.graph.add_node(node)
    }

    fn add_edge(
        &mut self,
        edge: SnapshotEdge,
        parent_node_index: NodeIndex,
        node_index: NodeIndex,
    ) -> EdgeIndex {
        self.graph.add_edge(parent_node_index, node_index, edge)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::workspace_snapshot::edge::SnapshotEdgeKind;

    #[test]
    fn new() {
        let graph = WorkspaceSnapshotGraph::new();
        assert!(graph.is_acyclic_directed());
    }

    #[test]
    fn add_nodes_and_edges() {
        let mut graph = WorkspaceSnapshotGraph::new();

        let schema_index = graph.add_node(SnapshotNode::new(SnapshotNodeKind::Schema));
        let schema_variant_index =
            graph.add_node(SnapshotNode::new(SnapshotNodeKind::SchemaVariant));
        let component_index = graph.add_node(SnapshotNode::new(SnapshotNodeKind::Component));

        graph.add_edge(SnapshotEdge::default(), graph.root_index, schema_index);
        graph.add_edge(SnapshotEdge::default(), schema_index, schema_variant_index);
        graph.add_edge(
            SnapshotEdge::default(),
            schema_variant_index,
            component_index,
        );

        let func_index = graph.add_node(SnapshotNode::new(SnapshotNodeKind::Func));
        let prop_index = graph.add_node(SnapshotNode::new(SnapshotNodeKind::Prop));

        graph.add_edge(SnapshotEdge::default(), graph.root_index, func_index);
        graph.add_edge(SnapshotEdge::default(), schema_variant_index, func_index);
        graph.add_edge(SnapshotEdge::default(), schema_variant_index, prop_index);
        graph.add_edge(SnapshotEdge::default(), prop_index, func_index);

        assert!(graph.is_acyclic_directed());
    }
}
