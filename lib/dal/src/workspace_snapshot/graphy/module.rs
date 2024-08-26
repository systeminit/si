use super::{GraphyCategoryElement, GraphyContentNode, GraphyNode, GraphyNodeRef, GraphyResult};
use crate::workspace_snapshot::{
    content_address::ContentAddressDiscriminants,
    node_weight::{category_node_weight::CategoryNodeKind, ContentNodeWeight, NodeWeight},
};
use si_events::ulid::Ulid;

///
/// Module containing sharable functions and schema/variant definitions.
///
/// TODO not super clear on exactly how this one works.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Module<'a>(GraphyNodeRef<'a>);

impl<'a> Module<'a> {}

impl<'a> GraphyNode<'a> for Module<'a> {
    type Id = Ulid;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}
impl<'a> GraphyContentNode<'a> for Module<'a> {
    fn content_kind() -> ContentAddressDiscriminants {
        ContentAddressDiscriminants::Module
    }
}

impl<'a> GraphyCategoryElement<'a> for Module<'a> {
    fn category_kind() -> CategoryNodeKind {
        CategoryNodeKind::Module
    }
}
