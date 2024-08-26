use super::{
    Category, GraphyCategoryElement, GraphyError, GraphyNode, GraphyNodeId, GraphyNodeRef, GraphyResult, GraphyItertools as _, RootPropAttributeValue, SchemaVariant, SocketAttributeValue
};
use crate::{
    workspace_snapshot::node_weight::{
        category_node_weight::CategoryNodeKind, ComponentNodeWeight, NodeWeight,
    },
    ComponentId, EdgeWeightKind, EdgeWeightKindDiscriminants,
};

impl GraphyNodeId for ComponentId {
    type Node<'a> = Component<'a>;
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Component<'a>(GraphyNodeRef<'a>);

impl<'a> Component<'a> {
    /// Defining SchemaVariant.
    ///
    /// All Props and InputSockets will match this, and defaults are grabbed from it as well.
    pub fn schema_variant(self) -> GraphyResult<SchemaVariant<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use).single()
    }
    /// Root prop value
    ///
    /// Prop value tree mirrors SchemaVariant.Prop tree 1:N (some props are
    /// array or map element props, and those can have multiple children).
    ///
    /// All props have corresponding attribute values.
    pub fn root_prop_value(self) -> GraphyResult<RootPropAttributeValue<'a>> {
        self.targets(EdgeWeightKind::Root).single()
    }
    /// Socket values
    ///
    /// Socket values are always single values and mirror SchemaVariant.Sockets 1:1
    pub fn socket_values(self) -> impl Iterator<Item = SocketAttributeValue<'a>> {
        self.targets(EdgeWeightKind::SocketValue)
    }
    /// Children inside this component (if this component is a frame)
    pub fn frame_children(self) -> impl Iterator<Item = Component<'a>> {
        self.targets(EdgeWeightKind::FrameContains)
    }

    //
    // Backreferences
    //
    pub fn category(self) -> GraphyResult<Category<'a, Self>> {
        self.sources(EdgeWeightKindDiscriminants::Use).single()
    }
    pub fn parent_frame(self) -> GraphyResult<Option<Component<'a>>> {
        self.sources(EdgeWeightKind::FrameContains).optional()
    }
}

impl<'a> GraphyNode<'a> for Component<'a> {
    type Id = ComponentId;
    type Weight = ComponentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Component(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryElement<'a> for Component<'a> {
    fn category_kind() -> CategoryNodeKind {
        CategoryNodeKind::Component
    }
}
