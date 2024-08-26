use si_events::ulid::Ulid;
use std::marker::PhantomData;

use super::{GraphyError, GraphyNode, GraphyNodeRef, GraphyResult, GraphyItertools as _};
use crate::{
    workspace_snapshot::node_weight::{NodeWeight, OrderingNodeWeight},
    EdgeWeightKind,
};

#[derive(Copy, Clone, derive_more::AsRef, derive_more::Deref)]
pub struct Ordering<'a, T: GraphyNode<'a> + 'a, P: GraphyNode<'a> + 'a = T>(
    #[deref] GraphyNodeRef<'a>,
    PhantomData<(T, P)>,
);

impl<'a, T: GraphyNode<'a> + 'a, P: GraphyNode<'a>> GraphyNode<'a> for Ordering<'a, T, P> {
    type Id = Ulid;
    type Weight = OrderingNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into(), PhantomData)
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Ordering(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a, T: GraphyNode<'a> + 'a, P: GraphyNode<'a> + 'a> Ordering<'a, T, P> {
    /// Children in order
    pub fn children(self) -> GraphyResult<impl Iterator<Item = GraphyResult<T>> + 'a> {
        let order = self.ordered_child_ids()?.into_iter();
        Ok(order.map(move |id| self.graph.node_ref_by_id(*id).map(T::as_node)))
    }
    /// Unordered children
    pub fn unordered_children(self) -> impl Iterator<Item = T> + 'a {
        self.targets(EdgeWeightKind::Ordinal)
    }

    //
    // Weight properties
    //
    pub fn ordered_child_ids(self) -> GraphyResult<&'a Vec<Ulid>> {
        Ok(&self.weight()?.order())
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<P> {
        self.sources(EdgeWeightKind::Ordering).single()
    }
}

impl<'a, T: GraphyNode<'a> + 'a, P: GraphyNode<'a> + 'a> From<Ordering<'a, T, P>>
    for GraphyNodeRef<'a>
{
    fn from(ordering: Ordering<'a, T, P>) -> Self {
        ordering.0
    }
}
