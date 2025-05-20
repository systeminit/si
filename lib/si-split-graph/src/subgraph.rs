use std::{
    collections::{
        BTreeMap,
        HashMap,
        HashSet,
    },
    io::Write,
    time::Instant,
};

use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::merkle_tree_hash::MerkleTreeHash;
use telemetry::prelude::*;

use crate::{
    CustomEdgeWeight,
    CustomNodeWeight,
    EdgeKind,
    SplitGraphEdgeWeight,
    SplitGraphEdgeWeightKind,
    SplitGraphError,
    SplitGraphNodeId,
    SplitGraphNodeWeight,
    SplitGraphResult,
    updates::ExternalSourceData,
};

pub type SubGraphNodeIndex = NodeIndex<usize>;
pub type SubGraphEdgeIndex = EdgeIndex<usize>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub(crate) graph: StableDiGraph<SplitGraphNodeWeight<N>, SplitGraphEdgeWeight<E, K, N>, usize>,
    pub(crate) node_index_by_id: BTreeMap<SplitGraphNodeId, SubGraphNodeIndex>,
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
            graph: StableDiGraph::with_capacity(32768, 32768 * 2),
            node_index_by_id: BTreeMap::new(),
            node_indexes_by_lineage_id: HashMap::new(),
            root_index: NodeIndex::new(0),

            touched_nodes: HashSet::new(),
        }
    }

    pub fn graph(
        &self,
    ) -> &StableDiGraph<SplitGraphNodeWeight<N>, SplitGraphEdgeWeight<E, K, N>, usize> {
        &self.graph
    }

    pub fn root_id(&self) -> Option<SplitGraphNodeId> {
        self.graph
            .node_weight(self.root_index)
            .map(|node| node.id())
    }

    pub(crate) fn new_with_root() -> Self {
        let mut subgraph = Self {
            graph: StableDiGraph::with_capacity(32768, 32768 * 2),
            node_index_by_id: BTreeMap::new(),
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

    /// Remove any nodes with no incoming edges from the graph, returning the ids of removed nodes.
    /// Note that this does not automatically cascade to remove nodes that were orphaned by the
    /// removal of the first "layer" of orphaned nodes. This function is intended to be used
    /// by the `SiSplitGraph::cleanup` call to remove orphaned nodes along with ExternalSource
    /// edges in *other* subgraphs that might point to the orphaned nodes.
    /// That method will call this in a loop until no orphaned nodes remain.
    pub(crate) fn remove_externals(&mut self) -> Vec<SplitGraphNodeId> {
        let mut removed_ids = vec![];
        let mut indexes_to_remove = vec![];
        for external in self
            .graph
            .externals(Incoming)
            .filter(|idx| *idx != self.root_index)
        {
            if let Some(node_id) = self.graph.node_weight(external).map(|node| node.id()) {
                removed_ids.push(node_id);
            }
            indexes_to_remove.push(external);
        }

        for index in indexes_to_remove {
            self.graph.remove_node(index);
        }

        removed_ids
    }

    pub fn node_weight(&self, node_id: SplitGraphNodeId) -> Option<&SplitGraphNodeWeight<N>> {
        self.node_id_to_index(node_id)
            .and_then(|index| self.graph.node_weight(index))
    }

    pub fn cleanup_maps(&mut self) {
        self.node_index_by_id
            .retain(|_, index| self.graph.node_weight(*index).is_some());
        self.node_indexes_by_lineage_id
            .iter_mut()
            .for_each(|(_, node_indexes)| {
                node_indexes.retain(|index| self.graph.node_weight(*index).is_some());
            });
        self.node_indexes_by_lineage_id
            .retain(|_, indexes| !indexes.is_empty());
    }

    pub(crate) fn add_ids_to_indexes(
        &mut self,
        node_id: SplitGraphNodeId,
        lineage_id: SplitGraphNodeId,
        node_index: SubGraphNodeIndex,
    ) {
        self.node_index_by_id.insert(node_id, node_index);
        self.node_indexes_by_lineage_id
            .entry(lineage_id)
            .and_modify(|set| {
                set.insert(node_index);
            })
            .or_insert(HashSet::from([node_index]));
    }

    pub(crate) fn remove_ids_from_indexes(
        &mut self,
        node_id: SplitGraphNodeId,
        lineage_id: SplitGraphNodeId,
    ) {
        // println!("removing {:?} from indexes", node_id);
        let node_index = self.node_index_by_id.remove(&node_id);

        if let Some(node_index) = node_index {
            if let Some(lineage_indexes) = self.node_indexes_by_lineage_id.get_mut(&lineage_id) {
                lineage_indexes.retain(|idx| *idx != node_index);
            }
        }
    }

    pub(crate) fn add_node(&mut self, node: SplitGraphNodeWeight<N>) -> SubGraphNodeIndex {
        let node_id = node.id();
        let lineage_id = node.lineage_id();
        let node_index = self.graph.add_node(node);
        self.add_ids_to_indexes(node_id, lineage_id, node_index);
        self.touched_nodes.insert(node_index);

        node_index
    }

    pub(crate) fn replace_node(
        &mut self,
        index: SubGraphNodeIndex,
        node: SplitGraphNodeWeight<N>,
    ) -> Option<SplitGraphNodeId> {
        let new_node_id = node.id();
        let new_lineage_id = node.lineage_id();
        let previous_ids = match self.graph.node_weight_mut(index) {
            Some(existing_node_ref) => {
                let node_id = existing_node_ref.id();
                let lineage_id = existing_node_ref.id();
                *existing_node_ref = node;
                Some((node_id, lineage_id))
            }
            None => None,
        };

        if let Some((previous_node_id, previous_lineage_id)) = previous_ids {
            if previous_node_id != new_node_id {
                self.remove_ids_from_indexes(previous_node_id, previous_lineage_id);
                self.add_ids_to_indexes(new_node_id, new_lineage_id, index);
            }
        }

        self.touched_nodes.insert(index);

        previous_ids.map(|(node_id, _)| node_id)
    }

    fn edge_exists(
        &self,
        from_index: SubGraphNodeIndex,
        edge_weight: &SplitGraphEdgeWeight<E, K, N>,
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

    pub fn root_node_merkle_tree_hash(&self) -> MerkleTreeHash {
        self.graph
            .node_weight(self.root_index)
            .map(|node| node.merkle_tree_hash())
            .unwrap_or(MerkleTreeHash::nil())
    }

    pub(crate) fn recalculate_entire_merkle_tree_hash(&mut self) {
        let mut dfs = petgraph::visit::DfsPostOrder::new(&self.graph, self.root_index);

        while let Some(node_index) = dfs.next(&self.graph) {
            if let Some((hash, _)) = self.calculate_merkle_hash_for_node(node_index) {
                if let Some(node_weight_mut) = self.graph.node_weight_mut(node_index) {
                    node_weight_mut.set_merkle_tree_hash(hash);
                }
            }
        }
    }

    pub(crate) fn recalculate_merkle_tree_hash_based_on_touched_nodes(&mut self) -> (usize, usize) {
        let mut node_count = 0;
        let mut edge_count = 0;
        let big_start = Instant::now();
        let mut dfs = petgraph::visit::DfsPostOrder::new(&self.graph, self.root_index);

        let mut discovered_nodes = HashSet::new();

        if self.touched_nodes.is_empty() {
            warn!(
                "touched nodes empty. merkle tree hash calculated in {:?}",
                big_start.elapsed()
            );
            return (0, 0);
        }

        while let Some(node_index) = dfs.next(&self.graph) {
            node_count += 1;
            if self.touched_nodes.contains(&node_index) || discovered_nodes.contains(&node_index) {
                if let Some((hash, edges_hashed)) = self.calculate_merkle_hash_for_node(node_index)
                {
                    edge_count += edges_hashed;
                    if let Some(node_weight_mut) = self.graph.node_weight_mut(node_index) {
                        node_weight_mut.set_merkle_tree_hash(hash);
                    }
                }
                for incoming_source_idx in self.graph.neighbors_directed(node_index, Incoming) {
                    discovered_nodes.insert(incoming_source_idx);
                }
            }
        }

        self.touched_nodes.clear();
        warn!("merkle tree hash calculated in {:?}", big_start.elapsed());

        (node_count, edge_count)
    }

    pub(crate) fn all_outgoing_stably_ordered(
        &self,
        node_index: SubGraphNodeIndex,
    ) -> Vec<(&SplitGraphEdgeWeight<E, K, N>, SubGraphNodeIndex)> {
        let ordered_children = self.ordered_children(node_index).unwrap_or_default();
        let ordered_children_edges: Vec<(_, _)> = ordered_children
            .iter()
            .flat_map(|child_index| {
                self.graph
                    .edges_connecting(node_index, *child_index)
                    .map(|edge_ref| (edge_ref.weight(), *child_index))
            })
            .collect();

        let mut unordered_children: Vec<(_, _, _)> = self
            .graph
            .edges_directed(node_index, Outgoing)
            .filter(|edge_ref| !ordered_children.contains(&edge_ref.target()))
            .filter_map(|edge_ref| {
                self.graph
                    .node_weight(edge_ref.target())
                    .map(|weight| (weight.id(), edge_ref.weight(), edge_ref.target()))
            })
            .collect();

        // We want to keep the "unordered" children stably sorted as well,
        // so that we get the same hash every time if there are no changes
        unordered_children.sort_by_cached_key(|(id, _, _)| *id);
        let mut all_children =
            Vec::with_capacity(ordered_children.len() + unordered_children.len());
        all_children.extend(ordered_children_edges);
        all_children.extend(
            unordered_children
                .into_iter()
                .map(|(_, weight, index)| (weight, index)),
        );

        all_children
    }

    fn calculate_merkle_hash_for_node(
        &self,
        node_index: SubGraphNodeIndex,
    ) -> Option<(MerkleTreeHash, usize)> {
        let mut edge_count = 0;
        let mut hasher = MerkleTreeHash::hasher();
        let node_weight = self.graph.node_weight(node_index)?;
        hasher.update(node_weight.node_hash().as_bytes());
        hasher.update(&node_weight.id().inner().to_bytes());

        let all_outgoing_stably_ordered = self.all_outgoing_stably_ordered(node_index);

        let mut hashed_children = HashSet::new();
        for (edge_weight, child_idx) in all_outgoing_stably_ordered {
            edge_count += 1;
            if !hashed_children.contains(&child_idx) {
                hasher.update(
                    self.graph
                        .node_weight(child_idx)?
                        .merkle_tree_hash()
                        .as_bytes(),
                );
                hashed_children.insert(child_idx);
            }
            if let Some(edge_entropy) = edge_weight.edge_entropy() {
                hasher.update(edge_entropy.as_slice());
            }
        }

        Some((hasher.finalize(), edge_count))
    }

    pub(crate) fn node_id_to_index(&self, id: SplitGraphNodeId) -> Option<SubGraphNodeIndex> {
        self.node_index_by_id.get(&id).copied()
    }

    /// Adds a SplitGraphEdgeWeight if one of the exact same kind does not exist between `from_index`
    /// and `to_index` and touches `from_index` so that the merkle tree hash will be recalculated.
    pub(crate) fn add_edge_raw(
        &mut self,
        from_index: SubGraphNodeIndex,
        edge_weight: SplitGraphEdgeWeight<E, K, N>,
        to_index: SubGraphNodeIndex,
    ) -> Option<SubGraphEdgeIndex> {
        if !self.edge_exists(from_index, &edge_weight, to_index) {
            self.touch_node(from_index);
            Some(self.graph.add_edge(from_index, to_index, edge_weight))
        } else {
            None
        }
    }

    pub(crate) fn remove_node(&mut self, node_index: SubGraphNodeIndex) {
        let Some((node_id, lineage_id)) = self
            .graph
            .node_weight(node_index)
            .map(|n| (n.id(), n.lineage_id()))
        else {
            return;
        };

        let parents: Vec<_> = self
            .graph
            .neighbors_directed(node_index, Incoming)
            .collect();

        self.graph.remove_node(node_index);
        self.remove_ids_from_indexes(node_id, lineage_id);

        parents
            .into_iter()
            .for_each(|parent_idx| self.touch_node(parent_idx));
    }

    pub(crate) fn add_or_get_ordering_node_for_node_index(
        &mut self,
        node_index: SubGraphNodeIndex,
    ) -> SubGraphNodeIndex {
        match self.ordering_node_for_node_index(node_index) {
            Some(existing_ordering_node_index) => existing_ordering_node_index,
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
                    node_index,
                    SplitGraphEdgeWeight::Ordering,
                    ordering_node_index,
                );

                self.touch_node(node_index);
                self.touch_node(ordering_node_index);

                ordering_node_index
            }
        }
    }

    pub(crate) fn add_ordered_edge(
        &mut self,
        from_index: SubGraphNodeIndex,
        edge_weight: SplitGraphEdgeWeight<E, K, N>,
        to_index: SubGraphNodeIndex,
    ) -> SplitGraphResult<(Option<SubGraphEdgeIndex>, Option<SubGraphEdgeIndex>)> {
        let target_id = self
            .graph
            .node_weight(to_index)
            .map(|n| n.id())
            .ok_or(SplitGraphError::NodeNotFoundAtIndex)?;

        let ordering_node_index = self.add_or_get_ordering_node_for_node_index(from_index);

        let mut ordinal_edge = None;
        if let Some(SplitGraphNodeWeight::Ordering { order, .. }) =
            self.graph.node_weight_mut(ordering_node_index)
        {
            if !order.contains(&target_id) {
                order.push(target_id);
            }

            ordinal_edge =
                self.add_edge_raw(ordering_node_index, SplitGraphEdgeWeight::Ordinal, to_index);
        }

        let edge = self.add_edge_raw(from_index, edge_weight, to_index);

        Ok((edge, ordinal_edge))
    }

    /// Add an edge between `from_index` and `to_index` if the edge does not exist.
    pub(crate) fn add_edge(
        &mut self,
        from_index: SubGraphNodeIndex,
        edge_weight: SplitGraphEdgeWeight<E, K, N>,
        to_index: SubGraphNodeIndex,
    ) -> Option<SubGraphEdgeIndex> {
        self.add_edge_raw(from_index, edge_weight, to_index)
    }

    pub(crate) fn touch_node(&mut self, node_index: SubGraphNodeIndex) {
        self.touched_nodes.insert(node_index);
    }

    pub(crate) fn update_external_target_ids(
        &mut self,
        from_index: SubGraphNodeIndex,
        old_target_id: SplitGraphNodeId,
        new_target_id: SplitGraphNodeId,
    ) {
        self.touch_node(from_index);

        let external_target_node_indexes: Vec<_> = self
            .graph
            .neighbors_directed(from_index, Outgoing)
            .filter(|neighbor_index| {
                self.graph
                    .node_weight(*neighbor_index)
                    .is_some_and(|weight| weight.external_target_id() == Some(old_target_id))
            })
            .collect();

        for neighbor_index in external_target_node_indexes {
            if let Some(SplitGraphNodeWeight::ExternalTarget { target, .. }) =
                self.graph.node_weight_mut(neighbor_index)
            {
                *target = new_target_id;
            }
            self.touch_node(neighbor_index);
        }
    }

    pub(crate) fn remove_external_source_edge(
        &mut self,
        from_index: SubGraphNodeIndex,
        to_index: SubGraphNodeIndex,
        external_source_data: ExternalSourceData<K, N>,
    ) {
        self.touch_node(from_index);
        let edge_indexes: Vec<_> = self
            .graph
            .edges_connecting(from_index, to_index)
            .filter(|edge_ref| {
                edge_ref.weight().external_source_data().as_ref() == Some(&external_source_data)
            })
            .map(|edge_ref| edge_ref.id())
            .collect();

        for edge_index in edge_indexes {
            self.graph.remove_edge(edge_index);
        }
    }

    /// Removes all edges between `from_index` and `to_index` that match the passed in kind.
    /// Also handles removing any correspond
    pub(crate) fn remove_edge_raw(
        &mut self,
        from_index: SubGraphNodeIndex,
        kind: SplitGraphEdgeWeightKind<K>,
        to_index: SubGraphNodeIndex,
    ) {
        self.touch_node(from_index);
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
    pub(crate) fn remove_edge_by_index(&mut self, edge_index: EdgeIndex<usize>) {
        if let Some((from_index, to_index)) = self.graph.edge_endpoints(edge_index) {
            self.touch_node(from_index);

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

            self.graph.remove_edge(edge_index);
        }
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = &SplitGraphNodeWeight<N>> {
        self.graph
            .node_indices()
            .filter_map(|node_index| self.graph.node_weight(node_index))
    }

    pub(crate) fn edges(
        &self,
    ) -> impl Iterator<
        Item = (
            &SplitGraphEdgeWeight<E, K, N>,
            SplitGraphNodeId,
            SplitGraphNodeId,
        ),
    > {
        self.graph.edge_indices().filter_map(|edge_index| {
            self.graph.edge_weight(edge_index).and_then(|edge_weight| {
                self.graph
                    .edge_endpoints(edge_index)
                    .and_then(|(source_idx, target_idx)| {
                        self.graph
                            .node_weight(source_idx)
                            .zip(self.graph.node_weight(target_idx))
                            .map(|(source, target)| (edge_weight, source.id(), target.id()))
                    })
            })
        })
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
                    SplitGraphEdgeWeight::ExternalSource { source_id, .. } => {
                        (format!("external source: {source_id}\n"), "red")
                    }
                    SplitGraphEdgeWeight::Ordering => ("ordering".into(), "green"),
                    SplitGraphEdgeWeight::Ordinal => ("ordinal".into(), "green"),
                };

                format!("label = \"{label}\"\ncolor = {color}")
            },
            &|_, (node_idx, node_weight)| {
                let (label, color) = match node_weight {
                    SplitGraphNodeWeight::Custom(n) => {
                        let node_dbg = n.dot_details();
                        (
                            format!("node: {} ({node_idx:?})\n{node_dbg}", n.id()),
                            "black",
                        )
                    }
                    SplitGraphNodeWeight::ExternalTarget { target, .. } => {
                        (format!("{node_idx:?}external target: {target}",), "red")
                    }
                    SplitGraphNodeWeight::GraphRoot { id, .. } => {
                        (format!("graph root: {id} ({node_idx:?})"), "blue")
                    }
                    SplitGraphNodeWeight::SubGraphRoot { id, .. } => {
                        (format!("subgraph root: {id} ({node_idx:?})"), "blue")
                    }
                    SplitGraphNodeWeight::Ordering { id, .. } => {
                        (format!("ordering: {id} ({node_idx:?})"), "green")
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
