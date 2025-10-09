//! Edges

use std::hash::Hash;

use serde::{
    Deserialize,
    Serialize,
};
use strum::EnumDiscriminants;

use crate::attribute::path::AttributePath;

/// This type is postcard serialized and new enum variants *MUST* be added to the end *ONLY*.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(derive(Hash, Serialize, Deserialize, strum::EnumIter, strum::Display))]
pub enum EdgeWeightKind {
    Action,
    /// A function used by a [`SchemaVariant`] to perform an action that affects its resource
    ActionPrototype,
    /// A function defined for a secret defining [`SchemaVariant`] to be executed before funcs on
    /// components that have a secret of that kind
    AuthenticationPrototype,
    /// An [`AttributeValue`] "contained" by another [`AttributeValue`], such as an entry in an
    /// array/map, or a field of an object. The optional [`String`] represents the key of the entry
    /// in a map.
    Contain(Option<String>),
    /// Used to indicate parentage within frames. It does not dictate data flow. That is provided via
    /// [`ComponentType`](crate::ComponentType).
    ///
    /// This replaces "Symbolic" edges and "Frame" sockets from the old engine.
    /// TODO (jkeiser) remove in a new snapshot format version
    #[serde(rename = "FrameContains")]
    DeprecatedFrameContains,
    /// Used to record the order that the elements of a container should be presented in.
    Ordering,
    /// Connects the node at the Ordering edge directly to the things it orders.
    Ordinal,
    /// Used to link an attribute value to the prop that it is for.
    Prop,
    /// An edge from a [`socket`](crate::socket), [`AttributeValue`](crate::AttributeValue) or [`Prop`](crate::Prop)
    /// to an [`AttributePrototype`](crate::AttributePrototype). The optional [`String`] is used for
    /// maps, arrays and relevant container types to indicate which element the prototype is for.
    Prototype(Option<String>),
    /// An edge from an [`AttributePrototype`][crate::AttributePrototype] to an
    /// [`AttributePrototypeArgument`][crate::AttributePrototypeArgument].
    PrototypeArgument,
    /// An edge from an
    /// [`AttributePrototypeArgument`][crate::AttributePrototypeArgument] to the
    /// source for the value for this argument
    PrototypeArgumentValue,
    Proxy,
    /// Indicates the "root" [`AttributeValue`](crate::AttributeValue) for a [`Component`](crate::Component).
    ///
    /// TODO(nick): in the future, this should be used for the "root" [`Prop`](crate::Prop) for a
    /// [`SchemaVariant`](crate::SchemaVariant) as well.
    Root,
    /// Used when the target/destination of an edge is an [`InputSocket`](crate::InputSocket), or an
    /// [`OutputSocket`](crate::OutputSocket).
    Socket,
    /// Edge from component to input or output Socket's attribute value
    SocketValue,
    /// Workspaces "use" functions, modules, schemas. Schemas "use" schema variants.
    /// Schema variants "use" props. Props "use" functions, and other props. Modules
    /// "use" functions, schemas, and eventually(?) components.
    Use {
        is_default: bool,
    },
    /// Edge from attribute value to validation result node
    ValidationOutput,
    /// Edge from [`Schema`](crate::Schema) or a [`SchemaVariant`](crate::SchemaVariant) to [`ManagementPrototype`](crate::ManagementPrototype).
    ManagementPrototype,
    /// From a Geometry node to the node it represents on a view.
    Represents,
    /// From a manager [`Component`][`crate::Component`] to a managed `Component
    Manages,
    /// From a view node to a diagram object node, to which geometries can be connected.
    DiagramObject,
    /// Indicates if there is an corresponding approval requirement definition.
    ApprovalRequirementDefinition,
    /// An edge from an
    /// [`AttributePrototypeArgument`][crate::AttributePrototypeArgument] representing a value
    /// from a component, where the edge target points at the component/root av id, and the
    /// [`AttributePath`] is a path to a nested child AV (e.g.
    /// `/domain/PolicyDocument/Statements/0/Operation`).
    ValueSubscription(AttributePath),
    /// An edge from an attribute value to the default subscription source
    /// category node, indicating that this value should be used as a default
    /// subscription source if it matches a prop suggestion on a component.
    DefaultSubscriptionSource,
    /// An edge from anything to a Reason object which explains the reason for
    /// the existence of said thing
    Reason,
}

impl EdgeWeightKind {
    /// Creates a new Use edge weight kind indicating that this is also the
    /// "default" use edge
    pub fn new_use_default() -> Self {
        EdgeWeightKind::Use { is_default: true }
    }

    /// Creates a non-default use EdgeWeightKind. This is what you normally want
    /// unless you know there should be a default/non-default difference
    pub fn new_use() -> Self {
        EdgeWeightKind::Use { is_default: false }
    }
}

#[derive(Hash, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EdgeWeight {
    pub kind: EdgeWeightKind,
}

#[test]
fn edge_weight_size_has_not_increased() {
    assert_eq!(32, std::mem::size_of::<EdgeWeight>());
}

impl EdgeWeight {
    pub fn kind(&self) -> &EdgeWeightKind {
        &self.kind
    }

    pub fn new(kind: EdgeWeightKind) -> Self {
        Self { kind }
    }
}

impl si_split_graph::EdgeKind for EdgeWeightKindDiscriminants {}

impl si_split_graph::CustomEdgeWeight<EdgeWeightKindDiscriminants> for EdgeWeight {
    fn kind(&self) -> EdgeWeightKindDiscriminants {
        self.kind().into()
    }

    fn edge_entropy(&self) -> Option<Vec<u8>> {
        match self.kind() {
            EdgeWeightKind::Contain(Some(key)) => Some(key.as_bytes().to_vec()),
            EdgeWeightKind::Prototype(Some(key)) => Some(key.as_bytes().to_vec()),
            EdgeWeightKind::Use { is_default } => Some(vec![*is_default as u8]),
            EdgeWeightKind::ValueSubscription(path) => Some(path.as_bytes().to_vec()),

            EdgeWeightKind::Contain(None)
            | EdgeWeightKind::Action
            | EdgeWeightKind::ActionPrototype
            | EdgeWeightKind::AuthenticationPrototype
            | EdgeWeightKind::DeprecatedFrameContains
            | EdgeWeightKind::Ordering
            | EdgeWeightKind::Ordinal
            | EdgeWeightKind::Prop
            | EdgeWeightKind::Prototype(None)
            | EdgeWeightKind::PrototypeArgument
            | EdgeWeightKind::PrototypeArgumentValue
            | EdgeWeightKind::Proxy
            | EdgeWeightKind::Root
            | EdgeWeightKind::Socket
            | EdgeWeightKind::SocketValue
            | EdgeWeightKind::ValidationOutput
            | EdgeWeightKind::ManagementPrototype
            | EdgeWeightKind::Represents
            | EdgeWeightKind::Manages
            | EdgeWeightKind::DiagramObject
            | EdgeWeightKind::DefaultSubscriptionSource
            | EdgeWeightKind::ApprovalRequirementDefinition
            | EdgeWeightKind::Reason => None,
        }
    }

    fn is_default(&self) -> bool {
        match self.kind() {
            EdgeWeightKind::Use { is_default } => *is_default,
            EdgeWeightKind::Action
            | EdgeWeightKind::ActionPrototype
            | EdgeWeightKind::AuthenticationPrototype
            | EdgeWeightKind::Contain(_)
            | EdgeWeightKind::DeprecatedFrameContains
            | EdgeWeightKind::Ordering
            | EdgeWeightKind::Ordinal
            | EdgeWeightKind::Prop
            | EdgeWeightKind::Prototype(_)
            | EdgeWeightKind::PrototypeArgument
            | EdgeWeightKind::PrototypeArgumentValue
            | EdgeWeightKind::Proxy
            | EdgeWeightKind::Root
            | EdgeWeightKind::Socket
            | EdgeWeightKind::SocketValue
            | EdgeWeightKind::ValidationOutput
            | EdgeWeightKind::ManagementPrototype
            | EdgeWeightKind::Represents
            | EdgeWeightKind::Manages
            | EdgeWeightKind::DiagramObject
            | EdgeWeightKind::ApprovalRequirementDefinition
            | EdgeWeightKind::DefaultSubscriptionSource
            | EdgeWeightKind::ValueSubscription(_)
            | EdgeWeightKind::Reason => false,
        }
    }

    fn clone_as_non_default(&self) -> Self {
        match self.kind() {
            EdgeWeightKind::Use { .. } => EdgeWeight::new(EdgeWeightKind::new_use()),
            EdgeWeightKind::Action
            | EdgeWeightKind::ActionPrototype
            | EdgeWeightKind::AuthenticationPrototype
            | EdgeWeightKind::Contain(_)
            | EdgeWeightKind::DeprecatedFrameContains
            | EdgeWeightKind::Ordering
            | EdgeWeightKind::Ordinal
            | EdgeWeightKind::Prop
            | EdgeWeightKind::Prototype(_)
            | EdgeWeightKind::PrototypeArgument
            | EdgeWeightKind::PrototypeArgumentValue
            | EdgeWeightKind::Proxy
            | EdgeWeightKind::Root
            | EdgeWeightKind::Socket
            | EdgeWeightKind::SocketValue
            | EdgeWeightKind::ValidationOutput
            | EdgeWeightKind::ManagementPrototype
            | EdgeWeightKind::Represents
            | EdgeWeightKind::Manages
            | EdgeWeightKind::DiagramObject
            | EdgeWeightKind::ApprovalRequirementDefinition
            | EdgeWeightKind::DefaultSubscriptionSource
            | EdgeWeightKind::ValueSubscription(_)
            | EdgeWeightKind::Reason => self.clone(),
        }
    }
}
