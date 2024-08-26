use super::{
    ActionPrototype, AnySocket, Component, Func, GraphyContentNode, GraphyNode, GraphyNodeRef,
    GraphyResult, InputSocket, Module, GraphyItertools as _, OutputSocket, Prop,
    ResultItertools as _, Schema,
};
use crate::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{ContentNodeWeight, NodeWeight},
    },
    EdgeWeightKind, EdgeWeightKindDiscriminants, SchemaVariantId,
};

///
/// Specific versioned definition of a component type.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct SchemaVariant<'a>(GraphyNodeRef<'a>);

impl<'a> SchemaVariant<'a> {
    /// Root prop
    pub fn root_prop(self) -> GraphyResult<Prop<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use).single()
    }
    /// Input sockets
    pub fn input_sockets(self) -> impl Iterator<Item = InputSocket<'a>> {
        self.sockets().try_as_nodes().skip_err()
    }
    /// Output sockets
    pub fn output_sockets(self) -> impl Iterator<Item = OutputSocket<'a>> {
        self.sockets().try_as_nodes().skip_err()
    }
    /// All sockets
    pub fn sockets(self) -> impl Iterator<Item = AnySocket<'a>> {
        self.targets(EdgeWeightKind::Socket)
    }
    /// Actions that can be run on components of this type
    pub fn actions(self) -> impl Iterator<Item = ActionPrototype<'a>> {
        self.targets(EdgeWeightKind::ActionPrototype)
    }
    /// Authentication functions (TODO not 100% sure what these are, need to dig)
    pub fn authentication_functions(self) -> impl Iterator<Item = Func<'a>> {
        self.targets(EdgeWeightKind::AuthenticationPrototype)
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<Schema<'a>> {
        self.sources(EdgeWeightKindDiscriminants::Use).single()
    }
    pub fn components(self) -> impl Iterator<Item = Component<'a>> {
        self.sources(EdgeWeightKindDiscriminants::Use)
    }
    // TODO is this an alternate parent or can it be in both here and the category?
    pub fn module(self) -> GraphyResult<Option<Module<'a>>> {
        self.sources(EdgeWeightKindDiscriminants::Use).optional()
    }
}

impl<'a> GraphyNode<'a> for SchemaVariant<'a> {
    type Id = SchemaVariantId;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNode<'a> for SchemaVariant<'a> {
    fn content_kind() -> ContentAddressDiscriminants {
        ContentAddressDiscriminants::SchemaVariant
    }
}
