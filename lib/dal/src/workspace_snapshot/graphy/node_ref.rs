use super::{
    AsNodes, EdgesOfKind, GraphyContext, GraphyEdges, GraphyItertools as _, GraphyError,
    GraphyNode, GraphyResult,
};
use crate::{
    workspace_snapshot::node_weight::NodeWeight, EdgeWeight, EdgeWeightKind,
    EdgeWeightKindDiscriminants,
};
use petgraph::{
    prelude::*,
    stable_graph::{EdgeReference, EdgesConnecting},
};
use si_events::ulid::Ulid;

#[derive(Copy, Clone)]
pub struct GraphyNodeRef<'a> {
    pub(super) graph: &'a GraphyContext<'a>,
    pub(super) index: NodeIndex,
}

#[derive(Copy, Clone)]
pub struct GraphyEdgeRef<'a> {
    pub(super) graph: &'a GraphyContext<'a>,
    pub(super) edge: EdgeReference<'a, EdgeWeight>,
}

pub trait KindMatcher {
    fn matches(&self, kind: &EdgeWeightKind) -> bool;
}

impl KindMatcher for EdgeWeightKindDiscriminants {
    fn matches(&self, kind: &EdgeWeightKind) -> bool {
        self == &kind.into()
    }
}

impl KindMatcher for EdgeWeightKind {
    fn matches(&self, kind: &EdgeWeightKind) -> bool {
        self == kind
    }
}

pub trait DirectionalEdge<'a>:
    Copy + Clone + AsRef<GraphyEdgeRef<'a>> + Into<GraphyEdgeRef<'a>>
{
    fn construct(edge: GraphyEdgeRef<'a>) -> Self;
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Outgoing<'a>(pub(super) GraphyEdgeRef<'a>);

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Incoming<'a>(pub(super) GraphyEdgeRef<'a>);

impl<'a> GraphyNode<'a> for GraphyNodeRef<'a> {
    type Id = Ulid;
    type Weight = NodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        node.into()
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Ok(weight)
    }
}

impl<'a> GraphyNodeRef<'a> {
    pub fn id(self) -> GraphyResult<Ulid> {
        self.graph.node_index_to_id(self.index)
    }
    pub fn graph(self) -> &'a StableGraph<NodeWeight, EdgeWeight> {
        self.graph.petgraph()
    }
    pub fn node_weight(self) -> GraphyResult<&'a NodeWeight> {
        self.graph()
            .node_weight(self.index)
            .ok_or(GraphyError::NodeNotFound(self.index))
    }
    pub fn incoming_edges(self) -> GraphyEdges<'a, Incoming<'a>> {
        GraphyEdges::new(
            self.graph().edges_directed(self.index, Direction::Incoming),
            self.graph,
        )
    }
    pub fn outgoing_edges(self) -> GraphyEdges<'a, Outgoing<'a>> {
        GraphyEdges::new(
            self.graph().edges_directed(self.index, Direction::Outgoing),
            self.graph,
        )
    }
    pub fn targets<T: GraphyNode<'a>, K: KindMatcher>(
        self,
        of_kind: K,
    ) -> AsNodes<'a, T, EdgesOfKind<'a, K, Outgoing<'a>, GraphyEdges<'a, Outgoing<'a>>>> {
        self.outgoing_edges().of_kind(of_kind).as_nodes()
    }
    pub fn sources<T: GraphyNode<'a>, K: KindMatcher>(
        self,
        of_kind: K,
    ) -> AsNodes<'a, T, EdgesOfKind<'a, K, Incoming<'a>, GraphyEdges<'a, Incoming<'a>>>> {
        self.incoming_edges().of_kind(of_kind).as_nodes()
    }

    pub fn exists(self) -> bool {
        self.graph().contains_node(self.index)
    }
    pub fn has_edge_to(self, other: impl Into<NodeIndex>) -> bool {
        self.graph().contains_edge(self.index, other.into())
    }
    pub fn edges_with(
        self,
        other: impl Into<NodeIndex>,
    ) -> EdgesConnecting<'a, EdgeWeight, Directed> {
        self.graph().edges_connecting(self.index, other.into())
    }
}

impl<'a> AsRef<GraphyNodeRef<'a>> for GraphyNodeRef<'a> {
    fn as_ref(&self) -> &GraphyNodeRef<'a> {
        self
    }
}

impl<'a> GraphyEdgeRef<'a> {
    pub fn graph(self) -> &'a GraphyContext<'a> {
        self.graph
    }
    pub fn kind(self) -> &'a EdgeWeightKind {
        self.edge.weight().kind()
    }
    pub fn discriminant(self) -> EdgeWeightKindDiscriminants {
        self.kind().into()
    }
    pub fn source(self) -> GraphyNodeRef<'a> {
        self.graph.node_ref(self.edge.source())
    }
    pub fn target(self) -> GraphyNodeRef<'a> {
        self.graph.node_ref(self.edge.target())
    }
}

impl<'a> DirectionalEdge<'a> for Outgoing<'a> {
    fn construct(edge: GraphyEdgeRef<'a>) -> Self {
        Self(edge)
    }
}

impl<'a> DirectionalEdge<'a> for Incoming<'a> {
    fn construct(edge: GraphyEdgeRef<'a>) -> Self {
        Self(edge)
    }
}

impl<'a> Outgoing<'a> {
    pub fn node(self) -> GraphyNodeRef<'a> {
        self.0.target()
    }
}

impl<'a> From<Outgoing<'a>> for GraphyNodeRef<'a> {
    fn from(edge: Outgoing<'a>) -> Self {
        edge.node()
    }
}

impl<'a> Incoming<'a> {
    pub fn node(self) -> GraphyNodeRef<'a> {
        self.0.source()
    }
}

impl<'a> From<Incoming<'a>> for GraphyNodeRef<'a> {
    fn from(edge: Incoming<'a>) -> Self {
        edge.node()
    }
}
