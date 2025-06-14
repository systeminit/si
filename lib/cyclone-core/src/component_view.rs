use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub enum ComponentKind {
    Credential,
    Standard,
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentView {
    pub kind: ComponentKind,
    pub properties: Value,
}

impl Default for ComponentView {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            properties: serde_json::json!({}),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentViewWithGeometry {
    // This is not component kind. Instead it's a schema name
    pub kind: Option<String>,
    pub properties: Value,
    pub sources: Value,
    pub geometry: Value,
    pub incoming_connections: Value,
}

impl Default for ComponentViewWithGeometry {
    fn default() -> Self {
        Self {
            kind: None,
            properties: serde_json::json!({}),
            sources: serde_json::json!({}),
            geometry: serde_json::json!({}),
            incoming_connections: serde_json::json!({}),
        }
    }
}
