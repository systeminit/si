use petgraph::prelude::NodeIndex;
use si_events::ulid::Ulid;

use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;

use super::{
    edge_info::EdgeInfo,
    edge_weight::{EdgeWeight, EdgeWeightKindDiscriminants},
    graph::WorkspaceSnapshotGraphResult,
};
use crate::{workspace_snapshot::NodeInformation, WorkspaceSnapshotGraph};

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
    // This is not correctly named. We really only want to replace the single node, as we also
    // generate Update entries to handle processing the rest of the subgraph.
    ReplaceSubgraph {
        onto: NodeInformation,
        // Check if already exists in "onto". Grab node weight from "to_rebase" and see if there is
        // an equivalent node (id and lineage) in "onto". If not, use "import_subgraph".
        to_rebase: NodeInformation,
    },
    MergeCategoryNodes {
        to_rebase_category_id: Ulid,
        onto_category_id: Ulid,
    },
}

impl Update {
    /// Produce a NewEdge update from an edge that exists only in the "onto" graph
    pub fn new_edge(
        to_rebase_graph: &WorkspaceSnapshotGraph,
        onto_graph: &WorkspaceSnapshotGraph,
        to_rebase_source_index: NodeIndex,
        only_onto_edge_info: &EdgeInfo,
        only_onto_edge_weight: EdgeWeight,
    ) -> WorkspaceSnapshotGraphResult<Self> {
        let source_node_weight = to_rebase_graph.get_node_weight(to_rebase_source_index)?;
        let target_node_weight =
            onto_graph.get_node_weight(only_onto_edge_info.target_node_index)?;

        let source = NodeInformation {
            index: to_rebase_source_index,
            id: source_node_weight.id().into(),
            node_weight_kind: source_node_weight.into(),
        };
        let destination = NodeInformation {
            index: only_onto_edge_info.target_node_index,
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
