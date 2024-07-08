use serde::{Deserialize, Serialize};
use si_events::{
    ActionKind, ActionPrototypeId, AttributePrototypeArgumentId, AttributePrototypeId, ComponentId,
    FuncArgumentId, FuncId, FuncKind, InputSocketId, OutputSocketId, PropId, SchemaVariantId,
    Timestamp,
};
use strum::{AsRefStr, Display, EnumIter, EnumString};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum IntrinsicFuncKind {
    Identity,
    SetArray,
    SetBoolean,
    SetInteger,
    SetJson,
    SetMap,
    SetObject,
    SetString,
    Unset,
    Validation,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncSummary {
    pub func_id: FuncId,
    pub kind: FuncKind,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub is_locked: bool,
    pub arguments: Vec<FuncArgument>,
    #[serde(flatten)]
    pub bindings: FuncBindings,
    pub types: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgument {
    pub id: Option<FuncArgumentId>,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncCode {
    pub func_id: FuncId,
    pub code: String,
}
#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncBindings {
    pub bindings: Vec<FuncBinding>,
}
#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Debug,
    Deserialize,
    EnumString,
    Eq,
    Serialize,
    Display,
    EnumIter,
    PartialEq,
    Hash,
)]
#[serde(rename_all = "camelCase", tag = "bindingKind")]
pub enum FuncBinding {
    #[serde(rename_all = "camelCase")]
    Action {
        // unique ids
        schema_variant_id: Option<SchemaVariantId>,
        action_prototype_id: Option<ActionPrototypeId>,
        func_id: Option<FuncId>,
        //thing that can be updated
        kind: Option<ActionKind>,
    },
    #[serde(rename_all = "camelCase")]
    Attribute {
        // unique ids
        func_id: Option<FuncId>,
        attribute_prototype_id: Option<AttributePrototypeId>,
        // things needed for create
        component_id: Option<ComponentId>,
        schema_variant_id: Option<SchemaVariantId>,

        // things that can be updated
        prop_id: Option<PropId>,
        output_socket_id: Option<OutputSocketId>,

        // can optionally send arguments when creating the prototype,
        // or update them later individually
        argument_bindings: Vec<AttributeArgumentBinding>,
    },
    #[serde(rename_all = "camelCase")]
    Authentication {
        // unique ids
        schema_variant_id: SchemaVariantId,
        func_id: Option<FuncId>,
    },
    #[serde(rename_all = "camelCase")]
    CodeGeneration {
        // unique ids
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        func_id: Option<FuncId>,
        attribute_prototype_id: Option<AttributePrototypeId>,

        // thing that can be updated
        inputs: Vec<LeafInputLocation>,
    },
    #[serde(rename_all = "camelCase")]
    Qualification {
        // unique ids
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        func_id: Option<FuncId>,
        attribute_prototype_id: Option<AttributePrototypeId>,

        // thing that can be updated
        inputs: Vec<LeafInputLocation>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct AttributeArgumentBinding {
    pub func_argument_id: FuncArgumentId,
    pub attribute_prototype_argument_id: Option<AttributePrototypeArgumentId>,
    pub prop_id: Option<PropId>,
    pub input_socket_id: Option<InputSocketId>,
}

/// This enum provides available child [`Prop`](crate::Prop) trees of [`RootProp`](crate::RootProp)
/// that can be used as "inputs" for [`Funcs`](crate::Func) on leaves.
///
/// _Note: not all [`children`](crate::RootPropChild) of [`RootProp`](crate::RootProp) can be used
/// as "inputs" in order to prevent cycles. This enum provides an approved subset of those
/// children_.
#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumString,
    Eq,
    Serialize,
    Display,
    EnumIter,
    Ord,
    PartialEq,
    PartialOrd,
    Hash,
)]
#[serde(rename_all = "camelCase")]
pub enum LeafInputLocation {
    /// The input location corresponding to "/root/code".
    Code,
    /// The input location corresponding to "/root/deleted_at"
    DeletedAt,
    /// The input location corresponding to "/root/domain".
    Domain,
    /// The input location corresponding to "/root/resource".
    Resource,
    /// The input location corresponding to "/root/secrets".
    Secrets,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumString,
    Eq,
    Serialize,
    Display,
    EnumIter,
    Ord,
    PartialEq,
    PartialOrd,
    Hash,
)]
#[serde(rename_all = "camelCase")]
pub enum FuncArgumentKind {
    Any,
    Array,
    Boolean,
    Integer,
    Json,
    Map,
    Object,
    String,
}
