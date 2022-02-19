use crate::MaybeSensitive;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub enum ComponentKind {
    Standard,
    Credential,
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemView {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentView {
    pub name: String,
    pub system: Option<SystemView>,
    pub kind: ComponentKind,
    pub properties: MaybeSensitive<serde_json::Value>,
}

impl Default for ComponentView {
    fn default() -> Self {
        Self {
            name: Default::default(),
            system: Default::default(),
            kind: Default::default(),
            properties: MaybeSensitive::Plain(serde_json::json!({})),
        }
    }
}
