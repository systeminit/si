use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    AttributeValueId,
    PropId,
};
use strum::Display;

use crate::{
    schema_variant::prop_tree::Prop,
    secret::Secret,
};

// This type goes into the content store so cannot be re-ordered, only extended
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Display)]
pub enum ValidationStatus {
    Pending,
    Error,
    Failure,
    Success,
}

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
pub struct ValidationOutput {
    pub status: ValidationStatus,
    pub message: Option<String>,
}

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct AttributeValue {
    pub id: AttributeValueId,
    pub key: Option<String>,
    pub path: Option<String>,
    pub prop_id: Option<PropId>,
    pub value: serde_json::Value,
    pub can_be_set_by_socket: bool, // true if this prop value is currently driven by a socket, even if the socket isn't in use
    pub is_from_external_source: bool, // true if this prop has a value provided by a socket
    pub is_controlled_by_ancestor: bool, // if ancestor of prop is set by dynamic func, ID of ancestor that sets it
    pub is_controlled_by_dynamic_func: bool, // props driven by non-dynamic funcs have a statically set value
    pub overridden: bool, // true if this prop has a different controlling func id than the default for this asset
    pub validation: Option<ValidationOutput>,
    pub secret: Option<Secret>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct AttributeTree {
    pub attribute_values: HashMap<AttributeValueId, AttributeValue>,
    pub props: HashMap<PropId, Prop>,
    pub tree_info: HashMap<AttributeValueId, AvTreeInfo>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
pub struct AvTreeInfo {
    pub parent: Option<AttributeValueId>,
    pub children: Vec<AttributeValueId>,
}
