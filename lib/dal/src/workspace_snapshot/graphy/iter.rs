use std::marker::PhantomData;

use super::{
    DirectionalEdge, GraphyContext, GraphyError, GraphyNode, GraphyNodeRef, GraphyResult,
    KindMatcher,
};
use crate::EdgeWeight;
use petgraph::{
    prelude::*,
    stable_graph::{EdgeReference, Edges},
};

pub trait GraphyItertools: Iterator + Sized {
    fn of_kind<'a, K: KindMatcher>(self, kind: K) -> EdgesOfKind<'a, K, Self::Item, Self>
        where Self::Item: DirectionalEdge<'a>;
    fn as_nodes<'a, T: GraphyNode<'a>>(self) -> AsNodes<'a, T, Self>
    where
        Self::Item: Into<GraphyNodeRef<'a>>;
    fn try_as_nodes<'a, T: GraphyNode<'a>>(self) -> TryAsNodes<'a, T, Self>
    where
        Self::Item: Into<GraphyNodeRef<'a>>;
    fn of_type<'a, T: GraphyNode<'a>>(self) -> SkipErr<T, GraphyError, TryAsNodes<'a, T, Self>>
    where
        Self::Item: Into<GraphyNodeRef<'a>>,
    {
        self.try_as_nodes().skip_err()
    }
    fn single(self) -> GraphyResult<Self::Item>;
    fn optional(self) -> GraphyResult<Option<Self::Item>>;
}

pub trait ResultItertools<T, E>: Iterator<Item = Result<T, E>> + Sized {
    fn skip_err(self) -> SkipErr<T, E, Self>;
}

pub struct GraphyEdges<
    'a,
    T: DirectionalEdge<'a>,
    I: Iterator<Item = EdgeReference<'a, EdgeWeight>> + Sized = Edges<'a, EdgeWeight, Directed>,
> {
    edges: I,
    graph: &'a GraphyContext<'a>,
    phantom: PhantomData<T>,
}

pub struct EdgesOfKind<
    'a,
    K: KindMatcher,
    T: DirectionalEdge<'a>,
    I: Iterator<Item = T> + Sized = GraphyEdges<'a, T>,
> {
    edges: I,
    kind: K,
    phantom: PhantomData<&'a T>,
}

pub struct AsNodes<'a, T: GraphyNode<'a>, I: Iterator<Item: Into<GraphyNodeRef<'a>>> + Sized>(
    I,
    PhantomData<&'a T>,
);

pub struct TryAsNodes<'a, T: GraphyNode<'a>, I: Iterator<Item: Into<GraphyNodeRef<'a>>> + Sized>(
    I,
    PhantomData<&'a T>,
);

pub struct SkipErr<T, E, I: Iterator<Item = Result<T, E>> + Sized>(I);

impl<I: Iterator + Sized> GraphyItertools for I {
    fn of_kind<'a, K: KindMatcher>(self, kind: K) -> EdgesOfKind<'a, K, Self::Item, Self> where Self::Item: DirectionalEdge<'a> {
        EdgesOfKind {
            edges: self,
            kind,
            phantom: PhantomData,
        }
    }

    fn as_nodes<'a, T: GraphyNode<'a>>(self) -> AsNodes<'a, T, Self>
    where
        Self::Item: Into<GraphyNodeRef<'a>>,
    {
        AsNodes(self, PhantomData)
    }
    fn try_as_nodes<'a, T: GraphyNode<'a>>(self) -> TryAsNodes<'a, T, Self>
    where
        Self::Item: Into<GraphyNodeRef<'a>>,
    {
        TryAsNodes(self, PhantomData)
    }
    fn of_type<'a, T: GraphyNode<'a>>(self) -> SkipErr<T, GraphyError, TryAsNodes<'a, T, Self>>
    where
        Self::Item: Into<GraphyNodeRef<'a>>,
    {
        self.try_as_nodes().skip_err()
    }
    fn single(self) -> GraphyResult<Self::Item> {
        self.optional()?.ok_or(GraphyError::NoNodes)
    }
    fn optional(mut self) -> GraphyResult<Option<Self::Item>> {
        match self.next() {
            Some(result) => match self.next() {
                None => Ok(Some(result)),
                Some(_) => Err(GraphyError::MultipleNodes),
            },
            None => Ok(None),
        }
    }
}

impl<T, E, I: Iterator<Item = Result<T, E>> + Sized> ResultItertools<T, E> for I {
    fn skip_err(self) -> SkipErr<T, E, Self> {
        SkipErr(self)
    }
}

impl<'a, T: DirectionalEdge<'a>, I: Iterator<Item = EdgeReference<'a, EdgeWeight>> + Sized>
    GraphyEdges<'a, T, I>
{
    pub fn new(edges: I, graph: &'a GraphyContext<'a>) -> Self {
        Self {
            edges,
            graph,
            phantom: PhantomData,
        }
    }
}

impl<'a, T: DirectionalEdge<'a>, I: Iterator<Item = EdgeReference<'a, EdgeWeight>> + Sized> Iterator
    for GraphyEdges<'a, T, I>
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.edges
            .next()
            .map(|edge| T::construct(self.graph.edge_ref(edge)))
    }
}

impl<'a, K: KindMatcher, T: DirectionalEdge<'a>, I: Iterator<Item = T> + Sized> Iterator
    for EdgesOfKind<'a, K, T, I>
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(edge) = self.edges.next() {
            if self.kind.matches(edge.as_ref().kind()) {
                return Some(edge);
            }
        }
        None
    }
}

impl<'a, T: GraphyNode<'a>, I: Iterator<Item: Into<GraphyNodeRef<'a>>> + Sized> Iterator
    for AsNodes<'a, T, I>
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|node| T::as_node(node.into()))
    }
}

impl<'a, T: GraphyNode<'a>, I: Iterator<Item: Into<GraphyNodeRef<'a>>> + Sized> Iterator
    for TryAsNodes<'a, T, I>
{
    type Item = Result<T, GraphyError>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|node| T::try_as_node(node.into()))
    }
}

impl<T, E, I: Iterator<Item = Result<T, E>> + Sized> Iterator for SkipErr<T, E, I> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.0.next() {
            if let Ok(next) = next {
                return Some(next);
            }
        }
        None
    }
}
