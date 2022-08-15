use cyclone_core::ResourceSyncResultSuccess;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerResourceSyncResultSuccess {
    pub execution_id: String,
    pub data: Value,
}

impl From<LangServerResourceSyncResultSuccess> for ResourceSyncResultSuccess {
    fn from(value: LangServerResourceSyncResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            data: value.data,
            timestamp: crate::timestamp(),
        }
    }
}
