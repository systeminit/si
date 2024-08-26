use super::{
    Category, FuncArgument, GraphyCategoryElement, GraphyItertools as _, GraphyError,
    GraphyNode, GraphyNodeRef, GraphyResult, Module,
};
use crate::{
    workspace_snapshot::node_weight::{
        category_node_weight::CategoryNodeKind, FuncNodeWeight, NodeWeight,
    },
    EdgeWeightKindDiscriminants, FuncId,
};

///
/// Function definition.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Func<'a>(GraphyNodeRef<'a>);

impl<'a> Func<'a> {
    /// Argument definitions for this function.
    pub fn arguments(self) -> impl Iterator<Item = FuncArgument<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use)
    }

    //
    // Backreferences
    //
    pub fn category(self) -> GraphyResult<Category<'a, Func<'a>>> {
        self.incoming_edges()
            .of_kind(EdgeWeightKindDiscriminants::Use)
            .of_type()
            .single()
    }
    // TODO is this an alternate parent or can it be in both here and the category?
    pub fn module(self) -> GraphyResult<Option<Module<'a>>> {
        self.incoming_edges()
            .of_kind(EdgeWeightKindDiscriminants::Use)
            .of_type()
            .optional()
    }
    // TODO used in AttributePrototype or ActionPrototype or in Func (as authentication function)
    // pub fn used_in_prototypes(self) -> impl Iterator<Item = AttributePrototype<'a>> {
    //     self.source_nodes(Use).map(AttributePrototype::from_node)
    // }
}

impl<'a> GraphyNode<'a> for Func<'a> {
    type Id = FuncId;
    type Weight = FuncNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Func(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryElement<'a> for Func<'a> {
    fn category_kind() -> CategoryNodeKind {
        CategoryNodeKind::Func
    }
}
