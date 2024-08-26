use super::{
    Category, GraphyCategoryElement, GraphyContentNode, GraphyNode, GraphyNodeRef, GraphyResult,
    Module, GraphyItertools as _, SchemaVariant,
};
use crate::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{category_node_weight::CategoryNodeKind, ContentNodeWeight, NodeWeight},
    },
    EdgeWeightKindDiscriminants, SchemaId,
};

///
/// Definition of a component type.
///
/// Variants are versions of said schema and actually define the schema.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Schema<'a>(GraphyNodeRef<'a>);

impl<'a> Schema<'a> {
    /// Versions of this component type definition
    pub fn variants(self) -> impl Iterator<Item = SchemaVariant<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use)
    }

    //
    // Backreferences
    //
    pub fn category(self) -> GraphyResult<Category<'a, Module<'a>>> {
        self.sources(EdgeWeightKindDiscriminants::Use).single()
    }
    // TODO is this an alternate parent or can it be in both here and the category?
    pub fn module(self) -> GraphyResult<Option<Module<'a>>> {
        self.sources(EdgeWeightKindDiscriminants::Use).optional()
    }
}

impl<'a> GraphyNode<'a> for Schema<'a> {
    type Id = SchemaId;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNode<'a> for Schema<'a> {
    fn content_kind() -> ContentAddressDiscriminants {
        ContentAddressDiscriminants::Schema
    }
}

impl<'a> GraphyCategoryElement<'a> for Schema<'a> {
    fn category_kind() -> CategoryNodeKind {
        CategoryNodeKind::Schema
    }
}
