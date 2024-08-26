use super::{
    AttributePrototypeArgumentValue, GraphyContentNode, GraphyNode, GraphyNodeRef, GraphyResult,
    GraphyItertools as _,
};
use crate::{
    attribute::prototype::argument::static_value::StaticArgumentValueId,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{ContentNodeWeight, NodeWeight},
    },
    EdgeWeightKind,
};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct StaticArgumentValue<'a>(GraphyNodeRef<'a>);

impl<'a> GraphyNode<'a> for StaticArgumentValue<'a> {
    type Id = StaticArgumentValueId;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNode<'a> for StaticArgumentValue<'a> {
    fn content_kind() -> ContentAddressDiscriminants {
        ContentAddressDiscriminants::StaticArgumentValue
    }
}

impl<'a> StaticArgumentValue<'a> {
    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<AttributePrototypeArgumentValue<'a>> {
        self.sources(EdgeWeightKind::PrototypeArgumentValue)
            .single()
    }
}
