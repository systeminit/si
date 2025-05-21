use std::{
    collections::{
        BTreeMap,
        HashMap,
        HashSet,
    },
    marker::PhantomData,
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
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::Change,
};
use strum::EnumDiscriminants;

use crate::{
    CustomEdgeWeight,
    CustomNodeWeight,
    EdgeKind,
    SplitGraphEdgeWeight,
    SplitGraphEdgeWeightKind,
    SplitGraphNodeId,
    SplitGraphNodeWeight,
    SplitGraphNodeWeightDiscriminants,
    subgraph::{
        SubGraph,
        SubGraphNodeIndex,
    },
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct ExternalSourceData<K, N>
where
    K: EdgeKind,
    N: CustomNodeWeight,
{
    pub source_id: SplitGraphNodeId,
    pub edge_kind: K,
    pub source_node_kind: Option<N::Kind>,

    #[serde(skip, default)]
    pub phantom_n: PhantomData<N>,
}

impl<K, N> ExternalSourceData<K, N>
where
    K: EdgeKind,
    N: CustomNodeWeight,
{
    pub fn source_id(&self) -> SplitGraphNodeId {
        self.source_id
    }

    pub fn source_edge_kind(&self) -> K {
        self.edge_kind
    }

    pub fn source_node_kind(&self) -> Option<N::Kind> {
        self.source_node_kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct UpdateNodeInfo<N>
where
    N: CustomNodeWeight,
{
    pub id: SplitGraphNodeId,
    pub external_target_id: Option<SplitGraphNodeId>,
    pub external_target_custom_kind: Option<N::Kind>,
    pub kind: SplitGraphNodeWeightDiscriminants,
    pub custom_kind: Option<N::Kind>,

    #[serde(skip)]
    pub phantom_n: PhantomData<N>,
}

impl<N> PartialOrd for UpdateNodeInfo<N>
where
    N: CustomNodeWeight,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<N> Ord for UpdateNodeInfo<N>
where
    N: CustomNodeWeight,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<N> From<&SplitGraphNodeWeight<N>> for UpdateNodeInfo<N>
where
    N: CustomNodeWeight,
{
    fn from(value: &SplitGraphNodeWeight<N>) -> Self {
        Self {
            id: value.id(),
            external_target_id: value.external_target_id(),
            external_target_custom_kind: value.external_target_custom_kind(),
            custom_kind: value.custom().map(|n| n.kind()),
            kind: value.into(),
            phantom_n: std::marker::PhantomData,
        }
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
        subgraph_root_id: SplitGraphNodeId,
        source: UpdateNodeInfo<N>,
        destination: UpdateNodeInfo<N>,
        edge_weight: SplitGraphEdgeWeight<E, K, N>,
    },
    RemoveEdge {
        subgraph_root_id: SplitGraphNodeId,
        source: UpdateNodeInfo<N>,
        destination: UpdateNodeInfo<N>,
        edge_kind: SplitGraphEdgeWeightKind<K>,
        external_source_data: Option<ExternalSourceData<K, N>>,
    },
    RemoveNode {
        subgraph_root_id: SplitGraphNodeId,
        id: SplitGraphNodeId,
    },
    ReplaceNode {
        subgraph_root_id: SplitGraphNodeId,
        base_graph_node_id: Option<SplitGraphNodeId>,
        node_weight: SplitGraphNodeWeight<N>,
    },
    NewNode {
        subgraph_root_id: SplitGraphNodeId,
        node_weight: SplitGraphNodeWeight<N>,
    },
    NewSubGraph {
        subgraph_root_id: SplitGraphNodeId,
    },
}

impl<N, E, K> Update<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    /// Produce a new edge update between two nodes. If the nodes are in different subgraphs,
    /// produce an edge from the source to an external target, and an external source edge
    /// from the target's subgraph root to the target.
    #[allow(clippy::too_many_arguments)]
    pub fn new_edge_between_nodes_updates(
        source_id: SplitGraphNodeId,
        source_subgraph_root_id: SplitGraphNodeId,
        source_kind: N::Kind,
        target_id: SplitGraphNodeId,
        target_subgraph_root_id: SplitGraphNodeId,
        target_kind: N::Kind,
        edge_weight: E,
        graph_root_id: SplitGraphNodeId,
    ) -> Vec<Self> {
        if source_subgraph_root_id == target_subgraph_root_id {
            vec![Update::NewEdge {
                subgraph_root_id: source_subgraph_root_id,
                source: UpdateNodeInfo {
                    id: source_id,
                    external_target_id: None,
                    external_target_custom_kind: None,
                    kind: SplitGraphNodeWeightDiscriminants::Custom,
                    custom_kind: Some(source_kind),
                    phantom_n: PhantomData,
                },
                destination: UpdateNodeInfo {
                    id: target_id,
                    external_target_id: None,
                    external_target_custom_kind: None,
                    kind: SplitGraphNodeWeightDiscriminants::Custom,
                    custom_kind: Some(target_kind),
                    phantom_n: PhantomData,
                },
                edge_weight: SplitGraphEdgeWeight::Custom(edge_weight),
            }]
        } else {
            let external_target_node_id = SplitGraphNodeId::new();
            let is_default = edge_weight.is_default();
            let edge_kind = edge_weight.kind();

            vec![
                Update::NewNode {
                    subgraph_root_id: source_subgraph_root_id,
                    node_weight: SplitGraphNodeWeight::ExternalTarget {
                        id: external_target_node_id,
                        target: target_id,
                        merkle_tree_hash: MerkleTreeHash::nil(),
                        target_kind: SplitGraphNodeWeightDiscriminants::Custom,
                        target_custom_kind: Some(target_kind),
                    },
                },
                Update::NewEdge {
                    subgraph_root_id: source_subgraph_root_id,
                    source: UpdateNodeInfo {
                        id: source_id,
                        external_target_id: None,
                        external_target_custom_kind: None,
                        kind: SplitGraphNodeWeightDiscriminants::Custom,
                        custom_kind: Some(source_kind),
                        phantom_n: PhantomData,
                    },
                    destination: UpdateNodeInfo {
                        id: external_target_node_id,
                        external_target_id: Some(target_id),
                        external_target_custom_kind: Some(target_kind),
                        kind: SplitGraphNodeWeightDiscriminants::ExternalTarget,
                        custom_kind: None,
                        phantom_n: PhantomData,
                    },
                    edge_weight: SplitGraphEdgeWeight::Custom(edge_weight),
                },
                Update::NewEdge {
                    subgraph_root_id: target_subgraph_root_id,
                    source: UpdateNodeInfo {
                        id: target_subgraph_root_id,
                        external_target_id: None,
                        external_target_custom_kind: None,
                        kind: if target_subgraph_root_id == graph_root_id {
                            SplitGraphNodeWeightDiscriminants::GraphRoot
                        } else {
                            SplitGraphNodeWeightDiscriminants::SubGraphRoot
                        },
                        custom_kind: None,
                        phantom_n: PhantomData,
                    },
                    destination: UpdateNodeInfo {
                        id: target_id,
                        external_target_id: None,
                        external_target_custom_kind: None,
                        kind: SplitGraphNodeWeightDiscriminants::Custom,
                        custom_kind: Some(target_kind),
                        phantom_n: PhantomData,
                    },
                    edge_weight: SplitGraphEdgeWeight::ExternalSource {
                        source_id,
                        is_default,
                        edge_kind,
                        source_node_kind: Some(source_kind),
                        phantom_n: PhantomData,
                    },
                },
            ]
        }
    }

    pub fn is_of_custom_edge_kind(&self, target_kind: K) -> bool {
        match self {
            Update::NewEdge { edge_weight, .. } => match edge_weight {
                SplitGraphEdgeWeight::Custom(c) => c.kind() == target_kind,
                SplitGraphEdgeWeight::ExternalSource { edge_kind, .. } => edge_kind == &target_kind,
                _ => false,
            },
            Update::RemoveEdge {
                edge_kind,
                external_source_data,
                ..
            } => match edge_kind {
                SplitGraphEdgeWeightKind::Custom(custom_kind) => custom_kind == &target_kind,
                SplitGraphEdgeWeightKind::ExternalSource => external_source_data
                    .as_ref()
                    .is_some_and(|esd| esd.edge_kind == target_kind),
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_edge_of_sort(
        &self,
        source_kind: N::Kind,
        edge_kind: K,
        destination_kind: N::Kind,
    ) -> bool {
        self.source_is_of_custom_node_kind(source_kind)
            && self.is_of_custom_edge_kind(edge_kind)
            && self.destination_is_of_custom_node_kind(destination_kind)
    }

    /// Returns true if the source node (across external source edges as well as same-graph) has a kind of `target_kind`.
    pub fn source_is_of_custom_node_kind(&self, target_kind: N::Kind) -> bool {
        match self {
            Update::NewEdge {
                source,
                edge_weight,
                ..
            } => {
                source.custom_kind == Some(target_kind)
                    || edge_weight
                        .external_source_data()
                        .as_ref()
                        .is_some_and(|esd| esd.source_node_kind().is_some_and(|k| k == target_kind))
            }
            Update::RemoveEdge {
                source,
                external_source_data,
                ..
            } => {
                source.custom_kind == Some(target_kind)
                    || external_source_data
                        .as_ref()
                        .is_some_and(|esd| esd.source_node_kind().is_some_and(|k| k == target_kind))
            }
            _ => false,
        }
    }

    /// Checks if the destination node of an edge update (RemoveEdge/NewEdge) has the given custom node kind, across subgraphs
    pub fn destination_is_of_custom_node_kind(&self, target_kind: N::Kind) -> bool {
        match self {
            Update::NewEdge { destination, .. } | Update::RemoveEdge { destination, .. } => {
                destination.custom_kind == Some(target_kind)
                    || destination
                        .external_target_custom_kind
                        .as_ref()
                        .is_some_and(|k| k == &target_kind)
            }
            _ => false,
        }
    }

    /// Checks if the source node of an edge update (RemoveEdge/NewEdge) has the given id, across subgraphs
    pub fn source_has_id(&self, id: SplitGraphNodeId) -> bool {
        match self {
            Update::NewEdge {
                source,
                edge_weight,
                ..
            } => {
                source.id == id
                    || edge_weight
                        .external_source_data()
                        .is_some_and(|esd| esd.source_id() == id)
            }
            Update::RemoveEdge {
                source,
                external_source_data,
                ..
            } => {
                source.id == id
                    || external_source_data
                        .as_ref()
                        .is_some_and(|esd| esd.source_id() == id)
            }
            _ => false,
        }
    }

    /// Gets the id of the source of a NewEdge/RemoveEdge update across graphs. That is, if this is an ExternalSource edge, we get the external source id, not the id of the subgraph root.
    pub fn source_id(&self) -> Option<SplitGraphNodeId> {
        match self {
            Update::NewEdge {
                source,
                edge_weight,
                ..
            } => Some(
                edge_weight
                    .external_source_data()
                    .map(|esd| esd.source_id())
                    .unwrap_or(source.id),
            ),
            Update::RemoveEdge {
                source,
                external_source_data,
                ..
            } => Some(
                external_source_data
                    .as_ref()
                    .map(|esd| esd.source_id())
                    .unwrap_or(source.id),
            ),
            _ => None,
        }
    }

    pub fn edge_endpoints(&self) -> Option<(SplitGraphNodeId, SplitGraphNodeId)> {
        self.source_id().zip(self.destination_id())
    }

    /// Checks if the destination node of an edge update (RemoveEdge/NewEdge) has the given id, across subgraphs
    pub fn destination_has_id(&self, id: SplitGraphNodeId) -> bool {
        match self {
            Update::NewEdge { destination, .. } | Update::RemoveEdge { destination, .. } => {
                destination.id == id || destination.external_target_id.is_some_and(|ext| ext == id)
            }
            _ => false,
        }
    }

    /// Gives the id of the destination of a NewEdge/RemoveEdge update across graphs. That is, if this is actually an edge to an ExternalTarget, we will return the *target* id, not the id of the ExternalTarget node.
    pub fn destination_id(&self) -> Option<SplitGraphNodeId> {
        match self {
            Update::NewEdge { destination, .. } | Update::RemoveEdge { destination, .. } => {
                Some(destination.external_target_id.unwrap_or(destination.id))
            }
            _ => None,
        }
    }
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
    graph_root_id: SplitGraphNodeId,
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
        graph_root_id: SplitGraphNodeId,
    ) -> Self {
        Self {
            graph_root_id,
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
                    subgraph_root_id: self.graph_root_id,
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
                                subgraph_root_id: self.graph_root_id,
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
                                                    subgraph_root_id: self.graph_root_id,
                                                    source: source_node.into(),
                                                    destination: destination_node.into(),
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
        struct EdgeInfo<N, E, K>
        where
            N: CustomNodeWeight,
            E: CustomEdgeWeight<K>,
            K: EdgeKind,
        {
            source_node: UpdateNodeInfo<N>,
            target_node: UpdateNodeInfo<N>,
            edge_weight: SplitGraphEdgeWeight<E, K, N>,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct UniqueEdgeInfo<K, N>
        where
            K: EdgeKind,
            N: CustomNodeWeight,
        {
            kind: SplitGraphEdgeWeightKind<K>,
            edge_entropy: Option<Vec<u8>>,
            external_source_data: Option<ExternalSourceData<K, N>>,
            target_lineage: SplitGraphNodeId,

            phantom_n: PhantomData<N>,
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
                        subgraph_root_id: self.graph_root_id,
                        base_graph_node_id: (base_graph_node_weight.id()
                            != updated_graph_node_weight.id())
                        .then_some(base_graph_node_weight.id()),
                        node_weight: updated_graph_node_weight.to_owned(),
                    });
                };

                let base_graph_edges: HashMap<UniqueEdgeInfo<K, N>, EdgeInfo<N, E, K>> = self
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
                                                source_node_kind,
                                                ..
                                            } => Some(ExternalSourceData {
                                                source_id: *source_id,
                                                edge_kind: *edge_kind,
                                                source_node_kind: *source_node_kind,

                                                phantom_n: PhantomData,
                                            }),
                                            SplitGraphEdgeWeight::Custom(_)
                                            | SplitGraphEdgeWeight::Ordering
                                            | SplitGraphEdgeWeight::Ordinal => None,
                                        },
                                        target_lineage: target_node_weight.lineage_id(),

                                        phantom_n: PhantomData,
                                    },
                                    EdgeInfo {
                                        source_node: base_graph_node_weight.into(),
                                        target_node: target_node_weight.into(),
                                        edge_weight: edge_ref.weight().to_owned(),
                                    },
                                )
                            },
                        )
                    })
                    .collect();

                let update_graph_edges: HashMap<UniqueEdgeInfo<K, N>, EdgeInfo<N, E, K>> = self
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
                                                source_node_kind,
                                                ..
                                            } => Some(ExternalSourceData {
                                                source_id: *source_id,
                                                edge_kind: *edge_kind,
                                                source_node_kind: *source_node_kind,

                                                phantom_n: PhantomData,
                                            }),
                                            SplitGraphEdgeWeight::Custom(_)
                                            | SplitGraphEdgeWeight::Ordering
                                            | SplitGraphEdgeWeight::Ordinal => None,
                                        },
                                        target_lineage: target_node_weight.lineage_id(),

                                        phantom_n: PhantomData,
                                    },
                                    EdgeInfo {
                                        source_node: updated_graph_node_weight.into(),
                                        target_node: target_node_weight.into(),
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
                            subgraph_root_id: self.graph_root_id,
                            source: edge_info.source_node.clone(),
                            destination: edge_info.target_node.clone(),
                            edge_kind: (&edge_info.edge_weight).into(),
                            external_source_data: edge_info.edge_weight.external_source_data(),
                        }),
                );

                updates.extend(
                    update_graph_edges
                        .into_iter()
                        .filter(|(edge_key, _)| !base_graph_edges.contains_key(edge_key))
                        .map(|(_, edge_info)| Update::NewEdge {
                            subgraph_root_id: self.graph_root_id,
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
    subgraph_root_id: SplitGraphNodeId,
) -> Vec<Update<N, E, K>>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    let mut updates = vec![];
    let mut node_index_to_info: BTreeMap<NodeIndex<usize>, UpdateNodeInfo<N>> = BTreeMap::new();

    petgraph::visit::depth_first_search(&subgraph.graph, Some(subgraph.root_index), |event| {
        match event {
            DfsEvent::Discover(node_index, _) => {
                if let Some(node_weight) = subgraph.graph.node_weight(node_index).cloned() {
                    node_index_to_info.insert(node_index, (&node_weight).into());
                    updates.push(Update::NewNode {
                        subgraph_root_id,
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
                            node_index_to_info
                                .get(&edge_ref.source())
                                .zip(node_index_to_info.get(&edge_ref.target()))
                                .map(|(source, destination)| Update::NewEdge {
                                    subgraph_root_id,
                                    source: source.clone(),
                                    destination: destination.clone(),
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
