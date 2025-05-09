use std::collections::{
    BTreeMap,
    HashMap,
    HashSet,
};

use petgraph::{
    prelude::*,
    visit::{
        Control,
        DfsEvent,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::Change;
use strum::EnumDiscriminants;

use crate::{
    CustomEdgeWeight,
    CustomNodeWeight,
    EdgeKind,
    SplitGraphEdgeWeight,
    SplitGraphEdgeWeightKind,
    SplitGraphNodeId,
    SplitGraphNodeWeight,
    subgraph::{
        SubGraph,
        SubGraphNodeIndex,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct ExternalSourceData<K>
where
    K: EdgeKind,
{
    pub source_id: SplitGraphNodeId,
    pub kind: K,
}

impl<K> ExternalSourceData<K>
where
    K: EdgeKind,
{
    pub fn source_id(&self) -> SplitGraphNodeId {
        self.source_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, EnumDiscriminants)]
pub enum Update<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    NewEdge {
        subgraph_index: usize,
        source: SplitGraphNodeId,
        destination: SplitGraphNodeId,
        edge_weight: SplitGraphEdgeWeight<E, K>,
    },
    RemoveEdge {
        subgraph_index: usize,
        source: SplitGraphNodeId,
        destination: SplitGraphNodeId,
        edge_kind: SplitGraphEdgeWeightKind<K>,
        external_source_data: Option<ExternalSourceData<K>>,
    },
    RemoveNode {
        subgraph_index: usize,
        id: SplitGraphNodeId,
    },
    ReplaceNode {
        subgraph_index: usize,
        base_graph_node_id: Option<SplitGraphNodeId>,
        node_weight: SplitGraphNodeWeight<N>,
    },
    NewNode {
        subgraph_index: usize,
        node_weight: SplitGraphNodeWeight<N>,
    },
    NewSubGraph,
}

#[derive(Clone, Debug)]
enum NodeDifference {
    NewNode,
    MerkleTreeHash(Vec<SubGraphNodeIndex>),
}

pub struct Detector<'a, 'b, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    updated_graph_index: usize,
    base_graph: &'a SubGraph<N, E, K>,
    updated_graph: &'b SubGraph<N, E, K>,
}

impl<'a, 'b, N, E, K> Detector<'a, 'b, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub fn new(
        base_graph: &'a SubGraph<N, E, K>,
        updated_graph: &'b SubGraph<N, E, K>,
        updated_graph_index: usize,
    ) -> Self {
        Self {
            updated_graph_index,
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
    pub fn detect_updates(&self) -> Vec<Update<N, E, K>> {
        let mut updates = vec![];
        let mut difference_cache = HashMap::new();

        petgraph::visit::depth_first_search(
            &self.updated_graph.graph,
            Some(self.updated_graph.root_index),
            |event| self.calculate_updates_dfs_event(event, &mut updates, &mut difference_cache),
        );

        // If a node is in base graph but not in updated_graph, it has been removed
        updates.extend(
            self.base_graph
                .node_index_by_id
                .keys()
                .filter(|id| !self.updated_graph.node_index_by_id.contains_key(id))
                .map(|id| Update::RemoveNode {
                    subgraph_index: self.updated_graph_index,
                    id: *id,
                }),
        );

        updates
    }

    pub fn detect_changes(&self) -> Vec<Change> {
        let mut changes = vec![];

        petgraph::visit::depth_first_search(
            &self.updated_graph.graph,
            Some(self.updated_graph.root_index),
            |event| {
                if let DfsEvent::Discover(updated_graph_index, _) = event {
                    let Some(updated_node_weight) =
                        self.updated_graph.graph.node_weight(updated_graph_index)
                    else {
                        return Control::Break(());
                    };

                    match updated_node_weight {
                        // Ordering node changes will impact the container node's merkle tree hash,
                        // and external target nodes will never change.
                        SplitGraphNodeWeight::ExternalTarget { .. }
                        | SplitGraphNodeWeight::Ordering { .. } => {
                            return Control::Prune;
                        }
                        _ => {}
                    }

                    let node_id = updated_node_weight.id();

                    if let Some(base_graph_weight) = self
                        .base_graph
                        .node_id_to_index(node_id)
                        .and_then(|index| self.base_graph.graph.node_weight(index))
                    {
                        if base_graph_weight.merkle_tree_hash()
                            == updated_node_weight.merkle_tree_hash()
                        {
                            return Control::Prune;
                        }
                    }

                    // We still want to prune if the subgraph root merkle tree hashes match.
                    // But we don't need to record the change, since subgraph roots only exist
                    // so the subgraph ... has a root. :)
                    if !matches!(
                        updated_node_weight,
                        SplitGraphNodeWeight::SubGraphRoot { .. }
                    ) {
                        changes.push(Change {
                            entity_id: node_id.into(),
                            entity_kind: updated_node_weight.entity_kind(),
                            merkle_tree_hash: updated_node_weight.merkle_tree_hash(),
                        });
                    }
                }
                Control::Continue
            },
        );

        changes
    }

    fn node_diff_from_base_graph(
        &self,
        updated_graph_node_index: SubGraphNodeIndex,
    ) -> Option<NodeDifference> {
        self.updated_graph
            .graph
            .node_weight(updated_graph_node_index)
            .and_then(|updated_graph_node_weight| {
                // dbg!(updated_graph_node_weight);
                let mut base_graph_node_indexes = HashSet::new();
                if updated_graph_node_index == self.updated_graph.root_index {
                    // There can only be one (valid/current) `ContentAddress::Root` at any
                    // given moment, and the `lineage_id` isn't really relevant as it's not
                    // globally stable (even though it is locally stable). This matters as we
                    // may be dealing with a `WorkspaceSnapshotGraph` that is coming to us
                    // externally from a module that we're attempting to import. The external
                    // `WorkspaceSnapshotGraph` will be `self`, and the "local" one will be
                    // `onto`.
                    base_graph_node_indexes.insert(self.base_graph.root_index);
                } else if let Some(new_base_graph_node_indexes) = self
                    .base_graph
                    .node_indexes_by_lineage_id
                    .get(&updated_graph_node_weight.lineage_id())
                {
                    base_graph_node_indexes.extend(new_base_graph_node_indexes);
                }

                base_graph_node_indexes
                    .is_empty()
                    .then_some(NodeDifference::NewNode)
                    .or_else(|| {
                        // If everything with the same `lineage_id` is identical, then
                        // we can prune the graph traversal, and avoid unnecessary
                        // lookups/comparisons.
                        let nodes_with_difference: Vec<SubGraphNodeIndex> = base_graph_node_indexes
                            .iter()
                            .filter_map(|&base_graph_index| {
                                self.base_graph
                                    .graph
                                    .node_weight(base_graph_index)
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

    fn calculate_updates_dfs_event(
        &self,
        event: DfsEvent<SubGraphNodeIndex>,
        updates: &mut Vec<Update<N, E, K>>,
        difference_cache: &mut HashMap<SubGraphNodeIndex, Option<NodeDifference>>,
    ) -> Control<()> {
        match event {
            DfsEvent::Discover(updated_graph_index, _) => {
                let node_diff = self.node_diff_from_base_graph(updated_graph_index);
                let control = match &node_diff {
                    Some(NodeDifference::NewNode) => {
                        if let Some(node_weight) =
                            self.updated_graph.graph.node_weight(updated_graph_index)
                        {
                            // NewNode updates are produced here, so that they
                            // are in the update array *before* the new edge
                            // updates which refer to them
                            updates.push(Update::NewNode {
                                subgraph_index: self.updated_graph_index,
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
                                        .graph
                                        .edges_directed(updated_graph_index, Outgoing)
                                        .map(|edge_ref| {
                                            (edge_ref.target(), edge_ref.weight().to_owned())
                                        })
                                        .filter_map(move |(target_index, edge_weight)| {
                                            if let Some((source_node, destination_node)) = self
                                                .updated_graph
                                                .graph
                                                .node_weight(updated_graph_index)
                                                .zip(
                                                    self.updated_graph
                                                        .graph
                                                        .node_weight(target_index),
                                                )
                                            {
                                                Some(Update::NewEdge {
                                                    subgraph_index: self.updated_graph_index,
                                                    source: source_node.id(),
                                                    destination: destination_node.id(),
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

    /// Produces ReplaceNode, NewEdge and RemoveEdge updates. The assumption we
    /// make here is that updated_graph has seen everything in base_graph. So if
    /// a node has a different hash from the matching one in base_graph, it has
    /// been changed and should be replaced. And if an edge is in base_graph but
    /// not in updated_graph, that means it's been removed. Finally, if an edge
    /// is in new_graph, but not in base_graph, that means it has been added.
    fn detect_updates_for_node_index(
        &self,
        updated_graph_node_index: SubGraphNodeIndex,
        base_graph_indexes: &[SubGraphNodeIndex],
    ) -> Vec<Update<N, E, K>> {
        #[derive(Debug, Clone)]
        struct EdgeInfo<E, K>
        where
            E: CustomEdgeWeight<K>,
            K: EdgeKind,
        {
            source_node: SplitGraphNodeId,
            target_node: SplitGraphNodeId,
            edge_weight: SplitGraphEdgeWeight<E, K>,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct UniqueEdgeInfo<K>
        where
            K: EdgeKind,
        {
            kind: SplitGraphEdgeWeightKind<K>,
            edge_entropy: Option<Vec<u8>>,
            external_source_data: Option<ExternalSourceData<K>>,
            target_lineage: SplitGraphNodeId,
        }

        let mut updates = vec![];

        let Some(updated_graph_node_weight) = self
            .updated_graph
            .graph
            .node_weight(updated_graph_node_index)
        else {
            return updates;
        };
        for base_graph_index in base_graph_indexes {
            let base_graph_index = *base_graph_index;

            if let Some(base_graph_node_weight) =
                self.base_graph.graph.node_weight(base_graph_index)
            {
                // if the node hash is different, then the node has been updated and
                // needs to be replaced in base_graph (node hash is a hash of the
                // content, which is not the same as the merkle tree hash, which
                // also gathers up the hashes of the outgoing neighbors).
                //
                // we have to also replace the node if the node id has changed.
                // Node ids are not captured in most of our node weight's node
                // hashes, unfortunately.
                if updated_graph_node_weight.node_hash() != base_graph_node_weight.node_hash()
                    || updated_graph_node_weight.id() != base_graph_node_weight.id()
                {
                    updates.push(Update::ReplaceNode {
                        subgraph_index: self.updated_graph_index,
                        base_graph_node_id: (base_graph_node_weight.id()
                            != updated_graph_node_weight.id())
                        .then_some(base_graph_node_weight.id()),
                        node_weight: updated_graph_node_weight.to_owned(),
                    });
                };

                let base_graph_edges: HashMap<UniqueEdgeInfo<K>, EdgeInfo<E, K>> = self
                    .base_graph
                    .graph
                    .edges_directed(base_graph_index, Outgoing)
                    .filter_map(|edge_ref| {
                        self.base_graph.graph.node_weight(edge_ref.target()).map(
                            |target_node_weight| {
                                (
                                    UniqueEdgeInfo {
                                        kind: edge_ref.weight().into(),
                                        edge_entropy: edge_ref.weight().edge_entropy(),
                                        external_source_data: match edge_ref.weight() {
                                            SplitGraphEdgeWeight::ExternalSource {
                                                source_id,
                                                edge_kind,
                                                ..
                                            } => Some(ExternalSourceData {
                                                source_id: *source_id,
                                                kind: *edge_kind,
                                            }),
                                            SplitGraphEdgeWeight::Custom(_)
                                            | SplitGraphEdgeWeight::Ordering
                                            | SplitGraphEdgeWeight::Ordinal => None,
                                        },
                                        target_lineage: target_node_weight.lineage_id(),
                                    },
                                    EdgeInfo {
                                        source_node: base_graph_node_weight.id(),
                                        target_node: target_node_weight.id(),
                                        edge_weight: edge_ref.weight().to_owned(),
                                    },
                                )
                            },
                        )
                    })
                    .collect();

                let update_graph_edges: HashMap<UniqueEdgeInfo<K>, EdgeInfo<E, K>> = self
                    .updated_graph
                    .graph
                    .edges_directed(updated_graph_node_index, Outgoing)
                    .filter_map(|edge_ref| {
                        self.updated_graph.graph.node_weight(edge_ref.target()).map(
                            |target_node_weight| {
                                (
                                    UniqueEdgeInfo {
                                        kind: edge_ref.weight().into(),
                                        edge_entropy: edge_ref.weight().edge_entropy(),
                                        external_source_data: match edge_ref.weight() {
                                            SplitGraphEdgeWeight::ExternalSource {
                                                source_id,
                                                edge_kind,
                                                ..
                                            } => Some(ExternalSourceData {
                                                source_id: *source_id,
                                                kind: *edge_kind,
                                            }),
                                            SplitGraphEdgeWeight::Custom(_)
                                            | SplitGraphEdgeWeight::Ordering
                                            | SplitGraphEdgeWeight::Ordinal => None,
                                        },
                                        target_lineage: target_node_weight.lineage_id(),
                                    },
                                    EdgeInfo {
                                        source_node: updated_graph_node_weight.id(),
                                        target_node: target_node_weight.id(),
                                        edge_weight: edge_ref.weight().to_owned(),
                                    },
                                )
                            },
                        )
                    })
                    .collect();

                updates.extend(
                    base_graph_edges
                        .iter()
                        .filter(|(edge_key, _)| !update_graph_edges.contains_key(edge_key))
                        .map(|(_, edge_info)| Update::RemoveEdge {
                            subgraph_index: self.updated_graph_index,
                            source: edge_info.source_node,
                            destination: edge_info.target_node,
                            edge_kind: (&edge_info.edge_weight).into(),
                            external_source_data: edge_info.edge_weight.external_source_data(),
                        }),
                );

                updates.extend(
                    update_graph_edges
                        .into_iter()
                        .filter(|(edge_key, _)| !base_graph_edges.contains_key(edge_key))
                        .map(|(_, edge_info)| Update::NewEdge {
                            subgraph_index: self.updated_graph_index,
                            source: edge_info.source_node,
                            destination: edge_info.target_node,
                            edge_weight: edge_info.edge_weight,
                        }),
                );
            }
        }

        updates
    }
}

/// Transforms a subgraph into NewNode and NewEdge updates
pub fn subgraph_as_updates<N, E, K>(
    subgraph: &SubGraph<N, E, K>,
    subgraph_index: usize,
) -> Vec<Update<N, E, K>>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    let mut updates = vec![];
    let mut node_index_to_id = BTreeMap::new();

    petgraph::visit::depth_first_search(&subgraph.graph, Some(subgraph.root_index), |event| {
        match event {
            DfsEvent::Discover(node_index, _) => {
                if let Some(node_weight) = subgraph.graph.node_weight(node_index).cloned() {
                    node_index_to_id.insert(node_index, node_weight.id());
                    updates.push(Update::NewNode {
                        subgraph_index,
                        node_weight,
                    })
                }
            }
            DfsEvent::Finish(node_index, _) => {
                updates.extend(
                    subgraph
                        .graph
                        .edges_directed(node_index, Outgoing)
                        .filter_map(|edge_ref| {
                            node_index_to_id
                                .get(&edge_ref.source())
                                .zip(node_index_to_id.get(&edge_ref.target()))
                                .map(|(&source, &destination)| Update::NewEdge {
                                    subgraph_index,
                                    source,
                                    destination,
                                    edge_weight: edge_ref.weight().to_owned(),
                                })
                        }),
                );
            }
            _ => {}
        }
    });

    updates
}
