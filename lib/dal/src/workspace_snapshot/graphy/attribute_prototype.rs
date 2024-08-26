use si_events::ulid::Ulid;
use std::ops::Deref;

use super::{
    impl_graphy_node, impl_inherited_graphy_node, AnyAttributePrototypeArgument, AttributeValue, Func, GraphyContentNode, GraphyItertools as _, GraphyNode, GraphyNodeRef, GraphyResult, InputSocket, LinkAttributePrototypeArgument, OutputSocket, Prop, ValueAttributePrototypeArgument
};
use crate::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{ContentNodeWeight, NodeWeight},
    },
    AttributePrototypeId, EdgeWeightKind, EdgeWeightKindDiscriminants,
};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct AnyAttributePrototype<'a>(GraphyNodeRef<'a>);

impl<'a> AnyAttributePrototype<'a> {
    //
    // Properties
    //
    pub fn function(self) -> GraphyResult<Func<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use).single()
    }

    //
    // Children
    //
    pub fn prototype_argument_nodes(self) -> impl Iterator<Item = AnyAttributePrototypeArgument<'a>> {
        self.targets(EdgeWeightKind::PrototypeArgument)
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<Option<AttributePrototypeParent<'a>>> {
        self.sources(EdgeWeightKindDiscriminants::Use).optional()
    }
    pub fn referencing_values(self) -> impl Iterator<Item = AttributeValue<'a>> {
        self.sources(EdgeWeightKindDiscriminants::Prototype)
    }
}

impl<'a> GraphyNode<'a> for AnyAttributePrototype<'a> {
    type Id = AttributePrototypeId;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNode<'a> for AnyAttributePrototype<'a> {
    fn content_kind() -> ContentAddressDiscriminants {
        ContentAddressDiscriminants::AttributePrototype
    }
}

///
/// An attribute prototype can be for a schema variant's prop or socket; or for an attribute value
/// of a component. It can only have *one* of these.
///
#[derive(Copy, Clone)]
pub enum AttributePrototypeParent<'a> {
    Prop(Prop<'a>),
    InputSocket(InputSocket<'a>),
    OutputSocket(OutputSocket<'a>),
    AttributeValue(AttributeValue<'a>),
}

impl<'a> GraphyNode<'a> for AttributePrototypeParent<'a> {
    type Id = Ulid;
    type Weight = NodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        GraphyNode::try_as_node(node)
            .map(AttributePrototypeParent::Prop)
            .or_else(|_| GraphyNode::try_as_node(node).map(AttributePrototypeParent::InputSocket))
            .or_else(|_| GraphyNode::try_as_node(node).map(AttributePrototypeParent::OutputSocket))
            .unwrap_or_else(|_| AttributePrototypeParent::AttributeValue(GraphyNode::as_node(node)))
    }
    fn try_as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> GraphyResult<Self> {
        GraphyNode::try_as_node(node)
            .map(AttributePrototypeParent::Prop)
            .or_else(|_| GraphyNode::try_as_node(node).map(AttributePrototypeParent::InputSocket))
            .or_else(|_| GraphyNode::try_as_node(node).map(AttributePrototypeParent::OutputSocket))
            .or_else(|_| {
                GraphyNode::try_as_node(node).map(AttributePrototypeParent::AttributeValue)
            })
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Ok(weight)
    }
}

impl<'a> From<AttributePrototypeParent<'a>> for GraphyNodeRef<'a> {
    fn from(node: AttributePrototypeParent<'a>) -> Self {
        match node {
            AttributePrototypeParent::Prop(node) => node.into(),
            AttributePrototypeParent::InputSocket(node) => node.into(),
            AttributePrototypeParent::OutputSocket(node) => node.into(),
            AttributePrototypeParent::AttributeValue(node) => node.into(),
        }
    }
}

impl<'a> AsRef<GraphyNodeRef<'a>> for AttributePrototypeParent<'a> {
    fn as_ref(&self) -> &GraphyNodeRef<'a> {
        match self {
            AttributePrototypeParent::Prop(node) => node.as_ref(),
            AttributePrototypeParent::InputSocket(node) => node.as_ref(),
            AttributePrototypeParent::OutputSocket(node) => node.as_ref(),
            AttributePrototypeParent::AttributeValue(node) => node.as_ref(),
        }
    }
}

impl<'a> Deref for AttributePrototypeParent<'a> {
    type Target = GraphyNodeRef<'a>;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct PropAttributePrototype<'a>(AnyAttributePrototype<'a>);

impl<'a> PropAttributePrototype<'a> {
    /// Arguments to the function.
    pub fn argument_values(self) -> impl Iterator<Item = ValueAttributePrototypeArgument<'a>> {
        self.targets(EdgeWeightKind::PrototypeArgument)
    }
}

impl_inherited_graphy_node! {
    impl * for PropAttributePrototype { AnyAttributePrototype }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct InputSocketAttributePrototype<'a>(AnyAttributePrototype<'a>);

impl<'a> InputSocketAttributePrototype<'a> {
    /// Argument representing the default value for the socket
    pub fn default_value_argument(self) -> GraphyResult<ValueAttributePrototypeArgument<'a>> {
        self.prototype_argument_nodes().of_type().single()
    }
    /// Arguments representing links to other sockets (for individual components)
    pub fn link_arguments(self) -> impl Iterator<Item = LinkAttributePrototypeArgument<'a>> {
        self.prototype_argument_nodes().of_type()
    }
}

impl_inherited_graphy_node! {
    impl * for InputSocketAttributePrototype { AnyAttributePrototype }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct OutputSocketAttributePrototype<'a>(AnyAttributePrototype<'a>);

impl<'a> OutputSocketAttributePrototype<'a> {
    /// Argument representing the default value for the socket
    pub fn default_value_argument(self) -> GraphyResult<ValueAttributePrototypeArgument<'a>> {
        self.prototype_argument_nodes().of_type().single()
    }
}

impl_inherited_graphy_node! {
    impl * for OutputSocketAttributePrototype { AnyAttributePrototype }
}

impl_graphy_node! {
    impl TryFrom<GraphyNodeRef> for AnyAttributePrototype;
    impl TryFrom<AnyAttributePrototype> for PropAttributePrototype;
    impl TryFrom<AnyAttributePrototype> for InputSocketAttributePrototype;
    impl TryFrom<AnyAttributePrototype> for OutputSocketAttributePrototype;
    impl TryFrom<GraphyNodeRef> for PropAttributePrototype         through AnyAttributePrototype;
    impl TryFrom<GraphyNodeRef> for InputSocketAttributePrototype  through AnyAttributePrototype;
    impl TryFrom<GraphyNodeRef> for OutputSocketAttributePrototype through AnyAttributePrototype;
}
