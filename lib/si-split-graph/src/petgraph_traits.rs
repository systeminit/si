use std::{
    collections::{
        HashSet,
        btree_map,
    },
    marker::PhantomData,
};

use petgraph::{
    prelude::*,
    stable_graph,
};
use telemetry::prelude::*;

use crate::{
    CustomEdgeWeight,
    CustomNodeWeight,
    EdgeKind,
    SplitGraph,
    SplitGraphEdgeIndex,
    SplitGraphEdgeWeight,
    SplitGraphNodeId,
    SplitGraphNodeWeight,
    SubGraph,
    SubGraphIndex,
    SubGraphNodeIndex,
};

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

    pub fn triplet(&self) -> (SplitGraphNodeId, K, SplitGraphNodeId) {
        (self.source(), self.weight().kind(), self.target())
    }
}

pub struct SplitGraphEdges<'a, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub(super) this_subgraph: &'a SubGraph<N, E, K>,
    pub(super) subgraphs: &'a [SubGraph<N, E, K>],
    pub(super) edges:
        stable_graph::Edges<'a, SplitGraphEdgeWeight<E, K, N>, Directed, SubGraphIndex>,
    pub(super) from_id: SplitGraphNodeId,
    pub(super) direction: Direction,
    pub(super) debug: bool,
}

pub struct RawSplitGraphEdges<'a, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub(super) this_subgraph: &'a SubGraph<N, E, K>,
    pub(super) edges:
        stable_graph::Edges<'a, SplitGraphEdgeWeight<E, K, N>, Directed, SubGraphIndex>,
}

#[derive(Debug)]
pub struct RawSplitGraphEdgeReference<'a, E, K, N>
where
    E: 'a + CustomEdgeWeight<K>,
    N: CustomNodeWeight,
    K: EdgeKind,
{
    source_id: SplitGraphNodeId,
    target_id: SplitGraphNodeId,
    weight: &'a SplitGraphEdgeWeight<E, K, N>,
}

impl<'a, E, K, N> RawSplitGraphEdgeReference<'a, E, K, N>
where
    E: 'a + CustomEdgeWeight<K>,
    N: CustomNodeWeight,
    K: EdgeKind,
{
    pub fn weight(&self) -> &'a SplitGraphEdgeWeight<E, K, N> {
        self.weight
    }

    pub fn source(&self) -> SplitGraphNodeId {
        self.source_id
    }

    pub fn target(&self) -> SplitGraphNodeId {
        self.target_id
    }
}

pub struct SplitGraphNeighbors<'a, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    pub(super) subgraph: Option<&'a SubGraph<N, E, K>>,
    pub(super) direction: Direction,
    pub(super) incoming_edges:
        Option<stable_graph::Edges<'a, SplitGraphEdgeWeight<E, K, N>, Directed, SubGraphIndex>>,
    pub(super) outgoing_neighbors:
        Option<stable_graph::Neighbors<'a, SplitGraphEdgeWeight<E, K, N>, usize>>,
}

impl<N, E, K> Iterator for SplitGraphNeighbors<'_, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Item = SplitGraphNodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_incoming_neighbor = || {
            self.incoming_edges
                .as_mut()
                .and_then(|edges| edges.next())
                .and_then(|incoming_edge| match incoming_edge.weight() {
                    SplitGraphEdgeWeight::Custom(_)
                    | SplitGraphEdgeWeight::Ordering
                    | SplitGraphEdgeWeight::Ordinal => self
                        .subgraph
                        .and_then(|subgraph| subgraph.graph.node_weight(incoming_edge.source()))
                        .map(|weight| match weight {
                            SplitGraphNodeWeight::ExternalTarget { target, .. } => *target,
                            _ => weight.id(),
                        }),
                    SplitGraphEdgeWeight::ExternalSource { source_id, .. } => Some(*source_id),
                })
        };

        let mut next_outgoing_neighbor = || {
            self.outgoing_neighbors
                .as_mut()
                .and_then(|neighbors| neighbors.next())
                .and_then(|outgoing_neighbor| {
                    self.subgraph.and_then(|subgraph| {
                        subgraph
                            .graph
                            .node_weight(outgoing_neighbor)
                            .map(|weight| match weight {
                                SplitGraphNodeWeight::Custom(_)
                                | SplitGraphNodeWeight::Ordering { .. }
                                | SplitGraphNodeWeight::GraphRoot { .. }
                                | SplitGraphNodeWeight::SubGraphRoot { .. } => weight.id(),
                                SplitGraphNodeWeight::ExternalTarget { target, .. } => *target,
                            })
                    })
                })
        };

        match self.direction {
            Incoming => next_incoming_neighbor(),
            Outgoing => next_outgoing_neighbor(),
        }
    }
}

impl<'a, N, E, K> SplitGraphEdges<'a, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    fn subgraph_for_node(&self, node_id: SplitGraphNodeId) -> Option<&'a SubGraph<N, E, K>> {
        self.subgraphs
            .iter()
            .find(|subgraph| subgraph.node_id_to_index(node_id).is_some())
    }
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
                    ..
                } => self.subgraph_for_node(*source_id).and_then(|ext_subgraph| {
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
                                    ext_subgraph.graph.node_weight(subgraph_edge_ref.target());
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
                            warn!("missing node for external source: {source_id:?}");
                            None
                        }
                    }
                }),
                SplitGraphEdgeWeight::Ordinal | SplitGraphEdgeWeight::Ordering => self.next(),
            },
        })
    }
}

impl<'a, N, E, K> Iterator for RawSplitGraphEdges<'a, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Item = RawSplitGraphEdgeReference<'a, E, K, N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.edges.next().and_then(|edge_ref| {
            self.this_subgraph
                .graph
                .node_weight(edge_ref.source())
                .zip(self.this_subgraph.graph.node_weight(edge_ref.target()))
                .map(|(source_node, target_node)| RawSplitGraphEdgeReference {
                    source_id: source_node.id(),
                    target_id: target_node.id(),
                    weight: edge_ref.weight(),
                })
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
    type NodeId = SplitGraphNodeId;
}

impl<N, E, K> petgraph::visit::Visitable for SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Map = HashSet<SplitGraphNodeId>;

    fn visit_map(&self) -> Self::Map {
        HashSet::with_capacity(self.node_count())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
    }
}

pub struct SplitGraphNodeIdIter<'a, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    cursor: usize,
    split_graph: &'a SplitGraph<N, E, K>,
    iterators: Vec<btree_map::Keys<'a, SplitGraphNodeId, SubGraphNodeIndex>>,
}

impl<N, E, K> Iterator for SplitGraphNodeIdIter<'_, N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Item = SplitGraphNodeId;

    fn next(&mut self) -> Option<Self::Item> {
        enum IterControl {
            NextIter,
            Found(SplitGraphNodeId),
            NextSubgraph,
        }

        while self.cursor < self.iterators.len() {
            loop {
                let control = match self
                    .iterators
                    .get_mut(self.cursor)
                    .and_then(|iter| iter.next().copied())
                {
                    Some(next_id) => match self.split_graph.raw_node_weight(next_id) {
                        Some(SplitGraphNodeWeight::ExternalTarget { .. }) => IterControl::NextIter,
                        Some(_) => IterControl::Found(next_id),
                        None => IterControl::NextSubgraph,
                    },
                    None => IterControl::NextSubgraph,
                };

                match control {
                    IterControl::NextIter => {
                        continue;
                    }
                    IterControl::Found(id) => return Some(id),
                    IterControl::NextSubgraph => {
                        self.cursor += 1;
                        break;
                    }
                }
            }
        }

        None
    }
}

impl<'a, N, E, K> petgraph::visit::IntoNeighborsDirected for &'a SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type NeighborsDirected = SplitGraphNeighbors<'a, N, E, K>;

    fn neighbors_directed(self, n: Self::NodeId, d: Direction) -> Self::Neighbors {
        self.raw_neighbors_directed(n, d)
    }
}

impl<'a, N, E, K> petgraph::visit::IntoNeighbors for &'a SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type Neighbors = SplitGraphNeighbors<'a, N, E, K>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.raw_neighbors(a)
    }
}

impl<'a, N, E, K> petgraph::visit::IntoNodeIdentifiers for &'a SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type NodeIdentifiers = SplitGraphNodeIdIter<'a, N, E, K>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        SplitGraphNodeIdIter {
            cursor: 0,
            split_graph: self,
            iterators: self
                .subgraphs
                .iter()
                .map(|subgraph| subgraph.node_index_by_id.keys())
                .collect(),
        }
    }
}

impl<N, E, K> petgraph::visit::Data for &SplitGraph<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    type NodeWeight = SplitGraphNodeWeight<N>;
    type EdgeWeight = SplitGraphEdgeWeight<E, K, N>;
}
