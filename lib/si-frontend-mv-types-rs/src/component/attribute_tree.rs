use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::EntityKind;
use si_id::{
    AttributeValueId,
    ComponentId,
    PropId,
};
use strum::Display;

use crate::{
    reference::ReferenceKind,
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
    Deserialize,
    Serialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSource {
    pub component_name: String,
    pub path: String,
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
    pub external_sources: Option<Vec<ExternalSource>>, // this is the detail of where the subscriptions are from
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
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
  trigger_entity = EntityKind::Component,
  reference_kind = ReferenceKind::AttributeTree,
)]
pub struct AttributeTree {
    pub id: ComponentId,
    pub attribute_values: HashMap<AttributeValueId, AttributeValue>,
    pub props: HashMap<PropId, Prop>,
    pub tree_info: HashMap<AttributeValueId, AvTreeInfo>,
    pub component_name: String,
    pub schema_name: String,
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
