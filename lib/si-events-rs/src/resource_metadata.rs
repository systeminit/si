use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::ComponentId;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ResourceMetadata {
    pub component_id: ComponentId,
    pub status: ResourceStatus,
    pub last_synced: DateTime<Utc>,
}

#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Copy)]
#[serde(rename_all = "camelCase")]
pub enum ResourceStatus {
    Error,
    Ok,
    Warning,
}
