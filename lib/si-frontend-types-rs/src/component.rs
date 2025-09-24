use std::num::ParseIntError;

use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use si_events::{
    ComponentId,
    SchemaId,
    SchemaVariantId,
    ViewId,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

#[remain::sorted]
#[derive(
    Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy, Display, EnumString, AsRefStr,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ChangeStatus {
    Added,
    Deleted,
    Modified,
    Unmodified,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GridPoint {
    pub x: isize,
    pub y: isize,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RawGeometry {
    pub x: isize,
    pub y: isize,
    pub width: Option<isize>,
    pub height: Option<isize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StringGeometry {
    pub x: String,
    pub y: String,
    pub height: Option<String>,
    pub width: Option<String>,
}

impl TryFrom<StringGeometry> for RawGeometry {
    type Error = ParseIntError;

    fn try_from(value: StringGeometry) -> Result<Self, Self::Error> {
        let mut maybe_width: Option<isize> = None;
        let mut maybe_height: Option<isize> = None;
        if let (Some(width), Some(height)) = (value.width, value.height) {
            maybe_width = Some(width.clone().parse::<isize>()?);
            maybe_height = Some(height.clone().parse::<isize>()?);
        }
        Ok(Self {
            x: value.x.clone().parse::<isize>()?,
            y: value.y.clone().parse::<isize>()?,
            width: maybe_width,
            height: maybe_height,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeometryAndView {
    pub view_id: ViewId,
    pub geometry: RawGeometry,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Size2D {
    pub width: isize,
    pub height: isize,
}

#[derive(Clone, Eq, Debug, PartialEq, Deserialize, Serialize)]
pub struct ConnectionAnnotation {
    pub tokens: Vec<String>,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum DiagramSocketDirection {
    Bidirectional,
    Input,
    Output,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum DiagramSocketNodeSide {
    Left,
    Right,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramSocket {
    pub id: String,
    pub label: String,
    pub connection_annotations: Vec<ConnectionAnnotation>,
    pub direction: DiagramSocketDirection,
    pub max_connections: Option<usize>,
    pub is_required: Option<bool>,
    pub node_side: DiagramSocketNodeSide,
    pub is_management: Option<bool>,
    pub value: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct DiagramComponentView {
    pub id: ComponentId,
    pub component_id: ComponentId,
    pub schema_name: String,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub schema_variant_name: String,
    pub schema_category: String,
    pub schema_docs_link: Option<String>,
    pub sockets: Vec<DiagramSocket>,
    pub display_name: String,
    pub resource_id: String,
    pub color: String,
    pub component_type: String,
    pub change_status: ChangeStatus,
    pub has_resource: bool,
    pub created_info: serde_json::Value,
    pub updated_info: serde_json::Value,
    pub deleted_info: serde_json::Value,
    pub to_delete: bool,
    pub can_be_upgraded: bool,
    pub from_base_change_set: bool,
    pub view_data: Option<GeometryAndView>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PotentialConnection {
    pub socket_id: String,
    pub attribute_value_id: String,
    pub value: Option<Value>,
    pub direction: DiagramSocketDirection, // whether it's input or output
    pub matches: Vec<PotentialMatch>,
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PotentialMatch {
    pub socket_id: String,
    pub component_id: String,
    pub value: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentQualificationStats {
    pub total: u64,
    pub warned: u64,
    pub succeeded: u64,
    pub failed: u64,
    pub running: u64,
}
