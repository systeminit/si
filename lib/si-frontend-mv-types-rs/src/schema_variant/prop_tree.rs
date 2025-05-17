use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_id::PropId;
use strum::Display;

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

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Prop {
    pub id: PropId,
    pub kind: PropKind,
    pub widget_kind: PropWidgetKind,
    pub name: String,
    pub path: String,
    pub hidden: bool,
    pub eligible_to_receive_data: bool,
    pub eligible_to_send_data: bool,
    pub create_only: bool,
    pub doc_link: Option<String>,
    pub documentation: Option<String>,
    pub validation_format: Option<String>,
    pub default_can_be_set_by_socket: bool,
    pub is_origin_secret: bool,
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Display, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PropKind {
    Any,
    Array,
    Boolean,
    Float,
    Integer,
    Json,
    Map,
    Object,
    String,
}
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
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct PropTree {
    pub props: HashMap<PropId, Prop>,
    pub tree_info: HashMap<PropId, PropTreeInfo>,
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
pub struct PropTreeInfo {
    pub parent: Option<PropId>,
    pub children: Vec<PropId>,
}
