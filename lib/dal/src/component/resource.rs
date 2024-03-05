//! This module contains the ability to work with "resources" for [`Components`](crate::Component).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use veritech_client::ResourceStatus;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceView {
    pub status: Option<ResourceStatus>,
    pub message: Option<String>,
    pub data: Option<Value>,
    pub logs: Vec<String>,
    pub last_synced: Option<String>,
}
