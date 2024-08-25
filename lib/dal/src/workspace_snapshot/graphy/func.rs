use crate::func::argument::FuncArgumentId;
use crate::workspace_snapshot::node_weight::FuncArgumentNodeWeight;
use crate::FuncId;
use super::*;
use super::super::node_weight::{category_node_weight::CategoryNodeKind, FuncNodeWeight, NodeWeight};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Func<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for Func<'a> {
    type Id = FuncId;
    type Weight = FuncNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Func }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Func(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryNodeType<'a> for Func<'a> {
    fn category_kind() -> CategoryNodeKind { CategoryNodeKind::Func }
}

impl<'a> Func<'a> {
    //
    // Children
    //
    pub fn arguments(self) -> impl Iterator<Item = FuncArgument<'a>> {
        self.0.target_nodes(EdgeWeightKindDiscriminants::Use).map(FuncArgument::construct)
    }

    //
    // Backreferences
    //
    pub fn category(self) -> GraphyResult<Category<'a, Func<'a>>> {
        self.0.matching_source(EdgeWeightKindDiscriminants::Use)
    }
    // TODO is this an alternate parent or can it be in both here and the category?
    pub fn module(self) -> GraphyResult<Option<Module<'a>>> {
        self.0.matching_source_opt(EdgeWeightKindDiscriminants::Use)
    }
    // TODO used in AttributePrototype or ActionPrototype or in Func (as authentication function)
    // pub fn used_in_prototypes(self) -> impl Iterator<Item = AttributePrototype<'a>> {
    //     self.0.source_nodes(EdgeWeightKindDiscriminants::Use).map(AttributePrototype::construct)
    // }
}

impl<'a> TryFrom<GraphyNode<'a>> for Func<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct FuncArgument<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for FuncArgument<'a> {
    type Id = FuncArgumentId;
    type Weight = FuncArgumentNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::FuncArgument }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::FuncArgument(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> FuncArgument<'a> {
    pub fn id(self) -> GraphyResult<FuncArgumentId> {
        self.0.id().map(Into::into)
    }
    pub fn weight(self) -> GraphyResult<&'a FuncArgumentNodeWeight> {
        match self.0.node_weight()? {
            NodeWeight::FuncArgument(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> TryFrom<GraphyNode<'a>> for FuncArgument<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}
