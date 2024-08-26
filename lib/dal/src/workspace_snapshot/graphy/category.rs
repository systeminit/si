use super::{
    AsNodes, EdgesOfKind, GraphyCategoryElement, GraphyError, GraphyNode, GraphyNodeRef,
    GraphyResult, GraphyItertools as _, Outgoing, Root,
};
use crate::{
    workspace_snapshot::node_weight::{
        category_node_weight::CategoryNodeKind, CategoryNodeWeight, NodeWeight,
    },
    EdgeWeightKindDiscriminants,
};
use si_events::ulid::Ulid;
use std::marker::PhantomData;

///
/// Category containing all nodes of a given type.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct AnyCategory<'a>(GraphyNodeRef<'a>);

impl<'a> AnyCategory<'a> {
    /// The category of this node.
    pub fn category_kind(self) -> GraphyResult<CategoryNodeKind> {
        Ok(self.weight()?.kind())
    }

    /// All nodes of the given type.
    pub fn all(
        self,
    ) -> AsNodes<'a, GraphyNodeRef<'a>, EdgesOfKind<'a, EdgeWeightKindDiscriminants, Outgoing<'a>>>
    {
        self.targets(EdgeWeightKindDiscriminants::Use)
    }

    //
    // Backreferences
    //
    pub fn root(self) -> GraphyResult<Root<'a>> {
        self.sources(EdgeWeightKindDiscriminants::Use).single()
    }
}

impl<'a> GraphyNode<'a> for AnyCategory<'a> {
    type Id = Ulid;
    type Weight = CategoryNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Category(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

///
/// Category containing all nodes of a given type.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Category<'a, T: GraphyCategoryElement<'a>>(#[deref] AnyCategory<'a>, PhantomData<T>);

impl<'a, T: GraphyCategoryElement<'a>> Category<'a, T> {
    /// All nodes of the given type.
    pub fn all(self) -> AsNodes<'a, T, EdgesOfKind<'a, EdgeWeightKindDiscriminants, Outgoing<'a>>> {
        self.targets(EdgeWeightKindDiscriminants::Use)
    }
}

impl<'a, T: GraphyCategoryElement<'a>> IntoIterator for Category<'a, T> {
    type Item = T;
    type IntoIter = AsNodes<'a, T, EdgesOfKind<'a, EdgeWeightKindDiscriminants, Outgoing<'a>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.all()
    }
}

impl<'a, T: GraphyCategoryElement<'a>> GraphyNode<'a> for Category<'a, T> {
    type Id = <AnyCategory<'a> as GraphyNode<'a>>::Id;
    type Weight = <AnyCategory<'a> as GraphyNode<'a>>::Weight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(AnyCategory::as_node(node), PhantomData)
    }
    fn try_as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> GraphyResult<Self> {
        let node = AnyCategory::try_as_node(node)?;
        let kind = node.category_kind()?;
        if kind == T::category_kind() {
            Ok(Category(node, PhantomData))
        } else {
            Err(GraphyError::WrongCategory(kind))
        }
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        AnyCategory::weight_as(weight)
    }
}

impl<'a, T: GraphyCategoryElement<'a>> From<Category<'a, T>> for GraphyNodeRef<'a> {
    fn from(category: Category<'a, T>) -> Self {
        category.0.into()
    }
}

impl<'a, T: GraphyCategoryElement<'a>> AsRef<GraphyNodeRef<'a>> for Category<'a, T> {
    fn as_ref(&self) -> &GraphyNodeRef<'a> {
        self.0.as_ref()
    }
}
