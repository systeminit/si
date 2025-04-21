use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_events::merkle_tree_hash::MerkleTreeHash;
use std::collections::{HashMap, HashSet};
use std::io::Write;

use crate::{
    CustomEdgeWeight, CustomNodeWeight, EdgeKind, MAX_NODES, SplitGraphEdgeWeight,
    SplitGraphEdgeWeightKind, SplitGraphError, SplitGraphNodeId, SplitGraphNodeWeight,
    SplitGraphResult,
};

pub type SubGraphNodeIndex = NodeIndex<u16>;
pub type SubGraphEdgeIndex = EdgeIndex<u16>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub(crate) graph: StableDiGraph<SplitGraphNodeWeight<N>, SplitGraphEdgeWeight<E, K>, u16>,
    pub(crate) node_index_by_id: HashMap<SplitGraphNodeId, SubGraphNodeIndex>,
    pub(crate) node_indexes_by_lineage_id: HashMap<SplitGraphNodeId, HashSet<SubGraphNodeIndex>>,
    pub(crate) root_index: SubGraphNodeIndex,

    #[serde(skip)]
    pub(crate) touched_nodes: HashSet<SubGraphNodeIndex>,
}

impl<N, E, K> Default for SubGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E, K> SubGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub(crate) fn new() -> Self {
        Self {
            graph: StableDiGraph::with_capacity(MAX_NODES, MAX_NODES * 2),
            node_index_by_id: HashMap::new(),
            node_indexes_by_lineage_id: HashMap::new(),
            root_index: NodeIndex::new(0),

            touched_nodes: HashSet::new(),
        }
    }

    pub(crate) fn new_with_root() -> Self {
        let mut subgraph = Self {
            graph: StableDiGraph::with_capacity(MAX_NODES, MAX_NODES * 2),
            node_index_by_id: HashMap::new(),
            node_indexes_by_lineage_id: HashMap::new(),
            root_index: NodeIndex::new(0),

            touched_nodes: HashSet::new(),
        };

        let root_id = SplitGraphNodeId::new();
        let root_index = subgraph.graph.add_node(SplitGraphNodeWeight::SubGraphRoot {
            id: root_id,
            merkle_tree_hash: MerkleTreeHash::nil(),
        });
        subgraph.node_index_by_id.insert(root_id, root_index);
        subgraph.root_index = root_index;

        subgraph
    }

    pub(crate) fn cleanup(&mut self) {
        loop {
            let orphaned_node_indexes: Vec<SubGraphNodeIndex> = self
                .graph
                .externals(Incoming)
                .filter(|idx| *idx != self.root_index)
                .collect();

            if orphaned_node_indexes.is_empty() {
                break;
            }

            for node_index in orphaned_node_indexes {
                self.graph.remove_node(node_index);
            }
        }

        self.node_index_by_id
            .retain(|_id, index| self.graph.node_weight(*index).is_some());
        self.node_indexes_by_lineage_id
            .iter_mut()
            .for_each(|(_, node_indexes)| {
                node_indexes.retain(|index| self.graph.node_weight(*index).is_some());
            });
        self.node_indexes_by_lineage_id
            .retain(|_, indexes| !indexes.is_empty());
    }

    pub(crate) fn add_node(&mut self, node: SplitGraphNodeWeight<N>) -> SubGraphNodeIndex {
        let node_id = node.id();
        let node_index = self.graph.add_node(node);
        self.node_index_by_id.insert(node_id, node_index);
        self.node_indexes_by_lineage_id
            .entry(node_id)
            .and_modify(|set| {
                set.insert(node_index);
            })
            .or_insert(HashSet::from([node_index]));
        self.touched_nodes.insert(node_index);

        node_index
    }

    pub(crate) fn replace_node(&mut self, index: SubGraphNodeIndex, node: SplitGraphNodeWeight<N>) {
        if let Some(node_ref) = self.graph.node_weight_mut(index) {
            *node_ref = node;
        }
        self.touched_nodes.insert(index);
    }

    fn edge_exists(
        &self,
        from_index: SubGraphNodeIndex,
        edge_weight: &SplitGraphEdgeWeight<E, K>,
        to_index: SubGraphNodeIndex,
    ) -> bool {
        self.graph
            .edges_connecting(from_index, to_index)
            .any(|edge_ref| match edge_ref.weight() {
                SplitGraphEdgeWeight::Custom(custom_edge) => {
                    Some(custom_edge.kind()) == edge_weight.custom().map(|e| e.kind())
                }
                SplitGraphEdgeWeight::ExternalSource {
                    source_id,
                    edge_kind,
                    ..
                } => match &edge_weight {
                    SplitGraphEdgeWeight::ExternalSource {
                        source_id: new_source_id,
                        edge_kind: new_edge_kind,
                        ..
                    } => source_id == new_source_id && edge_kind == new_edge_kind,
                    _ => false,
                },
                SplitGraphEdgeWeight::Ordering => {
                    matches!(edge_weight, SplitGraphEdgeWeight::Ordering)
                }
                SplitGraphEdgeWeight::Ordinal => {
                    matches!(edge_weight, SplitGraphEdgeWeight::Ordinal)
                }
            })
    }

    pub(crate) fn ordering_node_for_node_index(
        &self,
        node_index: SubGraphNodeIndex,
    ) -> Option<SubGraphNodeIndex> {
        let Some(true) = self
            .graph
            .node_weight(node_index)
            .and_then(|weight| weight.custom().map(|c| c.ordered()))
        else {
            return None;
        };

        let ordering_node_index = self
            .graph
            .edges_directed(node_index, Outgoing)
            .find(|edge_ref| matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordering))
            .map(|edge_ref| edge_ref.target())?;

        if let SplitGraphNodeWeight::Ordering { .. } =
            self.graph.node_weight(ordering_node_index)?
        {
            Some(ordering_node_index)
        } else {
            None
        }
    }

    pub(crate) fn reorder_node<L>(
        &mut self,
        node_index: SubGraphNodeIndex,
        lambda: L,
    ) -> SplitGraphResult<()>
    where
        L: FnOnce(&[SplitGraphNodeId]) -> Vec<SplitGraphNodeId>,
    {
        let Some(ordering_node_index) = self.ordering_node_for_node_index(node_index) else {
            return Ok(());
        };

        let Some(SplitGraphNodeWeight::Ordering { order, .. }) =
            self.graph.node_weight_mut(ordering_node_index)
        else {
            return Ok(());
        };

        let new_order = lambda(order.as_slice());

        // Validate the return here to prevent a panic in copy from slice, and to prevent removal of ordered children
        if new_order.len() != order.len() {
            return Err(SplitGraphError::OrderLengthMismatch);
        }
        for id in order.iter() {
            if !new_order.contains(id) {
                return Err(SplitGraphError::OrderContentMismatch);
            }
        }

        order.copy_from_slice(new_order.as_slice());

        self.touch_node(ordering_node_index);

        Ok(())
    }

    pub(crate) fn ordered_children(
        &self,
        node_index: SubGraphNodeIndex,
    ) -> Option<Vec<SubGraphNodeIndex>> {
        let ordering_node_index = self.ordering_node_for_node_index(node_index)?;

        let SplitGraphNodeWeight::Ordering { order, .. } =
            self.graph.node_weight(ordering_node_index)?
        else {
            return None;
        };

        Some(
            order
                .iter()
                .filter_map(|id| self.node_index_by_id.get(id).copied())
                .collect(),
        )
    }

    #[allow(unused)]
    pub(crate) fn root_node_merkle_tree_hash(&self) -> MerkleTreeHash {
        self.graph
            .node_weight(self.root_index)
            .map(|node| node.merkle_tree_hash())
            .unwrap_or(MerkleTreeHash::nil())
    }

    pub(crate) fn recalculate_entire_merkle_tree_hash(&mut self) {
        let mut dfs = petgraph::visit::DfsPostOrder::new(&self.graph, self.root_index);

        while let Some(node_index) = dfs.next(&self.graph) {
            if let Some(hash) = self.calculate_merkle_hash_for_node(node_index) {
                if let Some(node_weight_mut) = self.graph.node_weight_mut(node_index) {
                    node_weight_mut.set_merkle_tree_hash(hash);
                }
            }
        }
    }

    pub(crate) fn recalculate_merkle_tree_hash_based_on_touched_nodes(&mut self) {
        let mut dfs = petgraph::visit::DfsPostOrder::new(&self.graph, self.root_index);

        let mut discovered_nodes = HashSet::new();

        while let Some(node_index) = dfs.next(&self.graph) {
            if self.touched_nodes.contains(&node_index) || discovered_nodes.contains(&node_index) {
                if let Some(hash) = self.calculate_merkle_hash_for_node(node_index) {
                    if let Some(node_weight_mut) = self.graph.node_weight_mut(node_index) {
                        node_weight_mut.set_merkle_tree_hash(hash);
                    }
                }
                self.graph
                    .neighbors_directed(node_index, Incoming)
                    .for_each(|node_idx| {
                        discovered_nodes.insert(node_idx);
                    });
            }
        }

        self.touched_nodes.clear();
    }

    pub(crate) fn all_outgoing_stably_ordered(
        &self,
        node_index: SubGraphNodeIndex,
    ) -> Vec<SubGraphNodeIndex> {
        let ordered_children = self.ordered_children(node_index).unwrap_or_default();
        let mut unordered_children: Vec<(_, _)> = self
            .graph
            .neighbors_directed(node_index, Outgoing)
            .filter(|child_idx| !ordered_children.contains(child_idx))
            .filter_map(|child_idx| {
                self.graph
                    .node_weight(child_idx)
                    .map(|weight| (weight.id(), child_idx))
            })
            .collect();

        // We want to keep the "unordered" children stably sorted as well, so that we get the same hash every time if there are no changes
        unordered_children.sort_by_cached_key(|(id, _)| *id);
        let mut all_children =
            Vec::with_capacity(ordered_children.len() + unordered_children.len());
        all_children.extend(ordered_children);
        all_children.extend(unordered_children.into_iter().map(|(_, index)| index));

        all_children
    }

    fn calculate_merkle_hash_for_node(
        &self,
        node_index: SubGraphNodeIndex,
    ) -> Option<MerkleTreeHash> {
        let mut hasher = MerkleTreeHash::hasher();
        hasher.update(self.graph.node_weight(node_index)?.node_hash().as_bytes());

        for child_idx in self.all_outgoing_stably_ordered(node_index) {
            hasher.update(
                self.graph
                    .node_weight(child_idx)?
                    .merkle_tree_hash()
                    .as_bytes(),
            );

            for edge_ref in self.graph.edges_connecting(node_index, child_idx) {
                if let Some(edge_hash) = edge_ref.weight().edge_hash() {
                    hasher.update(edge_hash.as_bytes());
                }
            }
        }

        Some(hasher.finalize())
    }

    pub(crate) fn node_id_to_index(&self, id: SplitGraphNodeId) -> Option<SubGraphNodeIndex> {
        self.node_index_by_id.get(&id).copied()
    }

    /// Adds a SplitGraphEdgeWeight if one of the exact same kind does not exist between `from_index`
    /// and `to_index` and touches `from_index` so that the merkle tree hash will be recalculated.
    pub(crate) fn add_edge_raw(
        &mut self,
        from_index: SubGraphNodeIndex,
        edge_weight: SplitGraphEdgeWeight<E, K>,
        to_index: SubGraphNodeIndex,
    ) {
        if !self.edge_exists(from_index, &edge_weight, to_index) {
            self.graph.add_edge(from_index, to_index, edge_weight);
            self.touch_node(from_index);
        }
    }

    pub(crate) fn remove_node(&mut self, node_index: SubGraphNodeIndex) {
        let parents: Vec<_> = self
            .graph
            .neighbors_directed(node_index, Incoming)
            .collect();

        self.graph.remove_node(node_index);
        parents
            .into_iter()
            .for_each(|parent_idx| self.touch_node(parent_idx));
    }

    /// Add an edge between `from_index` and `to_index` if the edge does not exist.
    /// Handles the creation of ordering nodes and the ordering edges if the node at
    /// `from_index` is an ordered container.
    pub(crate) fn add_edge(
        &mut self,
        from_index: SubGraphNodeIndex,
        edge_weight: SplitGraphEdgeWeight<E, K>,
        to_index: SubGraphNodeIndex,
    ) -> SplitGraphResult<()> {
        let is_ordered_container = self
            .graph
            .node_weight(from_index)
            .and_then(|weight| weight.custom().map(|c| c.ordered()))
            .is_some_and(|ordered| ordered);

        if is_ordered_container {
            let target_id = self
                .graph
                .node_weight(to_index)
                .map(|n| n.id())
                .ok_or(SplitGraphError::NodeNotFoundAtIndex)?;

            let ordering_node_index = match self
                .graph
                .edges_directed(from_index, Outgoing)
                .find(|edge_ref| matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordering))
                .map(|edge_ref| edge_ref.target())
            {
                Some(target) => target,
                None => {
                    let new_ordering_node_id = SplitGraphNodeId::new();
                    let ordering_node_index = self.graph.add_node(SplitGraphNodeWeight::Ordering {
                        id: new_ordering_node_id,
                        order: vec![],
                        merkle_tree_hash: MerkleTreeHash::nil(),
                    });

                    self.node_index_by_id
                        .insert(new_ordering_node_id, ordering_node_index);

                    self.add_edge_raw(
                        from_index,
                        SplitGraphEdgeWeight::Ordering,
                        ordering_node_index,
                    );

                    ordering_node_index
                }
            };

            if let Some(SplitGraphNodeWeight::Ordering { order, .. }) =
                self.graph.node_weight_mut(ordering_node_index)
            {
                if !order.contains(&target_id) {
                    order.push(target_id);
                }
                self.add_edge_raw(ordering_node_index, SplitGraphEdgeWeight::Ordinal, to_index);
            }
        }

        self.add_edge_raw(from_index, edge_weight, to_index);

        Ok(())
    }

    pub(crate) fn touch_node(&mut self, node_index: SubGraphNodeIndex) {
        self.touched_nodes.insert(node_index);
    }

    /// Removes all edges between `from_index` and `to_index` that match the passed in kind.
    /// Also handles removing any correspond
    pub(crate) fn remove_edge_raw(
        &mut self,
        from_index: SubGraphNodeIndex,
        kind: SplitGraphEdgeWeightKind<K>,
        to_index: SubGraphNodeIndex,
    ) {
        let edge_indexes: Vec<_> = self
            .graph
            .edges_directed(from_index, Outgoing)
            .filter(|edge_ref| kind == edge_ref.weight().into() && edge_ref.target() == to_index)
            .map(|edge_ref| edge_ref.id())
            .collect();
        for edge_index in edge_indexes {
            self.graph.remove_edge(edge_index);
        }
    }

    pub(crate) fn remove_from_order(
        &mut self,
        ordering_node_index: SubGraphNodeIndex,
        item_id: SplitGraphNodeId,
    ) {
        if let Some(SplitGraphNodeWeight::Ordering { order, .. }) =
            self.graph.node_weight_mut(ordering_node_index)
        {
            order.retain(|id| *id != item_id);
        }
    }

    /// Removes the edge specified by `edge_index`. Also handles edges to and
    /// from the ordering node, if one exists for `from_index`, and removes
    /// the target from the order.
    pub(crate) fn remove_edge_by_index(&mut self, edge_index: EdgeIndex<u16>) {
        if let Some((from_index, to_index)) = self.graph.edge_endpoints(edge_index) {
            self.touch_node(from_index);

            let is_ordered_container = self
                .graph
                .node_weight(from_index)
                .and_then(|weight| weight.custom().map(|c| c.ordered()))
                .is_some_and(|ordered| ordered);

            if is_ordered_container {
                let target_id = self.graph.node_weight(to_index).map(|n| n.id()).unwrap();
                if let Some(ordering_node_index) = self
                    .graph
                    .edges_directed(from_index, Outgoing)
                    .find(|edge_ref| matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordering))
                    .map(|edge_ref| edge_ref.target())
                {
                    self.touch_node(ordering_node_index);
                    if let Some(ordinal_edge_index) = self
                        .graph
                        .edges_directed(ordering_node_index, Outgoing)
                        .find(|edge_ref| {
                            matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordinal)
                                && self.graph.node_weight(edge_ref.target()).map(|n| n.id())
                                    == Some(target_id)
                        })
                        .map(|edge_ref| edge_ref.id())
                    {
                        self.graph.remove_edge(ordinal_edge_index);
                        self.remove_from_order(ordering_node_index, target_id);
                    }
                }
            }

            self.graph.remove_edge(edge_index);
        }
    }

    pub(crate) fn tiny_dot_to_file(&self, name: &str) {
        let dot = petgraph::dot::Dot::with_attr_getters(
            &self.graph,
            &[
                petgraph::dot::Config::NodeNoLabel,
                petgraph::dot::Config::EdgeNoLabel,
            ],
            &|_, edge_ref| {
                let (label, color) = match edge_ref.weight() {
                    SplitGraphEdgeWeight::Custom(_) => ("".into(), "black"),
                    SplitGraphEdgeWeight::ExternalSource {
                        source_id,
                        subgraph,
                        ..
                    } => (
                        format!("external source: {source_id}\nsubgraph: {}", subgraph + 1),
                        "red",
                    ),
                    SplitGraphEdgeWeight::Ordering => ("ordering".into(), "green"),
                    SplitGraphEdgeWeight::Ordinal => ("ordinal".into(), "green"),
                };

                format!("label = \"{label}\"\ncolor = {color}")
            },
            &|_, (_, node_weight)| {
                let (label, color) = match node_weight {
                    SplitGraphNodeWeight::Custom(n) => {
                        let node_dbg = format!("{n:?}")
                            .replace("\"", "'")
                            .replace("{", "{\n")
                            .replace("}", "\n}");
                        (format!("node: {}\n{node_dbg}", n.id()), "black")
                    }
                    SplitGraphNodeWeight::ExternalTarget {
                        target, subgraph, ..
                    } => (
                        format!("external target: {target}\nsubgraph: {}", subgraph + 1),
                        "red",
                    ),
                    SplitGraphNodeWeight::GraphRoot { id, .. } => {
                        (format!("graph root: {id}"), "blue")
                    }
                    SplitGraphNodeWeight::SubGraphRoot { id, .. } => {
                        (format!("subgraph root: {id}"), "blue")
                    }
                    SplitGraphNodeWeight::Ordering { id, .. } => {
                        (format!("ordering: {id}"), "green")
                    }
                };

                format!("label = \"{label}\"\ncolor = {color}")
            },
        );

        #[allow(clippy::disallowed_methods)]
        let home_str = std::env::var("HOME").expect("could not find home directory via env");
        let home = std::path::Path::new(&home_str);

        let mut file =
            std::fs::File::create(home.join(format!("{name}.txt"))).expect("could not create file");
        file.write_all(format!("{dot:?}").as_bytes())
            .expect("could not write file");
    }
}
