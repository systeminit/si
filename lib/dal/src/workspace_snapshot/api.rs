use crate::change_set_pointer::ChangeSetPointer;
use petgraph::stable_graph::EdgeIndex;
use petgraph::stable_graph::Edges;
use petgraph::visit::EdgeRef;
use petgraph::Directed;
use std::collections::HashMap;
use ulid::Ulid;

use crate::workspace_snapshot::edge_weight::EdgeWeight;
use crate::workspace_snapshot::graph::Direction;
use crate::workspace_snapshot::graph::NodeIndex;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::WorkspaceSnapshotResult;
use crate::WorkspaceSnapshot;

use super::edge_weight::EdgeWeightKindDiscriminants;

pub mod attribute;
// pub mod component;
pub mod func;
// pub mod node;
pub mod prop;
pub mod provider;
pub mod schema;
pub mod socket;
pub mod validation;

impl WorkspaceSnapshot {
    pub fn get_category(
        &mut self,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotResult<(Ulid, NodeIndex)> {
        Ok(self.working_copy()?.get_category(kind)?)
    }

    pub fn edges_directed(
        &mut self,
        id: Ulid,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Edges<'_, EdgeWeight, Directed, u32>> {
        let node_index = self.working_copy()?.get_node_index_by_id(id)?;
        Ok(self.working_copy()?.edges_directed(node_index, direction))
    }

    pub fn edges_directed_by_index(
        &mut self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> WorkspaceSnapshotResult<Edges<'_, EdgeWeight, Directed, u32>> {
        Ok(self.working_copy()?.edges_directed(node_index, direction))
    }

    pub fn incoming_sources_for_edge_weight_kind(
        &mut self,
        id: Ulid,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        Ok(self
            .edges_directed(id, Direction::Incoming)?
            .filter_map(|edge_ref| {
                if edge_weight_kind_discrim == edge_ref.weight().kind().into() {
                    Some(edge_ref.source())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn outgoing_targets_for_edge_weight_kind(
        &mut self,
        id: Ulid,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        Ok(self
            .edges_directed(id, Direction::Outgoing)?
            .filter_map(|edge_ref| {
                if edge_weight_kind_discrim == edge_ref.weight().kind().into() {
                    Some(edge_ref.target())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn outgoing_targets_for_edge_weight_kind_by_index(
        &mut self,
        node_index: NodeIndex,
        edge_weight_kind_discrim: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<Vec<NodeIndex>> {
        Ok(self
            .edges_directed_by_index(node_index, Direction::Outgoing)?
            .filter_map(|edge_ref| {
                if edge_weight_kind_discrim == edge_ref.weight().kind().into() {
                    Some(edge_ref.target())
                } else {
                    None
                }
            })
            .collect())
    }

    pub fn remove_edge(
        &mut self,
        change_set: &ChangeSetPointer,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotResult<HashMap<NodeIndex, NodeIndex>> {
        Ok(self.working_copy()?.remove_edge(
            change_set,
            source_node_index,
            target_node_index,
            edge_kind,
        )?)
    }
}
