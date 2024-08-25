use std::marker::PhantomData;
use crate::workspace_snapshot::node_weight::{NodeWeight, OrderingNodeWeight};
use crate::EdgeWeightKindDiscriminants;
use super::*;

#[derive(Copy, Clone)]
pub struct Ordering<'a, T: GraphyNodeType<'a>+'a, P: GraphyNodeType<'a>+'a>(pub(super) GraphyNode<'a>, PhantomData<(T, P)>);

impl<'a, T: GraphyNodeType<'a>+'a, P: GraphyNodeType<'a>> GraphyNodeType<'a> for Ordering<'a, T, P> {
    type Id = Ulid;
    type Weight = OrderingNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Ordering }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node, PhantomData::default()) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Ordering(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a, T: GraphyNodeType<'a>+'a, P: GraphyNodeType<'a>+'a> Ordering<'a, T, P> {
    //
    // Children
    //
    pub fn children(self) -> impl Iterator<Item = T>+'a {
        self.0.target_nodes(EdgeWeightKindDiscriminants::Ordinal).map(T::construct)
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<P> {
        self.0.source_node(EdgeWeightKindDiscriminants::Ordering).map(P::construct)
    }
}

impl<'a, T: GraphyNodeType<'a>+'a, P: GraphyNodeType<'a>+'a> From<Ordering<'a, T, P>> for GraphyNode<'a> {
    fn from(ordering: Ordering<'a, T, P>) -> Self {
        ordering.0
    }
}

impl<'a, T: GraphyNodeType<'a>+'a, P: GraphyNodeType<'a>+'a> AsRef<GraphyNode<'a>> for Ordering<'a, T, P> {
    fn as_ref(&self) -> &GraphyNode<'a> {
        &self.0
    }
}

impl<'a, T: GraphyNodeType<'a>+'a, P: GraphyNodeType<'a>+'a> TryFrom<GraphyNode<'a>> for Ordering<'a, T, P> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self::construct(node);
        result.weight()?;
        Ok(result)
    }
}
