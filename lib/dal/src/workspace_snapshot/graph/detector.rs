use std::collections::{HashMap, HashSet};

use petgraph::{
    prelude::*,
    visit::{Control, DfsEvent},
};
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};
use strum::EnumDiscriminants;
use telemetry::prelude::*;

use crate::{
    workspace_snapshot::{node_weight::NodeWeight, NodeInformation},
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};

use super::WorkspaceSnapshotGraphVCurrent;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, EnumDiscriminants)]
pub enum Update {
    NewEdge {
        source: NodeInformation,
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

#[derive(Debug)]
pub struct Change {
    pub id: Ulid,
    pub merkle_tree_hash: MerkleTreeHash,
}

#[derive(Clone, Debug)]
enum NodeDifference {
    NewNode,
    MerkleTreeHash(Vec<NodeIndex>),
}

pub struct Detector<'a, 'b> {
    base_graph: &'a WorkspaceSnapshotGraphVCurrent,
    updated_graph: &'b WorkspaceSnapshotGraphVCurrent,
}

impl<'a, 'b> Detector<'a, 'b> {
    pub fn new(
        base_graph: &'a WorkspaceSnapshotGraphVCurrent,
        updated_graph: &'b WorkspaceSnapshotGraphVCurrent,
    ) -> Self {
        Self {
            base_graph,
            updated_graph,
        }
    }

    /// Performs a post order walk of the updated graph, finding the updates
    /// made to it when compared to the base graph, using the Merkle tree hash
    /// to detect changes and ignore unchanged branches.
    ///
    /// This assumes that all graphs involved to not have any "garbage" laying around. If in doubt,
    /// run [`cleanup`][WorkspaceSnapshotGraph::cleanup] on all involved graphs, before handing
    /// them over to the [`Detector`].
    pub fn detect_updates(&self) -> Vec<Update> {
        let mut updates = vec![];
        let mut difference_cache = HashMap::new();

        petgraph::visit::depth_first_search(
            self.updated_graph.graph(),
            Some(self.updated_graph.root()),
            |event| self.calculate_updates_dfs_event(event, &mut updates, &mut difference_cache),
        );

        updates
    }

    /// Performs a post order walk of an updated graph, finding the nodes that have been added, removed or modified.
    ///
    /// This assumes that all graphs involved to not have any "garbage" laying around. If in doubt, perform "cleanup"
    /// on both graphs before creating the [`Detector`].
    pub fn detect_changes(&self) -> Vec<Change> {
        let mut changes = Vec::new();

        petgraph::visit::depth_first_search(
            self.updated_graph.graph(),
            Some(self.updated_graph.root()),
            |event| self.calculate_changes_dfs_event(event, &mut changes),
        );

        changes
    }

    fn node_diff_from_base_graph(
        &self,
        updated_graph_node_index: NodeIndex,
    ) -> Option<NodeDifference> {
        self.updated_graph
            .get_node_weight_opt(updated_graph_node_index)
            .and_then(|updated_graph_node_weight| {
                let mut base_graph_node_indexes = HashSet::new();
                if updated_graph_node_index == self.updated_graph.root() {
                    // There can only be one (valid/current) `ContentAddress::Root` at any
                    // given moment, and the `lineage_id` isn't really relevant as it's not
                    // globally stable (even though it is locally stable). This matters as we
                    // may be dealing with a `WorkspaceSnapshotGraph` that is coming to us
                    // externally from a module that we're attempting to import. The external
                    // `WorkspaceSnapshotGraph` will be `self`, and the "local" one will be
                    // `onto`.
                    base_graph_node_indexes.insert(self.base_graph.root());
                } else {
                    let new_base_graph_node_indexes = self
                        .base_graph
                        .get_node_index_by_lineage(updated_graph_node_weight.lineage_id());

                    base_graph_node_indexes.extend(new_base_graph_node_indexes);
                }

                base_graph_node_indexes
                    .is_empty()
                    .then_some(NodeDifference::NewNode)
                    .or_else(|| {
                        // If everything with the same `lineage_id` is identical, then
                        // we can prune the graph traversal, and avoid unnecessary
                        // lookups/comparisons.
                        let nodes_with_difference: Vec<NodeIndex> = base_graph_node_indexes
                            .iter()
                            .filter_map(|&base_graph_index| {
                                self.base_graph
                                    .get_node_weight_opt(base_graph_index)
                                    .and_then(|base_graph_node_weight| {
                                        (base_graph_node_weight.merkle_tree_hash()
                                            != updated_graph_node_weight.merkle_tree_hash())
                                        .then_some(base_graph_index)
                                    })
                            })
                            .collect();

                        (!nodes_with_difference.is_empty())
                            .then_some(NodeDifference::MerkleTreeHash(nodes_with_difference))
                    })
            })
    }

    /// Produces ReplaceNode, NewEdge and RemoveEdge updates. The assumption we
    /// make here is that updated_graph has seen everything in base_graph. So if
    /// a node has a different hash from the matching one in base_graph, it has
    /// been changed and should be replaced. And if an edge is in base_graph but
    /// not in updated_graph, that means it's been removed. Finally, if an edge
    /// is in new_graph, but not in base_graph, that means it has been added.
    fn detect_updates_for_node_index(
        &self,
        updated_graph_node_index: NodeIndex,
        base_graph_indexes: &[NodeIndex],
    ) -> Vec<Update> {
        #[derive(Debug, Clone)]
        struct EdgeInfo {
            pub source_node: NodeInformation,
            pub target_node: NodeInformation,
            pub edge_weight: EdgeWeight,
        }

        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        struct UniqueEdgeInfo {
            pub kind: EdgeWeightKind,
            pub target_lineage: Ulid,
        }

        let mut updates = vec![];

        if let Some(updated_graph_node_weight) = self
            .updated_graph
            .get_node_weight_opt(updated_graph_node_index)
        {
            for base_graph_index in base_graph_indexes {
                let base_graph_index = *base_graph_index;

                if let Some(base_graph_node_weight) =
                    self.base_graph.get_node_weight_opt(base_graph_index)
                {
                    // if the node hash is different, then the node has been updated and
                    // needs to be replaced in base_graph (node hash is a hash of the
                    // content, which is not the same as the merkle tree hash, which
                    // also gathers up the hashes of the outgoing neighbors)
                    if updated_graph_node_weight.node_hash() != base_graph_node_weight.node_hash() {
                        updates.push(Update::ReplaceNode {
                            node_weight: updated_graph_node_weight.to_owned(),
                        });
                    }

                    let base_graph_edges: HashMap<UniqueEdgeInfo, EdgeInfo> = self
                        .base_graph
                        .graph()
                        .edges_directed(base_graph_index, Outgoing)
                        .filter_map(|edge_ref| {
                            self.base_graph.get_node_weight_opt(edge_ref.target()).map(
                                |target_node_weight| {
                                    (
                                        UniqueEdgeInfo {
                                            kind: edge_ref.weight().kind().clone(),
                                            target_lineage: target_node_weight.lineage_id(),
                                        },
                                        EdgeInfo {
                                            source_node: NodeInformation {
                                                id: base_graph_node_weight.id().into(),
                                                node_weight_kind: base_graph_node_weight.into(),
                                            },
                                            target_node: NodeInformation {
                                                id: target_node_weight.id().into(),
                                                node_weight_kind: target_node_weight.into(),
                                            },
                                            edge_weight: edge_ref.weight().to_owned(),
                                        },
                                    )
                                },
                            )
                        })
                        .collect();

                    let update_graph_edges: HashMap<UniqueEdgeInfo, EdgeInfo> = self
                        .updated_graph
                        .graph()
                        .edges_directed(updated_graph_node_index, Outgoing)
                        .filter_map(|edge_ref| {
                            self.updated_graph
                                .get_node_weight_opt(edge_ref.target())
                                .map(|target_node_weight| {
                                    (
                                        UniqueEdgeInfo {
                                            kind: edge_ref.weight().kind().clone(),
                                            target_lineage: target_node_weight.lineage_id(),
                                        },
                                        EdgeInfo {
                                            source_node: NodeInformation {
                                                id: updated_graph_node_weight.id().into(),
                                                node_weight_kind: updated_graph_node_weight.into(),
                                            },
                                            target_node: NodeInformation {
                                                id: target_node_weight.id().into(),
                                                node_weight_kind: target_node_weight.into(),
                                            },
                                            edge_weight: edge_ref.weight().to_owned(),
                                        },
                                    )
                                })
                        })
                        .collect();

                    updates.extend(
                        base_graph_edges
                            .iter()
                            .filter(|(edge_key, _)| !update_graph_edges.contains_key(edge_key))
                            .map(|(_, edge_info)| Update::RemoveEdge {
                                source: edge_info.source_node,
                                destination: edge_info.target_node,
                                edge_kind: edge_info.edge_weight.kind().into(),
                            }),
                    );

                    updates.extend(
                        update_graph_edges
                            .into_iter()
                            .filter(|(edge_key, _)| !base_graph_edges.contains_key(edge_key))
                            .map(|(_, edge_info)| Update::NewEdge {
                                source: edge_info.source_node,
                                destination: edge_info.target_node,
                                edge_weight: edge_info.edge_weight,
                            }),
                    );
                }
            }
        }

        updates
    }

    fn calculate_updates_dfs_event(
        &self,
        event: DfsEvent<NodeIndex>,
        updates: &mut Vec<Update>,
        difference_cache: &mut HashMap<NodeIndex, Option<NodeDifference>>,
    ) -> Control<()> {
        match event {
            DfsEvent::Discover(updated_graph_index, _) => {
                let node_diff = self.node_diff_from_base_graph(updated_graph_index);
                let control = match &node_diff {
                    Some(NodeDifference::NewNode) => {
                        if let Some(node_weight) =
                            self.updated_graph.get_node_weight_opt(updated_graph_index)
                        {
                            // NewNode updates are produced here, so that they
                            // are in the update array *before* the new edge
                            // updates which refer to them
                            updates.push(Update::NewNode {
                                node_weight: node_weight.to_owned(),
                            });
                        }

                        Control::Continue
                    }
                    Some(NodeDifference::MerkleTreeHash(_)) => Control::Continue,
                    // Node is neither different, nor new, prune this branch of
                    // the graph
                    None => Control::Prune,
                };

                difference_cache.insert(updated_graph_index, node_diff);

                control
            }
            DfsEvent::Finish(updated_graph_index, _) => {
                match difference_cache.get(&updated_graph_index) {
                    // None should be unreachable here....
                    None | Some(None) => Control::Continue,
                    Some(Some(diff)) => {
                        match diff {
                            NodeDifference::NewNode => {
                                // A new node! Just gather up all the outgoing edges as NewEdge updates
                                updates.extend(
                                    self.updated_graph
                                        .edges_directed(updated_graph_index, Outgoing)
                                        .map(|edge_ref| {
                                            (edge_ref.target(), edge_ref.weight().to_owned())
                                        })
                                        .filter_map(move |(target_index, edge_weight)| {
                                            if let (Some(source_node), Some(destination_node)) = (
                                                self.updated_graph
                                                    .get_node_weight_opt(updated_graph_index),
                                                self.updated_graph
                                                    .get_node_weight_opt(target_index),
                                            ) {
                                                Some(Update::NewEdge {
                                                    source: NodeInformation {
                                                        node_weight_kind: source_node.into(),
                                                        id: source_node.id().into(),
                                                    },
                                                    destination: NodeInformation {
                                                        node_weight_kind: destination_node.into(),
                                                        id: destination_node.id().into(),
                                                    },
                                                    edge_weight,
                                                })
                                            } else {
                                                None
                                            }
                                        }),
                                );
                            }
                            NodeDifference::MerkleTreeHash(base_graph_indexes) => {
                                updates.extend(self.detect_updates_for_node_index(
                                    updated_graph_index,
                                    base_graph_indexes,
                                ));
                            }
                        }

                        Control::Continue
                    }
                }
            }
            _ => Control::Continue,
        }
    }

    fn calculate_changes_dfs_event(
        &self,
        event: DfsEvent<NodeIndex>,
        changes: &mut Vec<Change>,
    ) -> Control<()> {
        if let DfsEvent::Discover(updated_graph_index, _) = event {
            match self.updated_graph.get_node_weight(updated_graph_index) {
                Ok(updated_node_weight) => {
                    if let Some(original_node_weight) = self.base_graph.get_node_weight_by_id_opt(updated_node_weight.id()) {
                        if original_node_weight.merkle_tree_hash() == updated_node_weight.merkle_tree_hash() {
                            return Control::Prune;
                        }
                    }

                    // If either the original node weight was not found or it was found the merkle tree hashes differ,
                    // then we have information that needs to be collected!
                    changes.push(Change {
                        id: updated_node_weight.id(),
                        merkle_tree_hash: updated_node_weight.merkle_tree_hash(),
                    });
                }
                Err(err) => error!(?err, "heat death of the universe error: updated node weight not found by updated node index from the same graph"),
            }
        }
        Control::Continue
    }
}
