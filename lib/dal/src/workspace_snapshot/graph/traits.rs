use std::collections::HashSet;

use async_trait::async_trait;
use petgraph::prelude::*;
use si_events::ulid::Ulid;

use crate::{
    workspace_snapshot::{
        graph::{detect_updates::Update, WorkspaceSnapshotGraphResult},
        node_weight::NodeWeight,
    },
    EdgeWeight,
};

#[async_trait]
pub trait SnapshotGraphInterface {
    type Inner;

    fn cleanup(&mut self);
    fn detect_updates(&self, updated_graph: &Self::Inner) -> Vec<Update>;
    fn edges(&self) -> impl Iterator<Item = (&EdgeWeight, NodeIndex, NodeIndex)>;
    fn get_edge_weight_opt(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<&EdgeWeight>>;
    /// The impl for this should `#[inline(always)]`.
    fn get_node_index_by_id(&self, id: impl Into<Ulid>) -> WorkspaceSnapshotGraphResult<NodeIndex>;
    /// The impl for this should `#[inline(always)]`.
    fn get_node_index_by_id_opt(&self, id: impl Into<Ulid>) -> Option<NodeIndex>;
    fn get_node_index_by_lineage(&self, lineage_id: Ulid) -> HashSet<NodeIndex>;
    fn get_node_weight(&self, node_index: NodeIndex) -> WorkspaceSnapshotGraphResult<&NodeWeight>;
    fn get_node_weight_by_id_opt(&self, id: impl Into<Ulid>) -> Option<&NodeWeight>;
    fn get_node_weight_mut(
        &mut self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&mut NodeWeight>;
    fn get_node_weight_opt(&self, node_index: NodeIndex) -> Option<&NodeWeight>;
    fn import_component_subgraph(
        &mut self,
        other: &Self::Inner,
        component_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()>;
    fn new() -> Self::Inner;
    fn node_index_to_id(&self, node_idx: NodeIndex) -> Option<Ulid>;
    fn nodes(&self) -> impl Iterator<Item = (&NodeWeight, NodeIndex)>;
    fn root(&self) -> NodeIndex;
}
