use serde::{Deserialize, Serialize};
use si_events::{FuncId, InputSocketId, OutputSocketId, SchemaId, SchemaVariantId, Timestamp};
use strum::{AsRefStr, Display, EnumIter, EnumString};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariant {
    pub schema_id: SchemaId,
    pub schema_name: String,
    pub schema_variant_id: SchemaVariantId,
    pub display_name: Option<String>,
    pub category: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub color: String,
    pub asset_func_id: FuncId,
    pub func_ids: Vec<FuncId>,
    pub component_type: ComponentType,
    pub input_sockets: Vec<InputSocket>,
    pub output_sockets: Vec<OutputSocket>,
    #[serde(flatten)]
    pub timestamp: Timestamp,
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
)]
#[serde(rename_all = "camelCase")]
pub enum ComponentType {
    AggregationFrame,
    Component,
    ConfigurationFrameDown,
    ConfigurationFrameUp,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputSocket {
    pub id: InputSocketId,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OutputSocket {
    pub id: OutputSocketId,
    pub name: String,
}
