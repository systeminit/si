use std::collections::{HashSet, VecDeque};

use async_trait::async_trait;
use opt_zip::OptZip;
use petgraph::{prelude::*, stable_graph};
use serde::{Deserialize, Serialize};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    workspace_snapshot::{Change, EntityKind},
    ContentHash,
};
use si_id::ulid::Ulid;
use thiserror::Error;

mod opt_zip;
pub mod subgraph;
pub mod subgraph_address;
pub mod updates;

use subgraph::{SubGraph, SubGraphEdgeIndex, SubGraphNodeIndex};
pub use subgraph_address::SubGraphAddress;
use updates::Update;

pub const MAX_NODES: usize = ((u16::MAX / 2) - 1) as usize;

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
    #[error("No subgraph at index: {0}")]
    SubGraphMissing(usize),
    #[error("error reading subgraph with address {0:?}: {1}")]
    SubGraphRead(SubGraphAddress, String),
    #[error("error writing subgraph: {0}")]
    SubGraphWrite(String),
}

pub type SplitGraphResult<T> = Result<T, SplitGraphError>;

pub type SplitGraphNodeId = Ulid;
pub type SubGraphIndex = u16;

#[async_trait]
pub trait SubGraphReader<N, E, K>: Clone + std::fmt::Debug
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Error: std::error::Error;

    async fn read_subgraph(
        &self,
        address: SubGraphAddress,
    ) -> Result<SubGraph<N, E, K>, Self::Error>;
}

#[async_trait]
pub trait SubGraphWriter<Node, Edge, K>: Clone + std::fmt::Debug
where
    Node: CustomNodeWeight,
    Edge: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Error: std::error::Error;

    async fn write_subgraph(
        &mut self,
        graph: &SubGraph<Node, Edge, K>,
    ) -> Result<SubGraphAddress, Self::Error>;
}

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

    pub fn lineage_id(&self) -> SplitGraphNodeId {
        match self {
            SplitGraphNodeWeight::Custom(n) => n.lineage_id(),
            other => other.id(),
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

    pub fn edge_hash(&self) -> Option<ContentHash> {
        match self {
            SplitGraphEdgeWeight::Custom(c) => c.edge_hash(),
            SplitGraphEdgeWeight::ExternalSource {
                source_id,
                subgraph,
                ..
            } => {
                let mut hasher = ContentHash::hasher();
                hasher.update(&source_id.inner().to_bytes());
                hasher.update(&subgraph.to_le_bytes());
                Some(hasher.finalize())
            }
            SplitGraphEdgeWeight::Ordering | SplitGraphEdgeWeight::Ordinal => None,
        }
    }
}

pub trait EdgeKind: std::hash::Hash + PartialEq + Eq + Copy + Clone + std::fmt::Debug {}

pub trait CustomNodeWeight: PartialEq + Eq + Clone + std::fmt::Debug {
    fn id(&self) -> SplitGraphNodeId;
    fn lineage_id(&self) -> SplitGraphNodeId;
    fn entity_kind(&self) -> EntityKind;

    fn set_merkle_tree_hash(&mut self, hash: MerkleTreeHash);
    fn merkle_tree_hash(&self) -> MerkleTreeHash;
    fn node_hash(&self) -> ContentHash;
    fn ordered(&self) -> bool;
}

pub trait CustomEdgeWeight<K>: std::hash::Hash + PartialEq + Eq + Clone + std::fmt::Debug
where
    K: EdgeKind,
{
    fn kind(&self) -> K;
    fn edge_hash(&self) -> Option<ContentHash>;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuperGraph {
    addresses: Vec<SubGraphAddress>,
    root_index: SplitGraphNodeIndex,
    split_max: u16,
}

#[derive(Clone, Debug)]
pub struct SplitGraph<'a, 'b, N, E, K, R, W>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
    R: SubGraphReader<N, E, K>,
    W: SubGraphWriter<N, E, K>,
{
    supergraph: SuperGraph,
    subgraphs: Vec<SubGraph<N, E, K>>,
    reader: &'a R,
    #[allow(unused)]
    writer: &'b W,
}

impl<'a, 'b, N, E, K, R, W> SplitGraph<'a, 'b, N, E, K, R, W>
where
    N: CustomNodeWeight,
    K: EdgeKind,
    E: CustomEdgeWeight<K>,
    R: SubGraphReader<N, E, K>,
    W: SubGraphWriter<N, E, K>,
{
    pub fn new(reader: &'a R, writer: &'b W, split_max: u16) -> Self {
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
            },
            subgraphs: vec![first_subgraph],
            reader,
            writer,
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

    pub async fn new_with_addresses(
        reader: &'a R,
        writer: &'b W,
        addresses: &[SubGraphAddress],
        split_max: u16,
    ) -> SplitGraphResult<Self> {
        // We need to do this without creating a root node
        let mut split_graph = Self::new(reader, writer, split_max);
        split_graph.add_subgraphs(addresses).await?;

        Ok(split_graph)
    }

    pub async fn add_subgraphs(
        &mut self,
        subgraph_addresses: &[SubGraphAddress],
    ) -> SplitGraphResult<()> {
        self.supergraph.addresses.extend(subgraph_addresses.iter());

        for (idx, address) in self.supergraph.addresses.iter().enumerate() {
            if self.subgraphs.get(idx).is_none() {
                let subgraph = self
                    .reader
                    .read_subgraph(*address)
                    .await
                    .map_err(|err| SplitGraphError::SubGraphRead(*address, err.to_string()))?;

                self.subgraphs.push(subgraph);
            }
        }

        Ok(())
    }

    pub fn recalculate_merkle_tree_hashes_based_on_touched_nodes(&mut self) {
        self.subgraphs
            .iter_mut()
            .for_each(|subgraph| subgraph.recalculate_merkle_tree_hash_based_on_touched_nodes());
    }

    pub fn recalculate_entire_merkle_tree_hashes(&mut self) {
        self.subgraphs
            .iter_mut()
            .for_each(|subgraph| subgraph.recalculate_entire_merkle_tree_hash());
    }

    pub fn make_node_id(&mut self) -> SplitGraphNodeId {
        Ulid::new()
    }

    fn new_subgraph(&mut self) -> u16 {
        self.supergraph.addresses.push(SubGraphAddress::nil());

        let subgraph = SubGraph::new_with_root();
        let subgraph_index = self.subgraphs.len() as u16;
        self.subgraphs.push(subgraph);

        subgraph_index
    }

    fn new_empty_subgraph(&mut self) -> u16 {
        self.supergraph.addresses.push(SubGraphAddress::nil());

        let subgraph = SubGraph::new();
        let subgraph_index = self.subgraphs.len() as u16;
        self.subgraphs.push(subgraph);

        subgraph_index
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
        let node_index = subgraph.add_node(node);

        Ok(SplitGraphNodeIndex::new(subgraph_index, node_index))
    }

    pub fn add_or_replace_node(&mut self, node: N) -> SplitGraphResult<SplitGraphNodeIndex> {
        let node_id = node.id();
        if let Some(split_graph_index) = self.node_id_to_index(node_id) {
            let subgraph = self.get_subgraph_mut(split_graph_index.subgraph as usize)?;
            subgraph.replace_node(split_graph_index.index, SplitGraphNodeWeight::Custom(node));

            return Ok(split_graph_index);
        }

        let subgraph_index = if let Some((index, _)) =
            self.subgraphs.iter().enumerate().find(|(_, sub)| {
                // We add one to the max so that the root node is not part of the count
                sub.node_index_by_id.len() < ((self.supergraph.split_max + 1) as usize)
            }) {
            index as u16
        } else {
            self.new_subgraph()
        };

        self.add_node_to_subgraph(subgraph_index, SplitGraphNodeWeight::Custom(node))
    }

    fn node_weight_by_index(&self, index: SplitGraphNodeIndex) -> Option<&SplitGraphNodeWeight<N>> {
        self.subgraphs
            .get(index.subgraph as usize)
            .and_then(|sub| sub.graph.node_weight(index.index))
    }

    pub fn subgraph_for_node(&self, node_id: SplitGraphNodeId) -> Option<usize> {
        for (index, sub) in self.subgraphs.iter().enumerate() {
            if sub.node_index_by_id.contains_key(&node_id) {
                return Some(index);
            }
        }

        None
    }

    pub fn subgraph_root_id(&self, subgraph_index: usize) -> Option<SplitGraphNodeId> {
        self.subgraphs
            .get(subgraph_index)
            .and_then(|sub| sub.graph.node_weight(sub.root_index))
            .map(|n| n.id())
    }

    pub fn raw_node_weight(&self, node_id: SplitGraphNodeId) -> Option<&SplitGraphNodeWeight<N>> {
        for sub in &self.subgraphs {
            if let Some(index) = sub.node_index_by_id.get(&node_id) {
                return sub.graph.node_weight(*index);
            }
        }

        None
    }

    pub fn node_weight(&self, node_id: SplitGraphNodeId) -> Option<&N> {
        self.raw_node_weight(node_id)
            .and_then(|weight| weight.custom())
    }

    pub fn raw_node_weight_mut(
        &mut self,
        node_id: SplitGraphNodeId,
    ) -> Option<&mut SplitGraphNodeWeight<N>> {
        for sub in self.subgraphs.iter_mut() {
            if let Some(index) = sub.node_index_by_id.get(&node_id) {
                return sub.graph.node_weight_mut(*index);
            }
        }

        None
    }

    pub fn node_weight_mut(&mut self, node_id: SplitGraphNodeId) -> Option<&mut N> {
        self.raw_node_weight_mut(node_id)
            .and_then(|weight| weight.custom_mut())
    }

    pub fn touch_node(&mut self, node_id: SplitGraphNodeId) {
        for subgraph in self.subgraphs.iter_mut() {
            if let Some(node_index) = subgraph.node_id_to_index(node_id) {
                subgraph.touch_node(node_index);
                break;
            }
        }
    }

    pub fn node_id_to_index(&self, id: SplitGraphNodeId) -> Option<SplitGraphNodeIndex> {
        self.subgraphs
            .iter()
            .enumerate()
            .find(|(_, sub)| sub.node_index_by_id.contains_key(&id))
            .and_then(|(idx, sub)| {
                sub.node_index_by_id
                    .get(&id)
                    .map(|subgraph_index| SplitGraphNodeIndex::new(idx as u16, *subgraph_index))
            })
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
                from_subgraph.remove_edge_by_index(edge_idx);
                let to_subgraph = self.get_subgraph_mut(to_subgraph_idx as usize)?;
                let root_index = to_subgraph.root_index;
                if let Some(edge_idx) = to_subgraph
                    .graph
                    .edges_directed(root_index, Outgoing)
                    .find(|edge_ref| match edge_ref.weight() {
                        SplitGraphEdgeWeight::ExternalSource {
                            source_id,
                            edge_kind: ek,
                            ..
                        } => {
                            *source_id == from_id
                                && *ek == edge_kind
                                && to_subgraph
                                    .graph
                                    .node_weight(edge_ref.target())
                                    .map(|node| node.id() == to_id)
                                    .unwrap_or(false)
                        }
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

    pub fn add_edge(
        &mut self,
        from_id: SplitGraphNodeId,
        edge: E,
        to_id: SplitGraphNodeId,
    ) -> SplitGraphResult<()> {
        let from_index = self
            .node_id_to_index(from_id)
            .ok_or(SplitGraphError::NodeNotFound(from_id))?;
        let to_index = self
            .node_id_to_index(to_id)
            .ok_or(SplitGraphError::NodeNotFound(to_id))?;

        let from_subgraph_idx = from_index.subgraph;
        let to_subgraph_idx = to_index.subgraph;
        if from_subgraph_idx == to_subgraph_idx {
            let from_subgraph = self.get_subgraph_mut(from_subgraph_idx as usize)?;
            from_subgraph.add_edge(
                from_index.index,
                SplitGraphEdgeWeight::Custom(edge),
                to_index.index,
            )?;
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
            from_subgraph.add_edge(
                from_index.index,
                SplitGraphEdgeWeight::Custom(edge.clone()),
                ext_target_idx.index,
            )?;
            let to_subgraph = self.get_subgraph_mut(to_subgraph_idx as usize)?;
            to_subgraph.add_edge(
                to_subgraph.root_index,
                SplitGraphEdgeWeight::ExternalSource {
                    source_id: from_id,
                    subgraph: from_subgraph_idx,
                    is_default: edge.is_default(),
                    edge_kind: edge.kind(),
                },
                to_index.index,
            )?;
        }

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
            subgraph,
            edges,
            from_id,
            direction,
        })
    }

    pub fn cleanup(&mut self) {
        for subgraph in self.subgraphs.iter_mut() {
            subgraph.cleanup();
        }
    }

    pub fn cleanup_and_merkle_tree_hash(&mut self) {
        self.cleanup();
        self.recalculate_merkle_tree_hashes_based_on_touched_nodes();
    }

    /// Calculate the updates that this graph has relative to `base_graph`
    pub fn detect_updates(
        &self,
        updated_graph: &SplitGraph<N, E, K, R, W>,
    ) -> Vec<Update<N, E, K>> {
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
                    updates::Detector::new(
                        base_subgraph,
                        updated_subgraph,
                        updated_subgraph_index as u16,
                    )
                    .detect_updates()
                    .into_iter(),
                ),
                None => {
                    updates.push(Update::NewSubGraph);
                    updates.extend(
                        updates::subgraph_as_updates(
                            updated_subgraph,
                            updated_subgraph_index as u16,
                        )
                        .into_iter(),
                    )
                }
            }
        }

        updates
    }

    pub fn detect_changes(
        &self,
        updated_graph: &SplitGraph<N, E, K, R, W>,
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
                        updated_subgraph_index as u16,
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
        for update in updates {
            match update {
                Update::NewEdge {
                    subgraph_index,
                    source,
                    destination,
                    edge_weight,
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

                    // if matches!(edge_kind, SplitGraphEdgeWeightKind::Ordinal) {
                    //     subgraph.remove_from_order(from_index, *destination);
                    // }

                    subgraph.remove_edge_raw(from_index, *edge_kind, to_index);
                }
                Update::RemoveNode { subgraph_index, id } => {
                    let Some(subgraph) = self.subgraphs.get_mut(*subgraph_index as usize) else {
                        continue;
                    };
                    let Some(node_index) = subgraph.node_id_to_index(*id) else {
                        continue;
                    };

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
                    subgraph.replace_node(node_index, node_weight.clone());
                }
                Update::NewNode {
                    subgraph_index,
                    node_weight,
                } => {
                    let Some(subgraph) = self.subgraphs.get_mut(*subgraph_index as usize) else {
                        continue;
                    };
                    match subgraph.node_id_to_index(node_weight.id()) {
                        Some(existing_index) => {
                            subgraph.replace_node(existing_index, node_weight.clone())
                        }
                        None => {
                            subgraph.add_node(node_weight.clone());
                        }
                    }
                }
                Update::NewSubGraph => {
                    self.new_empty_subgraph();
                }
            }
        }
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

pub struct SplitGraphEdgeReference<'a, E, K>
where
    E: 'a + CustomEdgeWeight<K>,
    K: EdgeKind,
{
    source_id: SplitGraphNodeId,
    target_id: SplitGraphNodeId,
    weight: &'a SplitGraphEdgeWeight<E, K>,
}

impl<E, K> SplitGraphEdgeReference<'_, E, K>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub fn source(&self) -> SplitGraphNodeId {
        self.source_id
    }

    pub fn target(&self) -> SplitGraphNodeId {
        self.target_id
    }

    pub fn weight(&self) -> &SplitGraphEdgeWeight<E, K> {
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
    subgraph: &'a SubGraph<N, E, K>,
    edges: stable_graph::Edges<'a, SplitGraphEdgeWeight<E, K>, Directed, SubGraphIndex>,
    from_id: SplitGraphNodeId,
    direction: Direction,
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
            Outgoing => {
                if matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordering) {
                    // Ordering nodes are hidden
                    self.next()
                } else {
                    self.subgraph
                        .graph
                        .node_weight(edge_ref.target())
                        .map(|n| match n {
                            SplitGraphNodeWeight::ExternalTarget { target, .. } => *target,
                            internal_target => internal_target.id(),
                        })
                        .map(|target_id| SplitGraphEdgeReference {
                            source_id: self.from_id,
                            target_id,
                            weight: edge_ref.weight(),
                        })
                }
            }
            Incoming => {
                let weight = edge_ref.weight();
                match weight {
                    SplitGraphEdgeWeight::Ordinal => {
                        return self.next();
                    }
                    SplitGraphEdgeWeight::Custom(_) | SplitGraphEdgeWeight::Ordering => self
                        .subgraph
                        .graph
                        .node_weight(edge_ref.source())
                        .map(|source_node| source_node.id()),
                    SplitGraphEdgeWeight::ExternalSource { source_id, .. } => Some(*source_id),
                }
                .map(|source_id| SplitGraphEdgeReference {
                    source_id,
                    target_id: self.from_id,
                    weight,
                })
            }
        })
    }
}

impl<N, E, K, R, W> petgraph::visit::GraphBase for SplitGraph<'_, '_, N, E, K, R, W>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
    R: SubGraphReader<N, E, K>,
    W: SubGraphWriter<N, E, K>,
{
    type EdgeId = SplitGraphEdgeIndex;
    type NodeId = Ulid;
}

impl<N, E, K, R, W> petgraph::visit::Visitable for SplitGraph<'_, '_, N, E, K, R, W>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
    R: SubGraphReader<N, E, K>,
    W: SubGraphWriter<N, E, K>,
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
