use std::{marker::PhantomData, ops::Deref};

use si_events::ulid::Ulid;
use crate::EdgeWeightKindDiscriminants;
use super::*;
use super::super::node_weight::{category_node_weight::CategoryNodeKind, CategoryNodeWeight, NodeWeight};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct AnyCategory<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for AnyCategory<'a> {
    type Id = Ulid;
    type Weight = CategoryNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Category }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Category(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }    
}

impl<'a> AnyCategory<'a> {
    pub fn category_kind(self) -> GraphyResult<CategoryNodeKind> {
        Ok(self.weight()?.kind())
    }

    //
    // Children
    //
    pub fn all(self) -> Targets<'a, EdgesOfKind<'a>> {
        self.0.target_nodes(EdgeWeightKindDiscriminants::Use)
    }

    //
    // Backreferences
    //
    pub fn root(self) -> GraphyResult<Root<'a>> {
        self.0.source_node(EdgeWeightKindDiscriminants::Use).map(Root)
    }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Category<'a, T: GraphyCategoryNodeType<'a>>(AnyCategory<'a>, PhantomData<T>);

impl<'a, T: GraphyCategoryNodeType<'a>> GraphyNodeType<'a> for Category<'a, T> {
    type Id = <AnyCategory<'a> as GraphyNodeType<'a>>::Id;
    type Weight = <AnyCategory<'a> as GraphyNodeType<'a>>::Weight;
    fn node_kind() -> NodeWeightDiscriminants { <AnyCategory<'a> as GraphyNodeType<'a>>::node_kind() }
    fn construct(node: GraphyNode<'a>) -> Self { Self(AnyCategory(node), PhantomData::default()) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> { AnyCategory::<'a>::weight_as(weight) }
}

impl<'a, T: GraphyCategoryNodeType<'a>> Category<'a, T> {
    //
    // Children
    //
    pub fn all(self) -> ConstructCategoryType<'a, T, Targets<'a, EdgesOfKind<'a>>> {
        ConstructCategoryType { nodes: self.0.all(), _phantom: PhantomData::default() }
    }
}

impl<'a, T: GraphyCategoryNodeType<'a>> IntoIterator for Category<'a, T> {
    type Item = T;
    type IntoIter = ConstructCategoryType<'a, T, Targets<'a, EdgesOfKind<'a>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.all()
    }
}

impl<'a, T: GraphyCategoryNodeType<'a>> From<Category<'a, T>> for GraphyNode<'a> {
    fn from(category: Category<'a, T>) -> Self {
        category.0.into()
    }
}

impl<'a, T: GraphyCategoryNodeType<'a>> AsRef<GraphyNode<'a>> for Category<'a, T> {
    fn as_ref(&self) -> &GraphyNode<'a> {
        self.0.as_ref()
    }
}

impl<'a, T: GraphyCategoryNodeType<'a>> Deref for Category<'a, T> {
    type Target = AnyCategory<'a>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: GraphyCategoryNodeType<'a>> TryFrom<GraphyNode<'a>> for Category<'a, T> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(AnyCategory(node), PhantomData::default());
        result.weight()?;
        Ok(result)
    }
}

impl<'a, T: GraphyCategoryNodeType<'a>> TryFrom<AnyCategory<'a>> for Category<'a, T> {
    type Error = GraphyError;

    fn try_from(value: AnyCategory<'a>) -> Result<Self, Self::Error> {
        let kind = value.category_kind()?;
        if kind == T::category_kind() {
            Ok(Category(value, PhantomData::default()))
        } else {
            Err(GraphyError::WrongCategory(kind))
        }
    }
}

pub struct ConstructCategoryType<'a, T: GraphyCategoryNodeType<'a>, I: Iterator<Item = GraphyNode<'a>>> {
    nodes: I,
    _phantom: PhantomData<T>,
}

impl<'a, T: GraphyCategoryNodeType<'a>, I: Iterator<Item = GraphyNode<'a>>> Iterator for ConstructCategoryType<'a, T, I> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.next().map(|node| T::construct(node))
    }
}

