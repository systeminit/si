use std::collections::HashSet;

use async_trait::async_trait;
use fixedbitset::FixedBitSet;
use petgraph::{prelude::*, stable_graph};
use serde::{Deserialize, Serialize};
use si_events::{
    merkle_tree_hash::{self, MerkleTreeHash},
    ContentHash,
};
use si_id::ulid::Ulid;
use thiserror::Error;

pub mod subgraph;
pub mod subgraph_address;

use subgraph::{SubGraph, SubGraphEdgeIndex, SubGraphNodeIndex};
pub use subgraph_address::SubGraphAddress;

pub const MAX_NODES: usize = ((u16::MAX / 2) - 1) as usize;

#[derive(Error, Debug)]
pub enum SplitGraphError {
    #[error("error reading subgraph with address {0:?}: {1}")]
    SubGraphRead(SubGraphAddress, String),
    #[error("error writing subgraph: {0}")]
    SubGraphWrite(String),
}

pub type SplitGraphResult<T> = Result<T, SplitGraphError>;

pub type SplitGraphNodeId = Ulid;
pub type SubGraphIndex = u16;

#[async_trait]
pub trait SubGraphReader<N, E, K>
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
pub trait SubGraphWriter<Node, Edge, K>
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// A ordering node, which cont
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

    pub fn custom(&self) -> Option<&N> {
        match self {
            SplitGraphNodeWeight::Custom(inner) => Some(inner),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SplitGraphEdgeWeight<E, K>
where
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    Custom(E),
    ExternalSource {
        source_id: SplitGraphNodeId,
        subgraph: SubGraphIndex,
        edge_kind: K,
    },
    Ordering,
    Ordinal,
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
}

pub trait EdgeKind: PartialEq + Copy + Clone + std::fmt::Debug {}

pub trait CustomNodeWeight: PartialEq + Clone + std::fmt::Debug {
    fn id(&self) -> SplitGraphNodeId;

    fn set_merkle_tree_hash(&mut self, hash: MerkleTreeHash);
    fn merkle_tree_hash(&self) -> MerkleTreeHash;
    fn node_hash(&self) -> ContentHash;
    fn ordered(&self) -> bool;
}

pub trait CustomEdgeWeight<K>: PartialEq + Clone + std::fmt::Debug
where
    K: EdgeKind,
{
    fn kind(&self) -> K;
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

#[derive(Serialize, Deserialize)]
pub struct SuperGraph {
    addresses: Vec<SubGraphAddress>,
    root_index: SplitGraphNodeIndex,
    split_max: u16,
}

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

    pub fn root_id(&self) -> SplitGraphNodeId {
        self.node_weight_by_index(self.supergraph.root_index)
            .map(|node| node.id())
            .unwrap()
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

    pub fn make_node_id(&mut self) -> SplitGraphNodeId {
        Ulid::new()
    }

    pub fn new_subgraph(&mut self) -> u16 {
        self.supergraph.addresses.push(SubGraphAddress::nil());

        let mut subgraph = SubGraph::new();
        let new_root_id = Ulid::new();

        let root_node = SplitGraphNodeWeight::SubGraphRoot {
            id: new_root_id,
            merkle_tree_hash: MerkleTreeHash::nil(),
        };
        let subgraph_index = self.subgraphs.len() as u16;
        let index = subgraph.graph.add_node(root_node);
        subgraph.root_index = index;
        subgraph.node_index_by_id.insert(new_root_id, index);

        self.subgraphs.push(subgraph);

        subgraph_index
    }

    fn add_node_to_subgraph(
        &mut self,
        subgraph_index: SubGraphIndex,
        node: SplitGraphNodeWeight<N>,
    ) -> SplitGraphNodeIndex {
        let subgraph = self.subgraphs.get_mut(subgraph_index as usize).unwrap();
        let node_index = subgraph.add_node(node);

        SplitGraphNodeIndex::new(subgraph_index, node_index)
    }

    pub fn add_or_replace_node(&mut self, node: N) -> SplitGraphNodeIndex {
        let node_id = node.id();
        if let Some(split_graph_index) = self.node_id_to_index(node_id) {
            if let Some(subgraph) = self.subgraphs.get_mut(split_graph_index.subgraph as usize) {
                subgraph.replace_node(split_graph_index.index, SplitGraphNodeWeight::Custom(node));
            }

            return split_graph_index;
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

    pub fn node_weight(&self, node_id: SplitGraphNodeId) -> Option<&SplitGraphNodeWeight<N>> {
        for sub in &self.subgraphs {
            if let Some(index) = sub.node_index_by_id.get(&node_id) {
                return sub.graph.node_weight(*index);
            }
        }

        None
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
    ) {
        let from_index = self.node_id_to_index(from_id).unwrap();
        let to_index = self.node_id_to_index(to_id).unwrap();

        let from_subgraph_idx = from_index.subgraph;
        let to_subgraph_idx = to_index.subgraph;

        if from_subgraph_idx == to_subgraph_idx {
            let from_subgraph = self.subgraphs.get_mut(from_subgraph_idx as usize).unwrap();
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
                from_subgraph.remove_edge(edge_idx);
            }
        } else {
            let from_subgraph = self.subgraphs.get_mut(from_subgraph_idx as usize).unwrap();
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
                from_subgraph.remove_edge(edge_idx);
                let to_subgraph = self.subgraphs.get_mut(to_subgraph_idx as usize).unwrap();
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
                    to_subgraph.remove_edge(edge_idx);
                }
            }
        }
    }

    pub fn add_edge(&mut self, from_id: SplitGraphNodeId, edge: E, to_id: SplitGraphNodeId) {
        let from_index = self.node_id_to_index(from_id).unwrap();
        let to_index = self.node_id_to_index(to_id).unwrap();

        let from_subgraph_idx = from_index.subgraph;
        let to_subgraph_idx = to_index.subgraph;
        if from_subgraph_idx == to_subgraph_idx {
            let from_subgraph = self.subgraphs.get_mut(from_subgraph_idx as usize).unwrap();
            from_subgraph.add_edge(
                from_index.index,
                SplitGraphEdgeWeight::Custom(edge),
                to_index.index,
            );
        } else {
            let ext_target_id = Ulid::new();
            let ext_target_idx = self.add_node_to_subgraph(
                from_subgraph_idx,
                SplitGraphNodeWeight::ExternalTarget {
                    id: ext_target_id,
                    subgraph: to_subgraph_idx,
                    target: to_id,
                    merkle_tree_hash: MerkleTreeHash::nil(),
                },
            );
            let from_subgraph = self.subgraphs.get_mut(from_subgraph_idx as usize).unwrap();
            from_subgraph.add_edge(
                from_index.index,
                SplitGraphEdgeWeight::Custom(edge.clone()),
                ext_target_idx.index,
            );
            let to_subgraph = self.subgraphs.get_mut(to_subgraph_idx as usize).unwrap();
            to_subgraph.add_edge(
                to_subgraph.root_index,
                SplitGraphEdgeWeight::ExternalSource {
                    source_id: from_id,
                    subgraph: from_subgraph_idx,
                    edge_kind: edge.kind(),
                },
                to_index.index,
            );
        }
    }

    pub fn edges_directed(
        &self,
        from_id: SplitGraphNodeId,
        direction: Direction,
    ) -> SplitGraphEdges<N, E, K> {
        let split_graph_index = self.node_id_to_index(from_id).unwrap();

        let subgraph = self
            .subgraphs
            .get(split_graph_index.subgraph as usize)
            .unwrap();

        let edges = subgraph
            .graph
            .edges_directed(split_graph_index.index, direction);

        SplitGraphEdges {
            subgraph,
            edges,
            from_id,
            direction,
        }
    }

    pub fn cleanup(&mut self) {
        for subgraph in self.subgraphs.iter_mut() {
            subgraph.cleanup();
        }
    }

    pub fn tiny_dot_to_file(&self, prefix: &str) {
        for (idx, subgraph) in self.subgraphs.iter().enumerate() {
            subgraph.tiny_dot_to_file(&format!("{prefix}-subgraph-{}", idx + 1));
        }
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
            Outgoing => self
                .subgraph
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
                }),
            Incoming => {
                let weight = edge_ref.weight();

                match weight {
                    SplitGraphEdgeWeight::Custom(_)
                    | SplitGraphEdgeWeight::Ordinal
                    | SplitGraphEdgeWeight::Ordering => self
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

    fn visit_map(self: &Self) -> Self::Map {
        HashSet::with_capacity(self.node_count())
    }

    fn reset_map(self: &Self, map: &mut Self::Map) {
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
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        u16,
    };

    use super::*;

    #[derive(Clone, PartialEq)]
    struct TestNodeWeight {
        id: SplitGraphNodeId,
        name: String,
        merkle_tree_hash: MerkleTreeHash,
    }

    impl std::fmt::Debug for TestNodeWeight {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "name = {}", self.name)
        }
    }

    impl CustomNodeWeight for TestNodeWeight {
        fn id(&self) -> SplitGraphNodeId {
            self.id
        }

        fn set_merkle_tree_hash(&mut self, hash: MerkleTreeHash) {
            self.merkle_tree_hash = hash;
        }

        fn merkle_tree_hash(&self) -> MerkleTreeHash {
            self.merkle_tree_hash
        }

        fn node_hash(&self) -> ContentHash {
            let mut hasher = ContentHash::hasher();
            hasher.update(&self.name.as_bytes());
            hasher.finalize()
        }

        fn ordered(&self) -> bool {
            false
        }
    }

    impl EdgeKind for () {}

    struct TestReadWriter {
        graphs: HashMap<SubGraphAddress, SubGraph<TestNodeWeight, (), ()>>,
    }

    #[async_trait]
    impl SubGraphReader<TestNodeWeight, (), ()> for TestReadWriter {
        type Error = SplitGraphError;

        async fn read_subgraph(
            &self,
            address: SubGraphAddress,
        ) -> Result<SubGraph<TestNodeWeight, (), ()>, SplitGraphError> {
            self.graphs
                .get(&address)
                .cloned()
                .ok_or(SplitGraphError::SubGraphRead(address, "not found".into()))
        }
    }

    #[async_trait]
    impl SubGraphWriter<TestNodeWeight, (), ()> for TestReadWriter {
        type Error = SplitGraphError;

        async fn write_subgraph(
            &mut self,
            _subgraph: &SubGraph<TestNodeWeight, (), ()>,
        ) -> Result<SubGraphAddress, SplitGraphError> {
            todo!()
        }
    }

    impl CustomEdgeWeight<()> for () {
        fn kind(&self) -> () {
            ()
        }
    }

    #[test]
    fn replace_node() {
        let reader_writer = TestReadWriter {
            graphs: HashMap::new(),
        };
        let mut splitgraph = SplitGraph::new(&reader_writer, &reader_writer, 2);

        let mut nodes: Vec<TestNodeWeight> = ["1", "2", "3", "4", "5", "6"]
            .into_iter()
            .map(|name| TestNodeWeight {
                id: Ulid::new(),
                name: name.to_string(),
                merkle_tree_hash: MerkleTreeHash::nil(),
            })
            .collect();

        for node in &nodes {
            splitgraph.add_or_replace_node(node.clone());
        }

        for node in nodes.iter_mut() {
            node.name = format!("{}-{}", node.name, node.id);
            splitgraph.add_or_replace_node(node.clone());
        }

        for node in &nodes {
            assert_eq!(
                Some(node),
                splitgraph.node_weight(node.id()).and_then(|n| n.custom())
            );
        }
    }

    #[test]
    fn test_cross_graph_edges() {
        let reader_writer = TestReadWriter {
            graphs: HashMap::new(),
        };

        let mut splitgraph = SplitGraph::new(&reader_writer, &reader_writer, 9);
        let mut unsplitgraph = SplitGraph::new(&reader_writer, &reader_writer, MAX_NODES as u16);

        let nodes = [
            "graph-1-a",
            "graph-1-b",
            "graph-1-c",
            "graph-1-d",
            "graph-1-e",
            "graph-1-f",
            "graph-1-g",
            "graph-1-h",
            "graph-1-i",
            "graph-2-j",
            "graph-2-k",
            "graph-2-l",
            "graph-2-m",
            "graph-2-n",
            "graph-2-o",
            "graph-2-p",
            "graph-2-q",
            "graph-2-r",
            "graph-3-s",
            "graph-3-t",
            "graph-3-u",
            "graph-3-v",
            "graph-3-w",
            "graph-3-x",
            "graph-3-y",
            "graph-3-z",
        ];

        let edges = [
            ("", "graph-1-a"),
            ("graph-1-a", "graph-1-b"),
            ("graph-1-a", "graph-1-c"),
            ("graph-1-c", "graph-1-d"),
            ("graph-1-d", "graph-1-e"),
            ("graph-1-e", "graph-1-f"),
            ("graph-1-f", "graph-1-g"),
            ("graph-1-g", "graph-1-h"),
            ("graph-1-h", "graph-1-i"),
            ("graph-1-a", "graph-2-j"),
            ("graph-1-a", "graph-2-k"),
            ("graph-1-b", "graph-2-k"),
            ("graph-1-c", "graph-2-k"),
            ("", "graph-2-l"),
            ("graph-2-l", "graph-1-b"),
            ("graph-2-l", "graph-1-c"),
            ("graph-2-l", "graph-1-d"),
            ("graph-2-l", "graph-2-m"),
            ("graph-2-l", "graph-2-n"),
            ("graph-2-l", "graph-2-o"),
            ("graph-2-l", "graph-2-p"),
            ("graph-2-p", "graph-2-q"),
            ("graph-2-q", "graph-2-r"),
            ("graph-2-q", "graph-3-s"),
            ("graph-2-q", "graph-3-t"),
            ("graph-3-t", "graph-1-b"),
            ("graph-3-t", "graph-3-u"),
            ("graph-3-t", "graph-3-v"),
            ("graph-3-t", "graph-3-w"),
            ("graph-3-t", "graph-3-x"),
            ("graph-3-t", "graph-3-y"),
            ("graph-3-t", "graph-3-z"),
        ];

        let mut name_to_id_map = HashMap::new();
        for name in &nodes {
            let id = Ulid::new();
            splitgraph.add_or_replace_node(TestNodeWeight {
                id,
                name: name.to_string(),
                merkle_tree_hash: MerkleTreeHash::nil(),
            });
            unsplitgraph.add_or_replace_node(TestNodeWeight {
                id,
                name: name.to_string(),
                merkle_tree_hash: MerkleTreeHash::nil(),
            });
            println!(
                "added node {name}:{id}, subgraphs: {}",
                splitgraph.subgraph_count()
            );
            name_to_id_map.insert(name, id);
        }

        let mut expected_outgoing_targets: HashMap<SplitGraphNodeId, HashSet<SplitGraphNodeId>> =
            HashMap::new();
        let mut split_expected_incoming_sources: HashMap<
            SplitGraphNodeId,
            HashSet<SplitGraphNodeId>,
        > = HashMap::new();
        let mut unsplit_expected_incoming_sources: HashMap<
            SplitGraphNodeId,
            HashSet<SplitGraphNodeId>,
        > = HashMap::new();

        for (from_name, to_name) in edges {
            let (split_from_id, unsplit_from_id) = if from_name.is_empty() {
                (splitgraph.root_id(), unsplitgraph.root_id())
            } else {
                (
                    name_to_id_map.get(&from_name).copied().unwrap(),
                    name_to_id_map.get(&from_name).copied().unwrap(),
                )
            };

            let to_id = name_to_id_map.get(&to_name).copied().unwrap();

            println!("adding edge {from_name}:{split_from_id} -> {to_name}:{to_id}");

            splitgraph.add_edge(dbg!(split_from_id), (), dbg!(to_id));
            println!("adding to unsplitgraph");
            unsplitgraph.add_edge(dbg!(unsplit_from_id), (), dbg!(to_id));

            expected_outgoing_targets
                .entry(split_from_id)
                .and_modify(|outgoing| {
                    outgoing.insert(to_id);
                })
                .or_insert(HashSet::from([to_id]));
            expected_outgoing_targets
                .entry(unsplit_from_id)
                .and_modify(|outgoing| {
                    outgoing.insert(to_id);
                })
                .or_insert(HashSet::from([to_id]));

            split_expected_incoming_sources
                .entry(to_id)
                .and_modify(|incoming| {
                    incoming.insert(split_from_id);
                })
                .or_insert(HashSet::from([split_from_id]));
            unsplit_expected_incoming_sources
                .entry(to_id)
                .and_modify(|incoming| {
                    incoming.insert(unsplit_from_id);
                })
                .or_insert(HashSet::from([unsplit_from_id]));
        }

        for from_name in &nodes {
            let (split_from_id, unsplit_from_id) = if from_name.is_empty() {
                (splitgraph.root_id(), unsplitgraph.root_id())
            } else {
                let id = name_to_id_map.get(&from_name).copied().unwrap();
                (id, id)
            };

            let outgoing_targets: HashSet<SplitGraphNodeId> = splitgraph
                .edges_directed(split_from_id, Outgoing)
                .map(|edge_ref| edge_ref.target())
                .collect();
            let unsplit_outgoing_targets: HashSet<SplitGraphNodeId> = unsplitgraph
                .edges_directed(unsplit_from_id, Outgoing)
                .map(|edge_ref| edge_ref.target())
                .collect();

            let incoming_sources: HashSet<SplitGraphNodeId> = splitgraph
                .edges_directed(split_from_id, Incoming)
                .map(|edge_ref| edge_ref.source())
                .collect();
            let unsplit_incoming_sources: HashSet<SplitGraphNodeId> = unsplitgraph
                .edges_directed(unsplit_from_id, Incoming)
                .map(|edge_ref| edge_ref.source())
                .collect();

            let name = splitgraph
                .node_weight(split_from_id)
                .and_then(|n| n.custom().map(|n| n.name.as_str()))
                .unwrap();

            println!(
                "{split_from_id} ({name}):\n\t{:?}\n\t{:?}",
                outgoing_targets, incoming_sources
            );

            if outgoing_targets.is_empty() {
                assert!(expected_outgoing_targets.get(&split_from_id).is_none());
                assert!(expected_outgoing_targets.get(&unsplit_from_id).is_none());
            } else {
                assert_eq!(
                    expected_outgoing_targets
                        .get(&split_from_id)
                        .cloned()
                        .unwrap(),
                    outgoing_targets
                );
                assert_eq!(
                    expected_outgoing_targets
                        .get(&unsplit_from_id)
                        .cloned()
                        .unwrap(),
                    unsplit_outgoing_targets
                );

                for target_id in outgoing_targets {
                    if let Some(node) = splitgraph.node_weight(target_id).and_then(|n| n.custom()) {
                        assert_eq!(
                            Some(target_id),
                            name_to_id_map.get(&node.name.as_str()).copied()
                        );
                    }
                }
            }

            if incoming_sources.is_empty() {
                assert!(split_expected_incoming_sources
                    .get(&split_from_id)
                    .is_none());
                assert!(unsplit_expected_incoming_sources
                    .get(&unsplit_from_id)
                    .is_none());
            } else {
                assert_eq!(
                    split_expected_incoming_sources
                        .get(&split_from_id)
                        .cloned()
                        .unwrap(),
                    incoming_sources
                );
                assert_eq!(
                    unsplit_expected_incoming_sources
                        .get(&unsplit_from_id)
                        .cloned()
                        .unwrap(),
                    unsplit_incoming_sources
                );
            }
        }

        // splitgraph.tiny_dot_to_file("before-removal");
        // unsplitgraph.tiny_dot_to_file("unsplitgraph");

        let graph_2_q = "graph-2-q";
        let graph_3_t = "graph-3-t";
        let graph_3_s = "graph-3-s";
        let graph_2_q_id = name_to_id_map.get(&graph_2_q).copied().unwrap();
        let graph_3_t_id = name_to_id_map.get(&graph_3_t).copied().unwrap();
        let graph_3_s_id = name_to_id_map.get(&graph_3_s).copied().unwrap();
        splitgraph.remove_edge(graph_2_q_id, (), graph_3_t_id);
        unsplitgraph.remove_edge(graph_2_q_id, (), graph_3_t_id);
        splitgraph.cleanup();
        unsplitgraph.cleanup();

        // splitgraph.tiny_dot_to_file("after-removal");

        assert!(splitgraph.node_weight(graph_2_q_id).is_some());
        assert!(unsplitgraph.node_weight(graph_2_q_id).is_some());
        assert!(splitgraph.node_weight(graph_3_s_id).is_some());
        assert!(unsplitgraph.node_weight(graph_3_s_id).is_some());

        for graph_3_name in [
            "graph-3-t",
            "graph-3-u",
            "graph-3-v",
            "graph-3-w",
            "graph-3-x",
            "graph-3-y",
            "graph-3-z",
        ] {
            let id = name_to_id_map.get(&graph_3_name).copied().unwrap();
            assert!(splitgraph.node_weight(id).is_none());
            assert!(unsplitgraph.node_weight(id).is_none());
        }
    }
}
