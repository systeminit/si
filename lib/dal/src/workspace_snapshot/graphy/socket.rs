use si_events::ulid::Ulid;

use super::{
    impl_graphy_node, AnyAttributePrototype, GraphyContentNode, GraphyError, GraphyItertools as _, GraphyNode, GraphyNodeRef, GraphyResult, InputSocketAttributePrototype, OutputSocketAttributePrototype, SchemaVariant
};
use crate::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{ContentNodeWeight, NodeWeight},
    },
    EdgeWeightKindDiscriminants, InputSocketId, OutputSocketId,
};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct AnySocket<'a>(GraphyNodeRef<'a>);

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct InputSocket<'a>(AnySocket<'a>);

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct OutputSocket<'a>(AnySocket<'a>);

impl<'a> AnySocket<'a> {
    //
    // Properties
    //
    pub fn prototype_node(self) -> GraphyResult<AnyAttributePrototype<'a>> {
        self.0.targets(EdgeWeightKindDiscriminants::Prototype)
            .single()
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<SchemaVariant<'a>> {
        self.0.sources(EdgeWeightKindDiscriminants::Use).single()
    }
}

impl<'a> InputSocket<'a> {
    pub fn prototype_node(self) -> GraphyResult<InputSocketAttributePrototype<'a>> {
        self.0.prototype_node()?.try_into()
    }
}

impl<'a> OutputSocket<'a> {
    pub fn prototype_node(self) -> GraphyResult<OutputSocketAttributePrototype<'a>> {
        self.0.prototype_node()?.try_into()
    }
}

impl_graphy_node! {
    impl From    <InputSocket>   for GraphyNodeRef;
    impl From    <OutputSocket>  for GraphyNodeRef;
    impl AsRef   <GraphyNodeRef> for InputSocket;
    impl AsRef   <GraphyNodeRef> for OutputSocket;
    impl TryFrom <GraphyNodeRef> for AnySocket;
    impl TryFrom <AnySocket>     for InputSocket;
    impl TryFrom <AnySocket>     for OutputSocket;
    impl TryFrom <GraphyNodeRef> for OutputSocket through AnySocket;
}

impl<'a> GraphyNode<'a> for AnySocket<'a> {
    type Id = Ulid;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Content(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyNode<'a> for InputSocket<'a> {
    type Id = InputSocketId;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(AnySocket::as_node(node))
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNode<'a> for InputSocket<'a> {
    fn content_kind() -> ContentAddressDiscriminants {
        ContentAddressDiscriminants::InputSocket
    }
}

impl<'a> GraphyNode<'a> for OutputSocket<'a> {
    type Id = OutputSocketId;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(AnySocket::as_node(node.into()))
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNode<'a> for OutputSocket<'a> {
    fn content_kind() -> ContentAddressDiscriminants {
        ContentAddressDiscriminants::OutputSocket
    }
}
