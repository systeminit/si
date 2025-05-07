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
    PropKind,
    reference::ReferenceKind,
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
    Clone,
    Debug,
    Deserialize,
    Eq,
    PartialEq,
    Serialize,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
pub struct WidgetOption {
    pub label: String,
    pub value: String,
}

pub type WidgetOptions = Vec<WidgetOption>;

#[remain::sorted]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Display)]
#[serde(rename_all = "camelCase")]
pub enum PropWidgetKind {
    Array,
    Checkbox,
    CodeEditor,
    Color,
    ComboBox { options: Option<WidgetOptions> },
    Header,
    Map,
    Password,
    Secret { options: Option<WidgetOptions> },
    Select { options: Option<WidgetOptions> },
    Text,
    TextArea,
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
pub struct Prop {
    pub id: PropId,
    pub path: String,
    pub name: String,
    pub kind: PropKind,
    pub widget_kind: PropWidgetKind,
    pub doc_link: Option<String>,
    pub documentation: Option<String>,
    pub validation_format: Option<String>,
    pub default_can_be_set_by_socket: bool,
    pub is_origin_secret: bool,
    pub create_only: bool,
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
