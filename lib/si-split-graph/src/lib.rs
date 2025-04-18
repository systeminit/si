use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    marker::PhantomData,
    time::Instant,
};

use dashmap::DashMap;
use opt_zip::OptZip;
use petgraph::{prelude::*, stable_graph};
use serde::{Deserialize, Serialize};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::{Change, EntityKind},
    ContentHash,
};
use si_id::ulid::Ulid;
use telemetry::prelude::*;
use thiserror::Error;

pub mod opt_zip;
pub mod subgraph;
pub mod subgraph_address;
pub mod updates;

pub use subgraph::{SubGraph, SubGraphEdgeIndex, SubGraphNodeIndex};
pub use subgraph_address::SubGraphAddress;
pub use updates::Update;

#[derive(Error, Debug)]
pub enum SplitGraphError {
    #[error("Node id {0} not found")]
    NodeNotFound(SplitGraphNodeId),
    #[error("Node at index not found, this is a bug")]
    NodeNotFoundAtIndex,
    #[error("The splitgraph root is missing")]
    RootNodeNotFound,
    #[error("reorder must contain all the same ids as the original")]
    OrderContentMismatch,
    #[error("reorder must be of the same length as original")]
    OrderLengthMismatch,
    #[error("too many edges of kind {2} {1:?} to/from {0}")]
    TooManyEdgesOfKind(SplitGraphNodeId, Direction, String),
    #[error("No subgraph at index: {0}")]
    SubGraphMissing(usize),
    #[error("error reading subgraph with address {0:?}: {1}")]
    SubGraphRead(SubGraphAddress, String),
    #[error("error writing subgraph: {0}")]
    SubGraphWrite(String),
}

pub type SplitGraphResult<T> = Result<T, SplitGraphError>;

pub type SplitGraphNodeId = Ulid;
pub type SubGraphIndex = usize;

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum SplitGraphNodeWeight<N>
where
    N: CustomNodeWeight,
{
    /// The node weight kind provided by users of this crate
    Custom(N),
    /// A placeholder for an edge that points to a node in another subgraph
    ExternalTarget {
        id: SplitGraphNodeId,
        subgraph: SubGraphIndex,
        target: SplitGraphNodeId,
        merkle_tree_hash: MerkleTreeHash,
    },
    /// The ordering node for an ordered container
    Ordering {
        id: SplitGraphNodeId,
        order: Vec<SplitGraphNodeId>,
        merkle_tree_hash: MerkleTreeHash,
    },
    /// The root node for the entire graph
    GraphRoot {
        id: SplitGraphNodeId,
        merkle_tree_hash: MerkleTreeHash,
    },
    /// The root node for a subgraph (besides the first subgraph, which has the GraphRoot root)
    SubGraphRoot {
        id: SplitGraphNodeId,
        merkle_tree_hash: MerkleTreeHash,
    },
}

impl<N> SplitGraphNodeWeight<N>
where
    N: CustomNodeWeight,
{
    pub fn id(&self) -> SplitGraphNodeId {
        match self {
            SplitGraphNodeWeight::Custom(n) => n.id(),
            SplitGraphNodeWeight::ExternalTarget { id, .. }
            | SplitGraphNodeWeight::Ordering { id, .. }
            | SplitGraphNodeWeight::GraphRoot { id, .. }
            | SplitGraphNodeWeight::SubGraphRoot { id, .. } => *id,
        }
    }

    pub fn set_id(&mut self, new_id: SplitGraphNodeId) {
        match self {
            SplitGraphNodeWeight::Custom(n) => n.set_id(new_id),
            SplitGraphNodeWeight::ExternalTarget { id, .. }
            | SplitGraphNodeWeight::Ordering { id, .. }
            | SplitGraphNodeWeight::GraphRoot { id, .. }
            | SplitGraphNodeWeight::SubGraphRoot { id, .. } => *id = new_id,
        }
    }

    pub fn lineage_id(&self) -> SplitGraphNodeId {
        match self {
            SplitGraphNodeWeight::Custom(n) => n.lineage_id(),
            other => other.id(),
        }
    }

    pub fn set_lineage_id(&mut self, new_lineage_id: SplitGraphNodeId) {
        if let SplitGraphNodeWeight::Custom(n) = self {
            n.set_lineage_id(new_lineage_id);
        }
    }

    pub fn entity_kind(&self) -> EntityKind {
        match self {
            SplitGraphNodeWeight::Custom(c) => c.entity_kind(),
            SplitGraphNodeWeight::ExternalTarget { .. } => EntityKind::ExternalTarget,
            SplitGraphNodeWeight::Ordering { .. } => EntityKind::Ordering,
            SplitGraphNodeWeight::GraphRoot { .. } => EntityKind::Root,
            SplitGraphNodeWeight::SubGraphRoot { .. } => EntityKind::SubGraphRoot,
        }
    }

    pub fn set_merkle_tree_hash(&mut self, hash: MerkleTreeHash) {
        match self {
            SplitGraphNodeWeight::Custom(n) => n.set_merkle_tree_hash(hash),
            SplitGraphNodeWeight::ExternalTarget {
                merkle_tree_hash, ..
            }
            | SplitGraphNodeWeight::Ordering {
                merkle_tree_hash, ..
            }
            | SplitGraphNodeWeight::GraphRoot {
                merkle_tree_hash, ..
            }
            | SplitGraphNodeWeight::SubGraphRoot {
                merkle_tree_hash, ..
            } => *merkle_tree_hash = hash,
        }
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        match self {
            SplitGraphNodeWeight::Custom(n) => n.merkle_tree_hash(),
            SplitGraphNodeWeight::ExternalTarget {
                merkle_tree_hash, ..
            }
            | SplitGraphNodeWeight::Ordering {
                merkle_tree_hash, ..
            }
            | SplitGraphNodeWeight::GraphRoot {
                merkle_tree_hash, ..
            }
            | SplitGraphNodeWeight::SubGraphRoot {
                merkle_tree_hash, ..
            } => *merkle_tree_hash,
        }
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut hasher = ContentHash::hasher();

        match self {
            SplitGraphNodeWeight::Custom(c) => {
                return c.node_hash();
            }
            SplitGraphNodeWeight::ExternalTarget {
                id,
                subgraph,
                target,
                ..
            } => {
                hasher.update(&id.inner().to_bytes());
                hasher.update(&subgraph.to_le_bytes());
                hasher.update(&target.inner().to_bytes());
            }
            SplitGraphNodeWeight::Ordering { id, order, .. } => {
                hasher.update(&id.inner().to_bytes());
                for id in order {
                    hasher.update(&id.inner().to_bytes());
                }
            }
            SplitGraphNodeWeight::GraphRoot { id, .. } => {
                hasher.update(&id.inner().to_bytes());
            }
            SplitGraphNodeWeight::SubGraphRoot { id, .. } => {
                hasher.update(&id.inner().to_bytes());
            }
        };

        hasher.finalize()
    }

    pub fn custom_mut(&mut self) -> Option<&mut N> {
        match self {
            SplitGraphNodeWeight::Custom(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn custom(&self) -> Option<&N> {
        match self {
            SplitGraphNodeWeight::Custom(inner) => Some(inner),
            _ => None,
        }
    }

    pub fn external_target_id(&self) -> Option<SplitGraphNodeId> {
        match self {
            SplitGraphNodeWeight::ExternalTarget { target, .. } => Some(*target),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum SplitGraphEdgeWeight<E, K>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    Custom(E),
    ExternalSource {
        source_id: SplitGraphNodeId,
        subgraph: SubGraphIndex,
        is_default: bool,
        edge_kind: K,
    },
    Ordering,
    Ordinal,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, Serialize, Deserialize, Hash)]
pub enum SplitGraphEdgeWeightKind<K>
where
    K: EdgeKind,
{
    Custom(K),
    ExternalSource,
    Ordering,
    Ordinal,
}

impl<E, K> From<SplitGraphEdgeWeight<E, K>> for SplitGraphEdgeWeightKind<K>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    fn from(value: SplitGraphEdgeWeight<E, K>) -> Self {
        match value {
            SplitGraphEdgeWeight::Custom(c) => SplitGraphEdgeWeightKind::Custom(c.kind()),
            SplitGraphEdgeWeight::ExternalSource { .. } => SplitGraphEdgeWeightKind::ExternalSource,
            SplitGraphEdgeWeight::Ordering => SplitGraphEdgeWeightKind::Ordering,
            SplitGraphEdgeWeight::Ordinal => SplitGraphEdgeWeightKind::Ordinal,
        }
    }
}

impl<E, K> From<&SplitGraphEdgeWeight<E, K>> for SplitGraphEdgeWeightKind<K>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    fn from(value: &SplitGraphEdgeWeight<E, K>) -> Self {
        match value {
            SplitGraphEdgeWeight::Custom(c) => SplitGraphEdgeWeightKind::Custom(c.kind()),
            SplitGraphEdgeWeight::ExternalSource { .. } => SplitGraphEdgeWeightKind::ExternalSource,
            SplitGraphEdgeWeight::Ordering => SplitGraphEdgeWeightKind::Ordering,
            SplitGraphEdgeWeight::Ordinal => SplitGraphEdgeWeightKind::Ordinal,
        }
    }
}

impl<E, K> SplitGraphEdgeWeight<E, K>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub fn custom(&self) -> Option<&E> {
        match self {
            SplitGraphEdgeWeight::Custom(weight) => Some(weight),
            _ => None,
        }
    }

    pub fn is_default(&self) -> bool {
        match self {
            SplitGraphEdgeWeight::Custom(c) => c.is_default(),
            SplitGraphEdgeWeight::ExternalSource { is_default, .. } => *is_default,
            SplitGraphEdgeWeight::Ordering => false,
            SplitGraphEdgeWeight::Ordinal => false,
        }
    }

    pub fn clone_as_non_default(&self) -> Self {
        match self {
            SplitGraphEdgeWeight::Custom(c) => {
                SplitGraphEdgeWeight::Custom(c.clone_as_non_default())
            }
            SplitGraphEdgeWeight::ExternalSource {
                source_id,
                subgraph,
                edge_kind,
                ..
            } => SplitGraphEdgeWeight::ExternalSource {
                source_id: *source_id,
                subgraph: *subgraph,
                is_default: false,
                edge_kind: *edge_kind,
            },
            SplitGraphEdgeWeight::Ordering => SplitGraphEdgeWeight::Ordering,
            SplitGraphEdgeWeight::Ordinal => SplitGraphEdgeWeight::Ordinal,
        }
    }

    pub fn edge_entropy(&self) -> Option<Vec<u8>> {
        match self {
            SplitGraphEdgeWeight::Custom(c) => c.edge_entropy(),
            SplitGraphEdgeWeight::ExternalSource {
                source_id,
                subgraph,
                ..
            } => {
                let mut entropy = vec![];
                entropy.extend_from_slice(&source_id.inner().to_bytes());
                entropy.extend_from_slice(&subgraph.to_le_bytes());
                Some(entropy)
            }
            SplitGraphEdgeWeight::Ordering | SplitGraphEdgeWeight::Ordinal => None,
        }
    }
}

pub trait EdgeKind: std::hash::Hash + PartialEq + Eq + Copy + Clone + std::fmt::Debug {}

pub trait CustomNodeWeight: PartialEq + Eq + Clone + std::fmt::Debug {
    fn id(&self) -> SplitGraphNodeId;
    fn set_id(&mut self, id: SplitGraphNodeId);

    fn lineage_id(&self) -> SplitGraphNodeId;
    fn set_lineage_id(&mut self, id: SplitGraphNodeId);

    fn entity_kind(&self) -> EntityKind;

    fn set_merkle_tree_hash(&mut self, hash: MerkleTreeHash);
    fn merkle_tree_hash(&self) -> MerkleTreeHash;
    fn node_hash(&self) -> ContentHash;

    fn dot_details(&self) -> String;
}

pub trait CustomEdgeWeight<K>: std::hash::Hash + PartialEq + Eq + Clone + std::fmt::Debug
where
    K: EdgeKind,
{
    fn kind(&self) -> K;
    fn edge_entropy(&self) -> Option<Vec<u8>>;
    // Default edges have a rule that there can be only *one* default edge of a certain kind
    // outgoing from a node. This rule will be enforced when updates are performed.
    fn is_default(&self) -> bool;
    fn clone_as_non_default(&self) -> Self;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SplitGraphNodeIndex {
    subgraph: SubGraphIndex,
    index: SubGraphNodeIndex,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SplitGraphEdgeIndex {
    subgraph: SubGraphIndex,
    index: SubGraphEdgeIndex,
}

impl SplitGraphNodeIndex {
    pub fn new(subgraph: SubGraphIndex, index: SubGraphNodeIndex) -> Self {
        Self { subgraph, index }
    }
}

pub type ExternalSourceMap = BTreeMap<SplitGraphNodeId, Vec<SplitGraphEdgeIndex>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuperGraph {
    addresses: Vec<SubGraphAddress>,
    root_index: SplitGraphNodeIndex,
    external_source_map: ExternalSourceMap,
    split_max: usize,
}

impl SuperGraph {
    pub fn new(
        split_max: usize,
        root_index: SplitGraphNodeIndex,
        external_source_map: ExternalSourceMap,
    ) -> Self {
        Self {
            addresses: vec![],
            root_index,
            split_max,
            external_source_map,
        }
    }

    pub fn split_max(&self) -> usize {
        self.split_max
    }

    pub fn root_index(&self) -> SplitGraphNodeIndex {
        self.root_index
    }

    pub fn add_subgraph_address(&mut self, subgraph_address: SubGraphAddress) {
        self.addresses.push(subgraph_address);
    }

    pub fn addresses(&self) -> &[SubGraphAddress] {
        self.addresses.as_slice()
    }

    pub fn address_for_subgraph(&self, index: usize) -> Option<SubGraphAddress> {
        self.addresses.get(index).copied()
    }

    pub fn external_source_map(&self) -> &ExternalSourceMap {
        &self.external_source_map
    }
}

#[derive(Clone, Debug)]
pub struct SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    supergraph: SuperGraph,
    subgraphs: Vec<SubGraph<N, E, K>>,
    id_to_split_graph_index: DashMap<SplitGraphNodeId, SplitGraphNodeIndex>,
}

impl<N, E, K> SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    K: EdgeKind,
    E: CustomEdgeWeight<K>,
{
    pub fn new(split_max: usize) -> Self {
        let mut first_subgraph = SubGraph::new();
        let root_id = Ulid::new();
        let root_index = first_subgraph
            .graph
            .add_node(SplitGraphNodeWeight::GraphRoot {
                id: root_id,
                merkle_tree_hash: MerkleTreeHash::nil(),
            });
        first_subgraph.node_index_by_id.insert(root_id, root_index);
        first_subgraph.root_index = root_index;

        Self {
            supergraph: SuperGraph {
                addresses: vec![],
                root_index: SplitGraphNodeIndex {
                    index: root_index,
                    subgraph: 0,
                },
                split_max,
                external_source_map: ExternalSourceMap::new(),
            },
            subgraphs: vec![first_subgraph],
            id_to_split_graph_index: DashMap::new(),
        }
    }

    pub fn from_parts(supergraph: SuperGraph, subgraphs: Vec<SubGraph<N, E, K>>) -> Self {
        Self {
            supergraph,
            subgraphs,
            id_to_split_graph_index: DashMap::new(),
        }
    }

    pub fn node_count(&self) -> usize {
        self.subgraphs.iter().fold(0, |count, subgraph| {
            count.saturating_add(subgraph.node_index_by_id.len())
        })
    }

    pub fn root_id(&self) -> SplitGraphResult<SplitGraphNodeId> {
        self.node_weight_by_index(self.supergraph.root_index)
            .map(|node| node.id())
            .ok_or(SplitGraphError::RootNodeNotFound)
    }

    pub fn subgraph_count(&self) -> usize {
        self.subgraphs.len()
    }

    pub fn recalculate_merkle_tree_hashes_based_on_touched_nodes(&mut self) {
        self.subgraphs
            .iter_mut()
            .enumerate()
            .for_each(|(idx, subgraph)| {
                let (nodes, edges) = subgraph.recalculate_merkle_tree_hash_based_on_touched_nodes();
                if idx == 0 {
                    warn!("nodes: {}, edges: {}", nodes, edges);
                }
            });
    }

    pub fn recalculate_entire_merkle_tree_hashes(&mut self) {
        self.subgraphs
            .iter_mut()
            .for_each(|subgraph| subgraph.recalculate_entire_merkle_tree_hash());
    }

    pub fn make_node_id(&mut self) -> SplitGraphNodeId {
        Ulid::new()
    }

    fn new_subgraph(&mut self) -> usize {
        self.supergraph.addresses.push(SubGraphAddress::nil());

        let subgraph = SubGraph::new_with_root();
        let subgraph_index = self.subgraphs.len();
        self.subgraphs.push(subgraph);

        subgraph_index
    }

    fn new_empty_subgraph(&mut self) -> usize {
        self.supergraph.addresses.push(SubGraphAddress::nil());

        let subgraph = SubGraph::new();
        let subgraph_index = self.subgraphs.len();
        self.subgraphs.push(subgraph);

        subgraph_index
    }

    pub fn supergraph(&self) -> &SuperGraph {
        &self.supergraph
    }

    pub fn subgraphs(&self) -> &[SubGraph<N, E, K>] {
        self.subgraphs.as_slice()
    }

    fn get_subgraph(&self, subgraph_index: usize) -> SplitGraphResult<&SubGraph<N, E, K>> {
        self.subgraphs
            .get(subgraph_index)
            .ok_or(SplitGraphError::SubGraphMissing(subgraph_index))
    }

    fn get_subgraph_mut(
        &mut self,
        subgraph_index: usize,
    ) -> SplitGraphResult<&mut SubGraph<N, E, K>> {
        self.subgraphs
            .get_mut(subgraph_index)
            .ok_or(SplitGraphError::SubGraphMissing(subgraph_index))
    }

    fn add_node_to_subgraph(
        &mut self,
        subgraph_index: SubGraphIndex,
        node: SplitGraphNodeWeight<N>,
    ) -> SplitGraphResult<SplitGraphNodeIndex> {
        let subgraph = self.get_subgraph_mut(subgraph_index as usize)?;
        let node_id = node.id();
        let node_index = subgraph.add_node(node);
        if node_index == NodeIndex::new(2085) {
            warn!(
                "added node {} to {} at index {:?}",
                node_id, subgraph_index, node_index
            );
        }

        Ok(SplitGraphNodeIndex::new(subgraph_index, node_index))
    }

    pub fn add_or_replace_node(&mut self, node: N) -> SplitGraphResult<SplitGraphNodeIndex> {
        let node_id = node.id();
        if let Some(split_graph_index) = self.node_id_to_index(node_id) {
            let subgraph = self.get_subgraph_mut(split_graph_index.subgraph as usize)?;
            if split_graph_index.index == NodeIndex::new(2085) {
                warn!("replace node {} at index {:?}", node_id, split_graph_index);
            }
            subgraph.replace_node(split_graph_index.index, SplitGraphNodeWeight::Custom(node));

            return Ok(split_graph_index);
        }

        let subgraph_index = if let Some((index, _)) =
            self.subgraphs.iter().enumerate().find(|(_, sub)| {
                // We add one to the max so that the root node is not part of the count
                sub.node_index_by_id.len() < (self.supergraph.split_max + 1)
            }) {
            index
        } else {
            self.new_subgraph()
        };

        let index =
            self.add_node_to_subgraph(subgraph_index, SplitGraphNodeWeight::Custom(node))?;

        self.id_to_split_graph_index.insert(node_id, index);

        Ok(index)
    }

    pub fn add_ordered_node(&mut self, node: N) -> SplitGraphResult<SplitGraphNodeIndex> {
        let split_graph_index = self.add_or_replace_node(node)?;
        let subgraph = self.get_subgraph_mut(split_graph_index.subgraph as usize)?;
        subgraph.add_or_get_ordering_node_for_node_index(split_graph_index.index);

        Ok(split_graph_index)
    }

    fn node_weight_by_index(&self, index: SplitGraphNodeIndex) -> Option<&SplitGraphNodeWeight<N>> {
        self.subgraphs
            .get(index.subgraph as usize)
            .and_then(|sub| sub.graph.node_weight(index.index))
    }

    pub fn subgraph_index_for_node(&self, node_id: SplitGraphNodeId) -> Option<usize> {
        let index = self.node_id_to_index(node_id);
        index.map(|index| index.subgraph)
    }

    pub fn subgraph_root_id(&self, subgraph_index: usize) -> Option<SplitGraphNodeId> {
        self.subgraphs
            .get(subgraph_index)
            .and_then(|sub| sub.graph.node_weight(sub.root_index))
            .map(|n| n.id())
    }

    pub fn raw_node_weight(&self, node_id: SplitGraphNodeId) -> Option<&SplitGraphNodeWeight<N>> {
        self.node_id_to_index(node_id).and_then(|index| {
            self.subgraphs
                .get(index.subgraph)
                .and_then(|subgraph| subgraph.graph.node_weight(index.index))
        })
    }

    pub fn node_weight(&self, node_id: SplitGraphNodeId) -> Option<&N> {
        self.raw_node_weight(node_id).and_then(|node| node.custom())
    }

    pub fn raw_node_weight_mut(
        &mut self,
        node_id: SplitGraphNodeId,
    ) -> Option<&mut SplitGraphNodeWeight<N>> {
        self.node_id_to_index(node_id).and_then(|index| {
            self.subgraphs
                .get_mut(index.subgraph)
                .and_then(|subgraph| subgraph.graph.node_weight_mut(index.index))
        })
    }

    pub fn node_weight_mut(&mut self, node_id: SplitGraphNodeId) -> Option<&mut N> {
        self.raw_node_weight_mut(node_id)
            .and_then(|weight| weight.custom_mut())
    }

    pub fn update_node_id(
        &mut self,
        current_node_id: SplitGraphNodeId,
        new_id: SplitGraphNodeId,
        new_lineage_id: SplitGraphNodeId,
    ) -> SplitGraphResult<()> {
        let index = self
            .node_id_to_index(current_node_id)
            .ok_or(SplitGraphError::NodeNotFound(current_node_id))?;
        let node_weight_mut = self
            .raw_node_weight_mut(current_node_id)
            .ok_or(SplitGraphError::NodeNotFound(current_node_id))?;

        let current_lineage_id = node_weight_mut.lineage_id();
        node_weight_mut.set_id(new_id);
        node_weight_mut.set_lineage_id(new_lineage_id);

        if let Some(subgraph) = self.subgraphs.get_mut(index.subgraph) {
            subgraph.remove_ids_from_indexes(current_node_id, current_lineage_id);
            subgraph.add_ids_to_indexes(new_id, new_lineage_id, index.index);
        }

        self.id_to_split_graph_index.insert(new_id, index);
        self.id_to_split_graph_index.remove(&current_node_id);
        self.touch_node(new_id);

        Ok(())
    }

    pub fn touch_node(&mut self, node_id: SplitGraphNodeId) {
        let Some(index) = self.node_id_to_index(node_id) else {
            return;
        };
        let Some(subgraph) = self.subgraphs.get_mut(index.subgraph) else {
            return;
        };
        subgraph.touch_node(index.index);
    }

    #[inline]
    pub fn node_id_to_index(&self, id: SplitGraphNodeId) -> Option<SplitGraphNodeIndex> {
        match self
            .id_to_split_graph_index
            .get(&id)
            .map(|entry_ref| *entry_ref.value())
        {
            Some(index) => Some(index),
            None => {
                let index = self
                    .subgraphs
                    .iter()
                    .enumerate()
                    .find(|(_, sub)| sub.node_index_by_id.contains_key(&id))
                    .and_then(|(idx, sub)| {
                        sub.node_index_by_id
                            .get(&id)
                            .map(|subgraph_index| SplitGraphNodeIndex::new(idx, *subgraph_index))
                    });

                if let Some(index) = index {
                    self.id_to_split_graph_index.insert(id, index);
                }

                index
            }
        }
    }

    pub fn raw_outgoing_edges_from_subgraph_root(
        &self,
        subgraph: usize,
    ) -> Option<Vec<(SplitGraphEdgeWeight<E, K>, SplitGraphNodeWeight<N>)>> {
        let subgraph = self.subgraphs().get(subgraph)?;
        let root = subgraph.root_index;

        Some(
            subgraph
                .graph
                .edges_directed(root, Outgoing)
                .filter_map(|edge_ref| {
                    subgraph
                        .graph
                        .node_weight(edge_ref.target())
                        .map(|node| (edge_ref.weight().clone(), node.clone()))
                })
                .collect(),
        )
    }

    pub fn raw_outgoing_edges(
        &self,
        node_id: SplitGraphNodeId,
    ) -> Option<Vec<(SplitGraphEdgeWeight<E, K>, SplitGraphNodeWeight<N>)>> {
        let index = self.node_id_to_index(node_id)?;
        let subgraph = self.subgraphs().get(index.subgraph)?;

        Some(
            subgraph
                .graph
                .edges_directed(index.index, Outgoing)
                .filter_map(|edge_ref| {
                    subgraph
                        .graph
                        .node_weight(edge_ref.target())
                        .map(|node| (edge_ref.weight().clone(), node.clone()))
                })
                .collect(),
        )
    }

    pub fn raw_incoming_edges(
        &self,
        node_id: SplitGraphNodeId,
    ) -> Option<Vec<SplitGraphEdgeWeight<E, K>>> {
        let index = self.node_id_to_index(node_id)?;
        let subgraph = self.subgraphs().get(index.subgraph)?;

        Some(
            subgraph
                .graph
                .edges_directed(index.index, Incoming)
                .map(|edge_ref| edge_ref.weight().clone())
                .collect(),
        )
    }

    pub fn remove_node(&mut self, node_id: SplitGraphNodeId) -> SplitGraphResult<()> {
        if self.node_id_to_index(node_id).is_none() {
            return Ok(());
        }

        // Although removing a node is enough to remove its edges from a single subgraph,
        // we have to call remove_edge here for all incoming and outgoing edges in order
        // to ensure we remove cross graph edges (from external sources and to external targets)
        let incoming_sources: Vec<_> = self
            .edges_directed(node_id, Incoming)?
            .map(|edge_ref| (edge_ref.source(), edge_ref.weight().kind()))
            .collect();

        let outgoing_targets: Vec<_> = self
            .edges_directed(node_id, Outgoing)?
            .map(|edge_ref| (edge_ref.target(), edge_ref.weight().kind()))
            .collect();

        for (incoming_source, kind) in incoming_sources {
            self.remove_edge(incoming_source, kind, node_id)?;
        }
        for (outgoing_target, kind) in outgoing_targets {
            self.remove_edge(node_id, kind, outgoing_target)?;
        }

        let node_index = self
            .node_id_to_index(node_id)
            .ok_or(SplitGraphError::NodeNotFound(node_id))?;

        let subgraph_idx = node_index.subgraph;
        let subgraph = self.get_subgraph_mut(subgraph_idx as usize)?;

        subgraph.remove_node(node_index.index);
        self.id_to_split_graph_index.remove(&node_id);

        Ok(())
    }

    pub fn find_edge(
        &self,
        from_id: SplitGraphNodeId,
        to_id: SplitGraphNodeId,
        edge_weight_kind: K,
    ) -> Option<&E> {
        let from_index = self.node_id_to_index(from_id.into())?;

        let from_subgraph_idx = from_index.subgraph;

        let subgraph = self.subgraphs.get(from_subgraph_idx as usize)?;

        subgraph
            .graph
            .edges_directed(from_index.index, Outgoing)
            .find(|edge_ref| {
                if Some(edge_weight_kind) == edge_ref.weight().custom().map(|edge| edge.kind()) {
                    match subgraph.graph.node_weight(edge_ref.target()) {
                        Some(node) => match node {
                            SplitGraphNodeWeight::Custom(c) => c.id() == to_id,
                            SplitGraphNodeWeight::ExternalTarget { target, .. } => *target == to_id,
                            _ => false,
                        },
                        None => false,
                    }
                } else {
                    false
                }
            })
            .and_then(|edge_ref| edge_ref.weight().custom())
    }

    pub fn remove_edge(
        &mut self,
        from_id: SplitGraphNodeId,
        edge_kind: K,
        to_id: SplitGraphNodeId,
    ) -> SplitGraphResult<()> {
        let from_index = self
            .node_id_to_index(from_id)
            .ok_or(SplitGraphError::NodeNotFound(from_id))?;
        let to_index = self
            .node_id_to_index(to_id)
            .ok_or(SplitGraphError::NodeNotFound(to_id))?;

        self.touch_node(from_id);

        let from_subgraph_idx = from_index.subgraph;
        let to_subgraph_idx = to_index.subgraph;

        if from_subgraph_idx == to_subgraph_idx {
            let from_subgraph = self.get_subgraph_mut(from_subgraph_idx as usize)?;
            if let Some(edge_idx) = from_subgraph
                .graph
                .edges_directed(from_index.index, Outgoing)
                .find(|edge_ref| {
                    edge_ref
                        .weight()
                        .custom()
                        .map(|edge| edge.kind() == edge_kind)
                        .unwrap_or(false)
                        && from_subgraph
                            .graph
                            .node_weight(edge_ref.target())
                            .map(|node| node.id() == to_id)
                            .unwrap_or(false)
                })
                .map(|edge_ref| edge_ref.id())
            {
                from_subgraph.remove_edge_by_index(edge_idx);
            }
        } else {
            let from_subgraph = self.get_subgraph_mut(from_subgraph_idx as usize)?;
            if let Some(edge_idx) = from_subgraph
                .graph
                .edges_directed(from_index.index, Outgoing)
                .find(|edge_ref| {
                    edge_ref
                        .weight()
                        .custom()
                        .map(|edge| edge.kind() == edge_kind)
                        .unwrap_or(false)
                        && from_subgraph
                            .graph
                            .node_weight(edge_ref.target())
                            .map(|node| match node {
                                SplitGraphNodeWeight::ExternalTarget { target, .. } => {
                                    *target == to_id
                                }
                                _ => false,
                            })
                            .unwrap_or(false)
                })
                .map(|edge_ref| edge_ref.id())
            {
                // println!(
                //     "attempting to find and remove external source to {:?}",
                //     from_id
                // );
                from_subgraph.remove_edge_by_index(edge_idx);
                let to_subgraph = self.get_subgraph_mut(to_subgraph_idx as usize)?;
                if let Some(edge_idx) = to_subgraph
                    .graph
                    .edges_directed(to_index.index, Incoming)
                    .find(|edge_ref| match edge_ref.weight() {
                        SplitGraphEdgeWeight::ExternalSource {
                            source_id,
                            edge_kind: ek,
                            ..
                        } => *source_id == from_id && *ek == edge_kind,
                        _ => false,
                    })
                    .map(|edge_ref| edge_ref.id())
                {
                    to_subgraph.remove_edge_by_index(edge_idx);
                }
            }
        }

        Ok(())
    }

    pub fn add_ordered_edge(
        &mut self,
        from_id: SplitGraphNodeId,
        edge: E,
        to_id: SplitGraphNodeId,
    ) -> SplitGraphResult<()> {
        self.add_edge_inner(from_id, edge, to_id, true)
    }

    pub fn add_edge(
        &mut self,
        from_id: SplitGraphNodeId,
        edge: E,
        to_id: SplitGraphNodeId,
    ) -> SplitGraphResult<()> {
        self.add_edge_inner(from_id, edge, to_id, false)
    }

    fn add_edge_inner(
        &mut self,
        from_id: SplitGraphNodeId,
        edge: E,
        to_id: SplitGraphNodeId,
        ordered: bool,
    ) -> SplitGraphResult<()> {
        let from_index = self
            .node_id_to_index(from_id)
            .ok_or(SplitGraphError::NodeNotFound(from_id))?;
        let to_index = self
            .node_id_to_index(to_id)
            .ok_or(SplitGraphError::NodeNotFound(to_id))?;

        let custom_edge_weight = SplitGraphEdgeWeight::Custom(edge.clone());

        let from_subgraph_idx = from_index.subgraph;
        let to_subgraph_idx = to_index.subgraph;
        if from_subgraph_idx == to_subgraph_idx {
            let from_subgraph = self.get_subgraph_mut(from_subgraph_idx as usize)?;
            if ordered {
                from_subgraph.add_ordered_edge(
                    from_index.index,
                    custom_edge_weight,
                    to_index.index,
                )?;
            } else {
                from_subgraph.add_edge(from_index.index, custom_edge_weight, to_index.index);
            }
        } else {
            let ext_target_id = SplitGraphNodeId::new();
            let ext_target_idx = self.add_node_to_subgraph(
                from_subgraph_idx,
                SplitGraphNodeWeight::ExternalTarget {
                    id: ext_target_id,
                    subgraph: to_subgraph_idx,
                    target: to_id,
                    merkle_tree_hash: MerkleTreeHash::nil(),
                },
            )?;

            let from_subgraph = self.get_subgraph_mut(from_subgraph_idx as usize)?;
            if ordered {
                from_subgraph.add_ordered_edge(
                    from_index.index,
                    custom_edge_weight,
                    ext_target_idx.index,
                )?;
            } else {
                from_subgraph.add_edge(from_index.index, custom_edge_weight, ext_target_idx.index);
            }

            let to_subgraph = self.get_subgraph_mut(to_subgraph_idx as usize)?;
            if let Some(edge_index) = to_subgraph.add_edge(
                to_subgraph.root_index,
                SplitGraphEdgeWeight::ExternalSource {
                    source_id: from_id,
                    subgraph: from_subgraph_idx,
                    is_default: edge.is_default(),
                    edge_kind: edge.kind(),
                },
                to_index.index,
            ) {
                let source_edge_index = SplitGraphEdgeIndex {
                    subgraph: to_subgraph_idx,
                    index: edge_index,
                };
                self.supergraph
                    .external_source_map
                    .entry(from_id)
                    .and_modify(|external_source_edges| {
                        external_source_edges.push(source_edge_index)
                    })
                    .or_insert(vec![source_edge_index]);
            }
        }

        self.ordered_children(from_id);

        Ok(())
    }

    pub fn reorder_node<L>(&mut self, node_id: SplitGraphNodeId, lambda: L) -> SplitGraphResult<()>
    where
        L: FnOnce(&[SplitGraphNodeId]) -> Vec<SplitGraphNodeId>,
    {
        let split_graph_index = self
            .node_id_to_index(node_id)
            .ok_or(SplitGraphError::NodeNotFound(node_id))?;
        let subgraph = self.get_subgraph_mut(split_graph_index.subgraph as usize)?;

        subgraph.reorder_node(split_graph_index.index, lambda)
    }

    pub fn ordered_children(&self, node_id: SplitGraphNodeId) -> Option<Vec<SplitGraphNodeId>> {
        let split_graph_index = self.node_id_to_index(node_id)?;
        let subgraph = self.subgraphs.get(split_graph_index.subgraph as usize)?;

        subgraph
            .ordered_children(split_graph_index.index)
            .map(|node_indexes| {
                node_indexes
                    .into_iter()
                    .filter_map(|idx| {
                        subgraph.graph.node_weight(idx).map(|n| match n {
                            SplitGraphNodeWeight::ExternalTarget { target, .. } => *target,
                            other => other.id(),
                        })
                    })
                    .collect()
            })
    }

    /// Find all ancestors of `node_id` across all subgraphs (includes non-custom nodes).
    /// Cycle-safe, and non-recursive.
    pub fn all_parents_of(
        &self,
        node_id: SplitGraphNodeId,
    ) -> SplitGraphResult<Vec<SplitGraphNodeId>> {
        let mut result = vec![];

        let mut seen_list = HashSet::new();
        let mut work_queue = VecDeque::from([node_id]);

        while let Some(node_id) = work_queue.pop_front() {
            if seen_list.contains(&node_id) {
                continue;
            }
            seen_list.insert(node_id);
            result.push(node_id);

            work_queue.extend(
                self.edges_directed(node_id, Incoming)?
                    .map(|edge_ref| edge_ref.source()),
            );
        }

        Ok(result)
    }

    /// Find the outgoing or incoming neighbor of `from_id` which is connected to
    /// `from_id` via an edge with kind `kind`. If there is no such neighbor,
    /// returns `None`. But if there is more than one such neighbor, an error
    /// is returned.
    pub fn directed_unique_neighbor_of_edge_weight_kind(
        &self,
        from_id: SplitGraphNodeId,
        direction: Direction,
        kind: K,
    ) -> SplitGraphResult<Option<Ulid>> {
        let mut edges_of_kind =
            self.edges_directed_for_edge_weight_kind(from_id, direction, kind)?;

        let Some(edge_ref) = edges_of_kind.next() else {
            return Ok(None);
        };

        if edges_of_kind.next().is_some() {
            return Err(SplitGraphError::TooManyEdgesOfKind(
                from_id,
                direction,
                format!("{:?}", kind),
            ));
        }

        Ok(Some(match direction {
            Outgoing => edge_ref.target(),
            Incoming => edge_ref.source(),
        }))
    }

    pub fn edges_directed_for_edge_weight_kind<'a>(
        &'a self,
        from_id: SplitGraphNodeId,
        direction: Direction,
        kind: K,
    ) -> SplitGraphResult<impl Iterator<Item = SplitGraphEdgeReference<'a, E, K>> + 'a> {
        let iter = self
            .edges_directed(from_id, direction)?
            .filter(move |edge_ref| edge_ref.weight().kind() == kind);

        Ok(iter)
    }

    /// Produces an iterator akin to the petgraph edges_directed iterator, but supports cross-subgraph edges.
    /// Note that this iterator does not expose "internal" split graph edges, such as Ordering, Ordinal,
    /// or ExternalSource edges. Only the "custom" edges produced.
    pub fn edges_directed(
        &self,
        from_id: SplitGraphNodeId,
        direction: Direction,
    ) -> SplitGraphResult<SplitGraphEdges<N, E, K>> {
        let split_graph_index = self
            .node_id_to_index(from_id)
            .ok_or(SplitGraphError::NodeNotFound(from_id))?;

        let subgraph = self.get_subgraph(split_graph_index.subgraph as usize)?;

        let edges = subgraph
            .graph
            .edges_directed(split_graph_index.index, direction);

        Ok(SplitGraphEdges {
            this_subgraph: subgraph,
            subgraphs: self.subgraphs(),
            edges,
            from_id,
            direction,
            debug: false,
        })
    }

    pub fn edges_directed_debug(
        &self,
        from_id: SplitGraphNodeId,
        direction: Direction,
    ) -> SplitGraphResult<SplitGraphEdges<N, E, K>> {
        let split_graph_index = self
            .node_id_to_index(from_id)
            .ok_or(SplitGraphError::NodeNotFound(from_id))?;

        let subgraph = self.get_subgraph(split_graph_index.subgraph as usize)?;

        let edges = subgraph
            .graph
            .edges_directed(split_graph_index.index, direction);

        Ok(SplitGraphEdges {
            this_subgraph: subgraph,
            subgraphs: self.subgraphs(),
            edges,
            from_id,
            direction,
            debug: true,
        })
    }

    pub fn cleanup(&mut self) {
        let mut external_source_edges_to_remove = vec![];
        for subgraph in self.subgraphs.iter_mut() {
            let removed_ids = subgraph.cleanup();
            for id in removed_ids {
                let external_source_edges = self.supergraph.external_source_map.remove(&id);
                let Some(external_source_edges) = external_source_edges else {
                    continue;
                };
                external_source_edges_to_remove.extend(external_source_edges);
                // dbg!(subgraph.node_id_to_index(id));
            }
        }

        for external_source_edge in external_source_edges_to_remove {
            if let Some(subgraph) = self.subgraphs.get_mut(external_source_edge.subgraph) {
                // dbg!(subgraph.graph.edge_weight(external_source_edge.index));
                // let endpoints = subgraph.graph.edge_endpoints(external_source_edge.index);
                // if let Some((_, target)) = endpoints {
                //     dbg!(subgraph.graph.node_weight(target));
                // }
                subgraph.graph.remove_edge(external_source_edge.index);
            }
        }

        self.id_to_split_graph_index.clear();
    }

    pub fn cleanup_and_merkle_tree_hash(&mut self) {
        let start = Instant::now();
        self.cleanup();
        warn!("cleanup took {:?}", start.elapsed());
        let start = Instant::now();
        self.recalculate_merkle_tree_hashes_based_on_touched_nodes();
        warn!(
            "recalculate_merkle_tree_hashes_based_on_touched_nodes took {:?}",
            start.elapsed()
        );
    }

    /// Calculate the updates that this graph has relative to `base_graph`
    pub fn detect_updates(&self, updated_graph: &SplitGraph<N, E, K>) -> Vec<Update<N, E, K>> {
        let mut updates = vec![];

        let mut subgraph_iter = OptZip::new(
            updated_graph.subgraphs.iter().enumerate(),
            self.subgraphs.iter(),
        );

        while let Some((Some((updated_subgraph_index, updated_subgraph)), maybe_base_subgraph)) =
            subgraph_iter.next()
        {
            match maybe_base_subgraph {
                Some(base_subgraph) => updates.extend(
                    updates::Detector::new(base_subgraph, updated_subgraph, updated_subgraph_index)
                        .detect_updates()
                        .into_iter(),
                ),
                None => {
                    updates.push(Update::NewSubGraph);
                    updates.extend(
                        updates::subgraph_as_updates(updated_subgraph, updated_subgraph_index)
                            .into_iter(),
                    )
                }
            }
        }

        updates
    }

    pub fn detect_changes(
        &self,
        updated_graph: &SplitGraph<N, E, K>,
    ) -> SplitGraphResult<Vec<Change>> {
        let mut changes = vec![];

        let mut subgraph_iter = OptZip::new(
            updated_graph.subgraphs.iter().enumerate(),
            self.subgraphs.iter(),
        );

        let mut detected_ids = HashSet::new();

        while let Some((Some((updated_subgraph_index, updated_subgraph)), maybe_base_subgraph)) =
            subgraph_iter.next()
        {
            match maybe_base_subgraph {
                Some(base_subgraph) => {
                    let mut subgraph_changes = updates::Detector::new(
                        base_subgraph,
                        updated_subgraph,
                        updated_subgraph_index,
                    )
                    .detect_changes();

                    for subgraph_change in &subgraph_changes {
                        detected_ids.insert(subgraph_change.entity_id);
                    }

                    subgraph_changes.extend(
                        base_subgraph
                            .node_index_by_id
                            .keys()
                            .filter(|node_id| {
                                !updated_subgraph.node_index_by_id.contains_key(*node_id)
                            })
                            .filter_map(|node_id| {
                                match base_subgraph
                                    .node_id_to_index(*node_id)
                                    .and_then(|index| base_subgraph.graph.node_weight(index))
                                {
                                    Some(SplitGraphNodeWeight::Custom(c)) => Some(Change {
                                        entity_id: (*node_id).into(),
                                        entity_kind: c.entity_kind(),
                                        merkle_tree_hash: c.merkle_tree_hash(),
                                    }),
                                    _ => None,
                                }
                            }),
                    );

                    changes.extend(subgraph_changes);
                }
                None => {
                    // This entire subgraph is a new set of changes
                    changes.extend(updated_subgraph.graph.node_weights().filter_map(
                        |node_weight| match node_weight {
                            SplitGraphNodeWeight::Custom(_)
                            | SplitGraphNodeWeight::GraphRoot { .. } => {
                                detected_ids.insert(node_weight.id().into());

                                Some(Change {
                                    entity_id: node_weight.id().into(),
                                    entity_kind: node_weight.entity_kind(),
                                    merkle_tree_hash: node_weight.merkle_tree_hash(),
                                })
                            }
                            SplitGraphNodeWeight::ExternalTarget { .. }
                            | SplitGraphNodeWeight::Ordering { .. }
                            | SplitGraphNodeWeight::SubGraphRoot { .. } => None,
                        },
                    ));
                }
            }
        }

        let mut final_changes = vec![];

        // Now that we've detected all the changed nodes in every subgraph, we need to detect all the
        // parents of these changed nodes, *across* subgraphs, since these will have also changed.
        // reversed so that parents come before children in the finalized list
        for change in &changes {
            for parent_id in self
                .all_parents_of(change.entity_id.into())?
                .into_iter()
                .rev()
            {
                if detected_ids.contains(&parent_id.into()) {
                    continue;
                }
                detected_ids.insert(parent_id.into());

                if let Some(
                    weight @ SplitGraphNodeWeight::GraphRoot { .. }
                    | weight @ SplitGraphNodeWeight::Custom(_),
                ) = self.raw_node_weight(parent_id)
                {
                    // If we find this node now, that means its merkle tree hash
                    // hasn't changed since it was in different subgraph than the
                    // child node which *did* change. This just adds a bit of entropy
                    // to the changes so that the checksum generated is different.
                    // May not be necessary since there *will* be at least one
                    // other changed node?
                    let mut hasher = MerkleTreeHash::hasher();
                    hasher.update(change.merkle_tree_hash.as_bytes());
                    hasher.update(weight.merkle_tree_hash().as_bytes());
                    final_changes.push(Change {
                        entity_id: parent_id.into(),
                        entity_kind: weight.entity_kind(),
                        merkle_tree_hash: hasher.finalize(),
                    });
                }
            }
        }

        final_changes.extend(changes);

        Ok(final_changes)
    }

    pub fn perform_updates(&mut self, updates: &[Update<N, E, K>]) {
        let mut removed_node_ids = vec![];
        for update in updates {
            match update {
                Update::NewEdge {
                    subgraph_index,
                    source,
                    destination,
                    edge_weight,
                } => {
                    let Some(subgraph) = self.subgraphs.get_mut(*subgraph_index) else {
                        continue;
                    };
                    let Some((from_index, to_index)) = subgraph
                        .node_id_to_index(*source)
                        .zip(subgraph.node_id_to_index(*destination))
                    else {
                        continue;
                    };

                    if edge_weight.is_default() {
                        ensure_only_one_default_edge(
                            subgraph,
                            from_index,
                            to_index,
                            edge_weight.clone(),
                        );
                    }

                    subgraph.add_edge_raw(from_index, edge_weight.clone(), to_index);
                }
                Update::RemoveEdge {
                    subgraph_index,
                    source,
                    destination,
                    edge_kind,
                } => {
                    let Some(subgraph) = self.subgraphs.get_mut(*subgraph_index as usize) else {
                        continue;
                    };
                    let Some((from_index, to_index)) = subgraph
                        .node_id_to_index(*source)
                        .zip(subgraph.node_id_to_index(*destination))
                    else {
                        continue;
                    };

                    subgraph.remove_edge_raw(from_index, *edge_kind, to_index);
                }
                Update::RemoveNode { subgraph_index, id } => {
                    let Some(subgraph) = self.subgraphs.get_mut(*subgraph_index as usize) else {
                        continue;
                    };
                    let Some(node_index) = subgraph.node_id_to_index(*id) else {
                        continue;
                    };

                    removed_node_ids.push((*subgraph_index, node_index));

                    subgraph.remove_node(node_index);
                }
                Update::ReplaceNode {
                    subgraph_index,
                    node_weight,
                } => {
                    let Some(subgraph) = self.subgraphs.get_mut(*subgraph_index as usize) else {
                        continue;
                    };
                    let Some(node_index) = subgraph.node_id_to_index(node_weight.id()) else {
                        continue;
                    };
                    let previous_id = subgraph.replace_node(node_index, node_weight.clone());
                    if previous_id.is_some_and(|previous_id| previous_id != node_weight.id()) {
                        self.id_to_split_graph_index.insert(
                            node_weight.id(),
                            SplitGraphNodeIndex::new(*subgraph_index, node_index),
                        );
                        self.id_to_split_graph_index.remove(&(previous_id.unwrap()));
                    }
                }
                Update::NewNode {
                    subgraph_index,
                    node_weight,
                } => {
                    if self.node_id_to_index(node_weight.id()).is_some() {
                        continue;
                    }
                    let Some(subgraph) = self.subgraphs.get_mut(*subgraph_index as usize) else {
                        continue;
                    };
                    let index = subgraph.add_node(node_weight.clone());
                    self.id_to_split_graph_index.insert(
                        node_weight.id(),
                        SplitGraphNodeIndex::new(*subgraph_index, index),
                    );
                }
                Update::NewSubGraph => {
                    self.new_empty_subgraph();
                }
            }
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &N> {
        self.subgraphs
            .iter()
            .flat_map(|subgraph| subgraph.nodes())
            .filter_map(|n| n.custom())
    }

    pub fn edges(&self) -> impl Iterator<Item = (&E, SplitGraphNodeId, SplitGraphNodeId)> {
        self.subgraphs
            .iter()
            .flat_map(|subgraph| subgraph.edges())
            .filter_map(|(e, source, target)| e.custom().map(|custom| (custom, source, target)))
    }

    pub fn tiny_dot_to_file(&self, prefix: &str) {
        for (idx, subgraph) in self.subgraphs.iter().enumerate() {
            subgraph.tiny_dot_to_file(&format!("{prefix}-subgraph-{}", idx + 1));
        }
    }
}

fn ensure_only_one_default_edge<N, E, K>(
    graph: &mut SubGraph<N, E, K>,
    source_idx: SubGraphNodeIndex,
    destination_idx: SubGraphNodeIndex,
    edge_weight: SplitGraphEdgeWeight<E, K>,
) where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    let edge_weight_kind: SplitGraphEdgeWeightKind<K> = edge_weight.into();
    let existing_default_targets: Vec<(_, _)> = graph
        .graph
        .edges_directed(source_idx, Outgoing)
        .filter(|edge_ref| {
            edge_weight_kind == edge_ref.weight().into()
                && edge_ref.weight().is_default()
                && edge_ref.target() != destination_idx
        })
        .map(|edge_ref| (edge_ref.weight().clone(), edge_ref.target()))
        .collect();

    for (edge_weight, target_idx) in existing_default_targets {
        graph.remove_edge_raw(source_idx, edge_weight_kind, target_idx);
        graph.add_edge_raw(source_idx, edge_weight.clone_as_non_default(), target_idx);
    }
}

#[derive(Debug)]
pub struct SplitGraphEdgeReference<'a, E, K>
where
    E: 'a + CustomEdgeWeight<K>,
    K: EdgeKind,
{
    source_id: SplitGraphNodeId,
    target_id: SplitGraphNodeId,
    weight: &'a E,
    phantom_k: PhantomData<K>,
}

impl<'a, E, K> SplitGraphEdgeReference<'a, E, K>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub fn new(source_id: SplitGraphNodeId, target_id: SplitGraphNodeId, weight: &'a E) -> Self {
        Self {
            source_id,
            target_id,
            weight,
            phantom_k: PhantomData,
        }
    }

    pub fn source(&self) -> SplitGraphNodeId {
        self.source_id
    }

    pub fn target(&self) -> SplitGraphNodeId {
        self.target_id
    }

    pub fn weight(&self) -> &'a E {
        self.weight
    }
}

// pub struct SplitGraphNeighbors<'a, N, E, K>
// where
//     N: CustomNodeWeight,
//     E: CustomEdgeWeight<K>,
//     K: EdgeKind,
// {
//     subgraphs: &'a [SubGraph<N, E, K>],
//     edges: stable_graph::Edges<'a, SplitGraphEdgeWeight<E, K>, Directed, SubGraphIndex>,
//     start:
// }

pub struct SplitGraphEdges<'a, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    this_subgraph: &'a SubGraph<N, E, K>,
    subgraphs: &'a [SubGraph<N, E, K>],
    edges: stable_graph::Edges<'a, SplitGraphEdgeWeight<E, K>, Directed, SubGraphIndex>,
    from_id: SplitGraphNodeId,
    direction: Direction,
    debug: bool,
}

impl<'a, N, E, K> Iterator for SplitGraphEdges<'a, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Item = SplitGraphEdgeReference<'a, E, K>;

    fn next(&mut self) -> Option<Self::Item> {
        self.edges.next().and_then(|edge_ref| match self.direction {
            Outgoing => match edge_ref.weight() {
                SplitGraphEdgeWeight::Custom(weight) => self
                    .this_subgraph
                    .graph
                    .node_weight(edge_ref.target())
                    .map(|n| match n {
                        SplitGraphNodeWeight::ExternalTarget { target, .. } => *target,
                        internal_target => internal_target.id(),
                    })
                    .map(|target_id| SplitGraphEdgeReference::new(self.from_id, target_id, weight)),
                SplitGraphEdgeWeight::ExternalSource { .. }
                | SplitGraphEdgeWeight::Ordering
                | SplitGraphEdgeWeight::Ordinal => self.next(),
            },
            Incoming => match edge_ref.weight() {
                SplitGraphEdgeWeight::Custom(weight) => self
                    .this_subgraph
                    .graph
                    .node_weight(edge_ref.source())
                    .map(|source_node| {
                        SplitGraphEdgeReference::new(source_node.id(), self.from_id, weight)
                    }),
                SplitGraphEdgeWeight::ExternalSource {
                    source_id,
                    edge_kind,
                    subgraph,
                    ..
                } => self.subgraphs.get(*subgraph).and_then(|ext_subgraph| {
                    if self.debug {
                        dbg!(source_id);
                        dbg!(edge_kind);
                        dbg!(ext_subgraph.node_id_to_index(*source_id));
                    }

                    match ext_subgraph.node_id_to_index(*source_id) {
                        Some(source_index) => ext_subgraph
                            .graph
                            .edges_directed(source_index, Outgoing)
                            .filter(|subgraph_edge_ref| {
                                if self.debug {
                                    dbg!(subgraph_edge_ref.weight().custom());
                                }
                                subgraph_edge_ref
                                    .weight()
                                    .custom()
                                    .is_some_and(|e| e.kind() == *edge_kind)
                            })
                            .find(|subgraph_edge_ref| {
                                if self.debug {
                                    dbg!(ext_subgraph
                                        .graph
                                        .node_weight(subgraph_edge_ref.target()));
                                }
                                ext_subgraph
                                    .graph
                                    .node_weight(subgraph_edge_ref.target())
                                    .is_some_and(|weight| {
                                        weight.external_target_id() == Some(self.from_id)
                                    })
                            })
                            .and_then(|subgraph_edge_ref| {
                                subgraph_edge_ref.weight().custom().map(|weight| {
                                    SplitGraphEdgeReference::new(*source_id, self.from_id, weight)
                                })
                            }),
                        None => {
                            println!("missing node for external source: {source_id:?}");
                            for (index, subgraph) in self.subgraphs.iter().enumerate() {
                                if let Some(bob) =
                                    subgraph.graph().node_indices().find(|node_index| {
                                        subgraph.graph().node_weight(*node_index).unwrap().id()
                                            == *source_id
                                    })
                                {
                                    println!("found {source_id:?} at {bob:?} in subgraph {index}");
                                }
                            }

                            self.next()
                        }
                    }
                }),
                SplitGraphEdgeWeight::Ordinal | SplitGraphEdgeWeight::Ordering => self.next(),
            },
        })
    }
}

impl<N, E, K> petgraph::visit::GraphBase for SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type EdgeId = SplitGraphEdgeIndex;
    type NodeId = Ulid;
}

impl<N, E, K> petgraph::visit::Visitable for SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Map = HashSet<Ulid>;

    fn visit_map(&self) -> Self::Map {
        HashSet::with_capacity(self.node_count())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

// impl<'a, N, E, K, R, W> petgraph::visit::IntoNeighbors for &'a SplitGraph<'_, '_, N, E, K, R, W>
// where
//     N: CustomNodeWeight,
//     E: CustomEdgeWeight<K>,
//     K: EdgeKind,
//     R: SubGraphReader<N, E, K>,
//     W: SubGraphWriter<N, E, K>,
// {
//     type Neighbors = ;

//     fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
//         todo!()
//     }
// }

#[cfg(test)]
mod tests;
