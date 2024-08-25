
use petgraph::{prelude::*, stable_graph::{EdgeReference, Edges, EdgesConnecting}};
use si_events::ulid::Ulid;
use crate::{EdgeWeight, EdgeWeightKindDiscriminants};
use super::*;
use super::super::node_weight::NodeWeight;

#[derive(Copy, Clone)]
pub struct GraphyNode<'a> {
    pub(super) graph: &'a GraphyContext<'a>,
    pub(super) index: NodeIndex
}

impl<'a> GraphyNode<'a> {
    pub fn id(self) -> GraphyResult<Ulid> {
        self.graph.node_index_to_id(self.index)
    }
    pub fn graph(self) -> &'a StableGraph<NodeWeight, EdgeWeight> {
        self.graph.petgraph()
    }
    pub fn node_weight(self) -> GraphyResult<&'a NodeWeight> {
        self.graph().node_weight(self.index).ok_or(GraphyError::NodeNotFound(self.index))
    }
    pub fn all_edges(self) -> Edges<'a, EdgeWeight, Directed> {
        self.graph().edges(self.index)
    }
    pub fn all_edges_directed(self, direction: Direction) -> Edges<'a, EdgeWeight, Directed> {
        self.graph().edges_directed(self.index, direction)
    }
    pub fn all_incoming_edges(self) -> Edges<'a, EdgeWeight, Directed> {
        self.all_edges_directed(Direction::Incoming)
    }
    pub fn incoming_edges(self, kind: EdgeWeightKindDiscriminants) -> EdgesOfKind<'a> {
        EdgesOfKind { edges: self.all_incoming_edges(), kind }
    }
    pub fn all_outgoing_edges(self) -> Edges<'a, EdgeWeight, Directed> {
        self.all_edges_directed(Direction::Outgoing)
    }
    pub fn outgoing_edges(self, kind: EdgeWeightKindDiscriminants) -> EdgesOfKind<'a> {
        EdgesOfKind { edges: self.all_outgoing_edges(), kind }
    }

    pub fn source_nodes(self, kind: EdgeWeightKindDiscriminants) -> Sources<'a, EdgesOfKind<'a>> {
        Sources { edges: self.outgoing_edges(kind), graph: self.graph }
    }
    pub fn source_node_opt(self, kind: EdgeWeightKindDiscriminants) -> GraphyResult<GraphyNode<'a>> {
        self.matching_source(kind)
    }
    pub fn source_node(self, kind: EdgeWeightKindDiscriminants) -> GraphyResult<GraphyNode<'a>> {
        self.matching_source(kind)
    }
    pub fn matching_sources<T: TryFrom<GraphyNode<'a>>>(self, kind: EdgeWeightKindDiscriminants) -> impl Iterator<Item = T>+'a {
        self.source_nodes(kind).filter_map(|node| T::try_from(node).ok())
    }
    pub fn matching_source_opt<T: TryFrom<GraphyNode<'a>>>(self, kind: EdgeWeightKindDiscriminants) -> GraphyResult<Option<T>> {
        let mut results = self.matching_sources(kind);
        let result = results.next();
        if results.next().is_some() {
            return Err(GraphyError::MultipleMatchingNodes(self.index, kind));
        }
        Ok(result)
    }
    pub fn matching_source<T: TryFrom<GraphyNode<'a>>>(self, kind: EdgeWeightKindDiscriminants) -> GraphyResult<T> {
        self.matching_source_opt(kind)?.ok_or(GraphyError::NoMatchingNode(self.index, kind))
    }

    pub fn target_nodes(self, kind: EdgeWeightKindDiscriminants) -> Targets<'a, EdgesOfKind<'a>> {
        Targets { edges: self.outgoing_edges(kind), graph: self.graph }
    }
    pub fn target_node_opt(self, kind: EdgeWeightKindDiscriminants) -> GraphyResult<Option<GraphyNode<'a>>> {
        self.matching_target_opt(kind)
    }
    pub fn target_node(self, kind: EdgeWeightKindDiscriminants) -> GraphyResult<GraphyNode<'a>> {
        self.matching_target(kind)
    }
    pub fn matching_targets<T: TryFrom<GraphyNode<'a>>>(self, kind: EdgeWeightKindDiscriminants) -> impl Iterator<Item = T>+'a {
        self.target_nodes(kind).filter_map(|node| T::try_from(node).ok())
    }
    pub fn matching_target_opt<T: TryFrom<GraphyNode<'a>>>(self, kind: EdgeWeightKindDiscriminants) -> GraphyResult<Option<T>> {
        let mut results = self.matching_targets(kind);
        let result = results.next();
        if results.next().is_some() {
            return Err(GraphyError::MultipleMatchingNodes(self.index, kind));
        }
        Ok(result)
    }
    pub fn matching_target<T: TryFrom<GraphyNode<'a>>>(self, kind: EdgeWeightKindDiscriminants) -> GraphyResult<T> {
        self.matching_target_opt(kind)?.ok_or(GraphyError::NoMatchingNode(self.index, kind))
    }


    pub fn exists(self) -> bool {
        self.graph().contains_node(self.index)
    }
    pub fn has_edge_to(self, other: impl Into<NodeIndex>) -> bool {
        self.graph().contains_edge(self.index, other.into())
    }
    pub fn edges_with(self, other: impl Into<NodeIndex>) -> EdgesConnecting<'a, EdgeWeight, Directed>{
        self.graph().edges_connecting(self.index, other.into())
    }
}

pub struct EdgesOfKind<'a> {
    edges: Edges<'a, EdgeWeight, Directed>,
    kind: EdgeWeightKindDiscriminants,
}

impl<'a> Iterator for EdgesOfKind<'a> {
    type Item = EdgeReference<'a, EdgeWeight>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(edge) = self.edges.next() {
            if EdgeWeightKindDiscriminants::from(edge.weight().kind()) == self.kind {
                return Some(edge);
            }
        }
        None
    }
}

pub struct Sources<'a, E: Iterator<Item = EdgeReference<'a, EdgeWeight>>> {
    graph: &'a GraphyContext<'a>,
    edges: E
}

impl<'a, E: Iterator<Item = EdgeReference<'a, EdgeWeight>>> Iterator for Sources<'a, E> {
    type Item = GraphyNode<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.edges.next().map(|edge| self.graph.node(edge.source()))
    }
}

pub struct Targets<'a, E: Iterator<Item = EdgeReference<'a, EdgeWeight>>> {
    graph: &'a GraphyContext<'a>,
    edges: E
}

impl<'a, E: Iterator<Item = EdgeReference<'a, EdgeWeight>>> Iterator for Targets<'a, E> {
    type Item = GraphyNode<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.edges.next().map(|edge| self.graph.node(edge.target()))
    }
}

