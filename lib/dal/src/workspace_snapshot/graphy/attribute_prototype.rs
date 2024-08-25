use crate::attribute::prototype::argument::AttributePrototypeArgumentId;
use crate::workspace_snapshot::node_weight::{AttributePrototypeArgumentNodeWeight, NodeWeight};
use crate::{AttributePrototypeId, EdgeWeightKindDiscriminants};
use super::*;
use super::super::{content_address::ContentAddressDiscriminants, node_weight::ContentNodeWeight};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct AttributePrototype<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for AttributePrototype<'a> {
    type Id = AttributePrototypeId;
    type Weight = ContentNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Content }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNodeType<'a> for AttributePrototype<'a> {
    fn content_kind() -> ContentAddressDiscriminants { ContentAddressDiscriminants::AttributePrototype }
}

impl<'a> AttributePrototype<'a> {
    //
    // Properties
    //
    pub fn func(self) -> GraphyResult<Func<'a>> {
        self.0.target_node(EdgeWeightKindDiscriminants::Use).map(Func)
    }

    //
    // Children
    //
    pub fn arguments(self) -> impl Iterator<Item = AttributePrototypeArgument<'a>> {
        self.0.target_nodes(EdgeWeightKindDiscriminants::PrototypeArgument).map(AttributePrototypeArgument::construct)
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<AttributePrototypeParent<'a>> {
        self.0.source_node(EdgeWeightKindDiscriminants::Use).and_then(TryFrom::try_from)
    }
}

impl<'a> TryFrom<GraphyNode<'a>> for AttributePrototype<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct AttributePrototypeArgument<'a>(pub(super) GraphyNode<'a>);

pub enum AttributePrototypeArgumentValue<'a> {
    Prop(Prop<'a>),
    // Socket(Socket<'a>),
    Secret(Secret<'a>),
    // StaticArgumentValue(StaticArgumentValue<'a>),
}

impl<'a> GraphyNodeType<'a> for AttributePrototypeArgument<'a> {
    type Id = AttributePrototypeArgumentId;
    type Weight = AttributePrototypeArgumentNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::AttributePrototypeArgument }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::AttributePrototypeArgument(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> AttributePrototypeArgument<'a> {
    //
    // Properties
    //
    pub fn func_argument(self) -> GraphyResult<FuncArgument<'a>> {
        self.0.target_node(EdgeWeightKindDiscriminants::Use).map(FuncArgument::construct)
    }

    //
    // Children
    //
    // pub fn values(self) -> GraphyResult<impl Iterator<Item = AttributePrototypeArgumentValue<'a>>> {
    //     self.0.target_nodes(EdgeWeightKindDiscriminants::PrototypeArgumentValue).map(AttributePrototypeArgumentValue)
    // }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<AttributePrototype<'a>> {
        self.0.source_node(EdgeWeightKindDiscriminants::PrototypeArgument).map(AttributePrototype)
    }
}

impl<'a> TryFrom<GraphyNode<'a>> for AttributePrototypeArgument<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}

///
/// An attribute prototype can be for a schema variant's prop or socket; or for an attribute value
/// of a component. It can only have *one* of these.
/// 
#[derive(Copy, Clone)]
pub enum AttributePrototypeParent<'a> {
    Prop(Prop<'a>),
    // Socket(Socket<'a>)
    // AttributeValue(AttributeValue<'a>),
}

impl<'a> From<AttributePrototypeParent<'a>> for GraphyNode<'a> {
    fn from(parent: AttributePrototypeParent<'a>) -> Self {
        match parent {
            AttributePrototypeParent::Prop(prop) => prop.into(),
            // AttributePrototypeParent::Socket(socket) => socket.into(),
            // AttributePrototypeParent::AttributeValue(attribute_value) => attribute_value.into(),
        }
    }
}

impl<'a> AsRef<GraphyNode<'a>> for AttributePrototypeParent<'a> {
    fn as_ref(&self) -> &GraphyNode<'a> {
        match self {
            AttributePrototypeParent::Prop(prop) => prop.as_ref(),
            // AttributePrototypeParent::Socket(socket) => socket.as_ref(),
            // AttributePrototypeParent::AttributeValue(attribute_value) => attribute_value.as_ref(),
        }
    }
}

impl<'a> TryFrom<GraphyNode<'a>> for AttributePrototypeParent<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        Prop::try_from(node).map(Self::Prop)
        // AttributeValue::try_from(node).map(Self::AttributeValue)
        //     .or_else(|_| Prop::try_from(node).map(Self::Prop))
        //     .or_else(|_| Socket::try_from(node).map(Self::Socket))
    }
}

