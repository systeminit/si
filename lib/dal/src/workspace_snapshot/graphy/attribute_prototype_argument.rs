use si_events::ulid::Ulid;
use std::ops::Deref;

use super::{
    impl_inherited_graphy_node, AnyAttributePrototype, Component, FuncArgument, GraphyError, GraphyNode, GraphyNodeRef, GraphyResult, InputSocket, InputSocketAttributePrototype, GraphyItertools as _, OutputSocket, Prop, Secret, StaticArgumentValue
};
use crate::{
    attribute::prototype::argument::AttributePrototypeArgumentId, workspace_snapshot::node_weight::{
        ArgumentTargets, AttributePrototypeArgumentNodeWeight, NodeWeight,
    }, ComponentId, EdgeWeightKind, EdgeWeightKindDiscriminants
};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct AnyAttributePrototypeArgument<'a>(GraphyNodeRef<'a>);

impl<'a> AnyAttributePrototypeArgument<'a> {
    /// Function argument definition
    pub fn function_argument(self) -> GraphyResult<FuncArgument<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use).single()
    }

    /// Value for this argument
    pub fn value(self) -> impl Iterator<Item = AttributePrototypeArgumentValue<'a>> {
        self.targets(EdgeWeightKind::PrototypeArgumentValue)
    }

    /// Argument targets to match input socket and show output socket
    pub fn argument_targets(self) -> GraphyResult<Option<ArgumentTargets>> {
        Ok(self.weight()?.targets())
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<AnyAttributePrototype<'a>> {
        self.sources(EdgeWeightKind::PrototypeArgument).single()
    }
}

impl<'a> GraphyNode<'a> for AnyAttributePrototypeArgument<'a> {
    type Id = AttributePrototypeArgumentId;
    type Weight = AttributePrototypeArgumentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::AttributePrototypeArgument(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct ValueAttributePrototypeArgument<'a>(AnyAttributePrototypeArgument<'a>);

impl<'a> ValueAttributePrototypeArgument<'a> {
    pub fn value(self) -> GraphyResult<AttributePrototypeArgumentValue<'a>> {
        self.targets(EdgeWeightKind::PrototypeArgumentValue).single()
    }
    pub fn argument_targets(self) -> GraphyResult<()> {
        match self.0.argument_targets()? {
            Some(_) => Err(GraphyError::UnexpectedArgumentTargets),
            None => Ok(())
        }
    }
}

impl_inherited_graphy_node! {
    impl * for ValueAttributePrototypeArgument { AnyAttributePrototypeArgument }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct LinkAttributePrototypeArgument<'a>(AnyAttributePrototypeArgument<'a>);

impl<'a> LinkAttributePrototypeArgument<'a> {
    pub fn source_component(self) -> GraphyResult<Component<'a>> {
        Ok(Component::as_node(self.0.graph.node_ref_by_id(self.source_component_id()?)?))
    }
    pub fn source_output_socket(self) -> GraphyResult<OutputSocket<'a>> {
        self.0.targets(EdgeWeightKind::PrototypeArgumentValue).single()
    }
    pub fn component(self) -> GraphyResult<Component<'a>> {
        Ok(Component::as_node(self.0.graph.node_ref_by_id(self.component_id()?)?))
    }

    pub fn argument_targets(self) -> GraphyResult<ArgumentTargets> {
        self.0.argument_targets()?.ok_or(GraphyError::NoArgumentTargets)
    }
    pub fn source_component_id(self) -> GraphyResult<ComponentId> {
        Ok(self.argument_targets()?.source_component_id)
    }
    pub fn component_id(self) -> GraphyResult<ComponentId> {
        Ok(self.argument_targets()?.destination_component_id)
    }

    pub fn parent(self) -> GraphyResult<InputSocketAttributePrototype<'a>> {
        self.0.sources(EdgeWeightKind::PrototypeArgument).single()
    }
}

impl_inherited_graphy_node! {
    impl * for LinkAttributePrototypeArgument { AnyAttributePrototypeArgument }
}

///
/// An attribute prototype can be for a schema variant's prop or socket; or for an attribute value
/// of a component. It can only have *one* of these.
///
#[derive(Copy, Clone)]
pub enum AttributePrototypeArgumentValue<'a> {
    Prop(Prop<'a>),
    InputSocket(InputSocket<'a>),
    OutputSocket(OutputSocket<'a>),
    Secret(Secret<'a>),
    StaticArgumentValue(StaticArgumentValue<'a>),
}

impl<'a> GraphyNode<'a> for AttributePrototypeArgumentValue<'a> {
    type Id = Ulid;
    type Weight = NodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        GraphyNode::try_as_node(node)
            .map(AttributePrototypeArgumentValue::StaticArgumentValue)
            .or_else(|_| GraphyNode::try_as_node(node).map(AttributePrototypeArgumentValue::Prop))
            .or_else(|_| {
                GraphyNode::try_as_node(node).map(AttributePrototypeArgumentValue::OutputSocket)
            })
            .or_else(|_| GraphyNode::try_as_node(node).map(AttributePrototypeArgumentValue::Secret))
            .unwrap_or_else(|_| {
                AttributePrototypeArgumentValue::InputSocket(GraphyNode::as_node(node))
            })
    }
    fn try_as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> GraphyResult<Self> {
        GraphyNode::try_as_node(node)
            .map(AttributePrototypeArgumentValue::StaticArgumentValue)
            .or_else(|_| GraphyNode::try_as_node(node).map(AttributePrototypeArgumentValue::Prop))
            .or_else(|_| {
                GraphyNode::try_as_node(node).map(AttributePrototypeArgumentValue::OutputSocket)
            })
            .or_else(|_| GraphyNode::try_as_node(node).map(AttributePrototypeArgumentValue::Secret))
            .or_else(|_| {
                GraphyNode::try_as_node(node).map(AttributePrototypeArgumentValue::InputSocket)
            })
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Ok(weight)
    }
}

impl<'a> From<AttributePrototypeArgumentValue<'a>> for GraphyNodeRef<'a> {
    fn from(node: AttributePrototypeArgumentValue<'a>) -> Self {
        match node {
            AttributePrototypeArgumentValue::Prop(node) => node.into(),
            AttributePrototypeArgumentValue::InputSocket(node) => node.into(),
            AttributePrototypeArgumentValue::OutputSocket(node) => node.into(),
            AttributePrototypeArgumentValue::Secret(node) => node.into(),
            AttributePrototypeArgumentValue::StaticArgumentValue(node) => node.into(),
        }
    }
}

impl<'a> AsRef<GraphyNodeRef<'a>> for AttributePrototypeArgumentValue<'a> {
    fn as_ref(&self) -> &GraphyNodeRef<'a> {
        match self {
            AttributePrototypeArgumentValue::Prop(node) => node.as_ref(),
            AttributePrototypeArgumentValue::InputSocket(node) => node.as_ref(),
            AttributePrototypeArgumentValue::OutputSocket(node) => node.as_ref(),
            AttributePrototypeArgumentValue::Secret(node) => node.as_ref(),
            AttributePrototypeArgumentValue::StaticArgumentValue(node) => node.as_ref(),
        }
    }
}

impl<'a> Deref for AttributePrototypeArgumentValue<'a> {
    type Target = GraphyNodeRef<'a>;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
