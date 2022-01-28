use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: QualificationCheckComponent,
    pub code_base64: String,
}

impl QualificationCheckRequest {
    pub fn deserialize_from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn serialize_to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckComponent {
    pub name: String,
    pub properties: HashMap<String, Value>,
}

// Note: these map 1:1 to the DAL qualificationsubcheck data in the qualification view.
//       perhaps they should live permanently here, and be exported via veritech?
//       for now I'm duplicating, so we can experiement with it.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum QualificationSubCheckStatus {
    Success,
    Failure,
    Unknown,
}

impl Default for QualificationSubCheckStatus {
    fn default() -> Self {
        QualificationSubCheckStatus::Unknown
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationSubCheck {
    pub description: String,
    pub status: QualificationSubCheckStatus,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QualificationCheckResultSuccess {
    pub execution_id: String,
    pub qualified: bool,
    pub title: Option<String>,
    pub link: Option<String>,
    pub message: Option<String>,
    pub sub_checks: Option<Vec<QualificationSubCheck>>,
    pub timestamp: u64,
}
