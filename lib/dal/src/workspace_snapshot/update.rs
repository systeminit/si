use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use super::{
    edge_info::EdgeInfo,
    edge_weight::{EdgeWeight, EdgeWeightKindDiscriminants},
    graph::WorkspaceSnapshotGraphResult,
    node_weight::NodeWeight,
};
use crate::{workspace_snapshot::NodeInformation, WorkspaceSnapshotGraphV1};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, EnumDiscriminants)]
pub enum Update {
    NewEdge {
        source: NodeInformation,
        // Check if already exists in "onto" (source). Grab node weight from "to_rebase"
        // (destination) and see if there is an equivalent node (id and lineage) in "onto".
        // If not, use "import_subgraph".
        destination: NodeInformation,
        edge_weight: EdgeWeight,
    },
    RemoveEdge {
        source: NodeInformation,
        destination: NodeInformation,
        edge_kind: EdgeWeightKindDiscriminants,
    },
    ReplaceNode {
        node_weight: NodeWeight,
    },
    NewNode {
        node_weight: NodeWeight,
    },
}

impl Update {
    /// Produce a NewEdge update from an edge that exists only in the "onto" graph
    pub fn new_edge(
        onto_graph: &WorkspaceSnapshotGraphV1,
        only_onto_edge_info: &EdgeInfo,
        only_onto_edge_weight: EdgeWeight,
    ) -> WorkspaceSnapshotGraphResult<Self> {
        let source_node_weight =
            onto_graph.get_node_weight(only_onto_edge_info.source_node_index)?;
        let target_node_weight =
            onto_graph.get_node_weight(only_onto_edge_info.target_node_index)?;

        let source = NodeInformation {
            id: source_node_weight.id().into(),
            node_weight_kind: source_node_weight.into(),
        };
        let destination = NodeInformation {
            id: target_node_weight.id().into(),
            node_weight_kind: target_node_weight.into(),
        };

        Ok(Update::NewEdge {
            source,
            destination,
            edge_weight: only_onto_edge_weight,
        })
    }
}
