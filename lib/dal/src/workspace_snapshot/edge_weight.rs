//! Edges

use serde::{
    Deserialize,
    Serialize,
};
use strum::EnumDiscriminants;

use crate::workspace_snapshot::graph::deprecated::v1::DeprecatedEdgeWeightV1;

pub mod deprecated;

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
    FrameContains,
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
    /// Edge from [`SchemaVariant`](crate::SchemaVariant) to [`ManagementPrototype`](crate::ManagementPrototype).
    ManagementPrototype,
    /// From a Geometry node to the node it represents on a view.
    Represents,
    /// From a manager [`Component`][`crate::Component`] to a managed `Component
    Manages,
    /// From a view node to a diagram object node, to which geometries can be connected.
    DiagramObject,
    /// Indicates if there is an corresponding approval requirement definition.
    ApprovalRequirementDefinition,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EdgeWeight {
    kind: EdgeWeightKind,
}

impl EdgeWeight {
    pub fn kind(&self) -> &EdgeWeightKind {
        &self.kind
    }

    pub fn new(kind: EdgeWeightKind) -> Self {
        Self { kind }
    }
}

impl From<DeprecatedEdgeWeightV1> for EdgeWeight {
    fn from(value: DeprecatedEdgeWeightV1) -> Self {
        Self { kind: value.kind }
    }
}
