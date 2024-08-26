use super::{
    impl_inherited_graphy_node, AnySocket, AnyAttributePrototype, Component, GraphyError, GraphyNode,
    GraphyNodeRef, GraphyResult, Ordering, GraphyItertools as _, Prop,
};
use crate::{
    workspace_snapshot::node_weight::{AttributeValueNodeWeight, NodeWeight},
    AttributeValueId, EdgeWeightKind, EdgeWeightKindDiscriminants,
};

///
/// Value of a prop, input socket, or output socket.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct AttributeValue<'a>(GraphyNodeRef<'a>);

impl<'a> GraphyNode<'a> for AttributeValue<'a> {
    type Id = AttributeValueId;
    type Weight = AttributeValueNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::AttributeValue(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> AttributeValue<'a> {
    /// Component-specific value
    ///
    /// If not specified, the default value from the corresponding Prop /
    /// InputSocket / OutputSocket on the SchemaVariant is used.
    ///
    /// For an InputSocket, component-specific values are always links, and the
    /// matching value can be found in the PrototypeArguments for the
    /// corresponding InputSocket.
    pub fn value(self) -> GraphyResult<Option<AnyAttributePrototype<'a>>> {
        self.targets(EdgeWeightKindDiscriminants::Prototype)
            .optional()
    }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct PropAttributeValue<'a>(AttributeValue<'a>);

impl<'a> PropAttributeValue<'a> {
    /// Child values in order
    pub fn children(
        self,
    ) -> GraphyResult<impl Iterator<Item = GraphyResult<ChildPropAttributeValue<'a>>>> {
        let optional_children = match self.ordering_node()? {
            Some(ordering) => Some(ordering.children()?),
            // If there are Contains children, an ordering is presently required
            None => match self.unordered_children().next() {
                Some(_) => return Err(GraphyError::MissingOrdering(self.index)),
                None => None,
            },
        };
        Ok(optional_children.into_iter().flatten())
    }
    /// Child values
    pub fn unordered_children(self) -> impl Iterator<Item = ChildPropAttributeValue<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Contain)
    }
    /// Order of child values
    pub fn ordering_node(
        self,
    ) -> GraphyResult<Option<Ordering<'a, ChildPropAttributeValue<'a>, Self>>> {
        self.targets(EdgeWeightKind::Ordering).optional()
    }

    /// Prop this is a value for
    pub fn prop(self) -> GraphyResult<Prop<'a>> {
        self.targets(EdgeWeightKind::Prop).single()
    }

    //
    // Backreferences
    //
    pub fn parent_component(self) -> GraphyResult<Option<Component<'a>>> {
        self.sources(EdgeWeightKind::Root).optional()
    }
    pub fn parent_value(self) -> GraphyResult<Option<PropAttributeValue<'a>>> {
        self.sources(EdgeWeightKindDiscriminants::Contain)
            .optional()
    }
    pub fn parent_ordering(self) -> GraphyResult<Ordering<'a, Self, PropAttributeValue<'a>>> {
        self.sources(EdgeWeightKind::Ordering).single()
    }
}

impl_inherited_graphy_node! {
    impl * for PropAttributeValue { AttributeValue }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct RootPropAttributeValue<'a>(PropAttributeValue<'a>);

impl<'a> RootPropAttributeValue<'a> {
    //
    // Children
    //
    pub fn ordering(self) -> GraphyResult<Option<Ordering<'a, ChildPropAttributeValue<'a>, Self>>> {
        self.targets(EdgeWeightKind::Ordering).optional()
    }

    //
    // Backreferences
    //
    pub fn parent_component(self) -> GraphyResult<Component<'a>> {
        self.sources(EdgeWeightKind::Root).single()
    }
    pub fn parent_ordering(self) -> GraphyResult<Ordering<'a, Self, PropAttributeValue<'a>>> {
        self.sources(EdgeWeightKind::Ordering).single()
    }
}

impl_inherited_graphy_node! {
    impl * for RootPropAttributeValue { PropAttributeValue }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct ChildPropAttributeValue<'a>(PropAttributeValue<'a>);

impl<'a> ChildPropAttributeValue<'a> {
    //
    // Children
    //
    pub fn ordering(self) -> GraphyResult<Option<Ordering<'a, ChildPropAttributeValue<'a>, Self>>> {
        self.targets(EdgeWeightKind::Ordering).optional()
    }

    //
    // Backreferences
    //
    pub fn parent_value(self) -> GraphyResult<PropAttributeValue<'a>> {
        self.sources(EdgeWeightKindDiscriminants::Contain).single()
    }
    pub fn parent_ordering(self) -> GraphyResult<Ordering<'a, Self, PropAttributeValue<'a>>> {
        self.sources(EdgeWeightKind::Ordering).single()
    }
}

impl_inherited_graphy_node! {
    impl * for ChildPropAttributeValue { PropAttributeValue }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct SocketAttributeValue<'a>(AttributeValue<'a>);

impl<'a> SocketAttributeValue<'a> {
    //
    // Properties
    //
    pub fn socket(self) -> GraphyResult<AnySocket<'a>> {
        self.targets(EdgeWeightKind::Socket).single()
    }

    //
    // Backreferences
    //
    pub fn parent_component(self) -> GraphyResult<Component<'a>> {
        self.sources(EdgeWeightKind::SocketValue).single()
    }
}

impl_inherited_graphy_node! {
    impl * for SocketAttributeValue { AttributeValue }
}
