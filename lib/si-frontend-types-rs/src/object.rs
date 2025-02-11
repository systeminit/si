use serde::Serialize;
use si_events::workspace_snapshot::Checksum;

// Payload wrapper for sending data views to the frontend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FrontendObject {
    pub kind: String,
    pub id: String,
    pub checksum: Checksum,
    pub data: serde_json::Value,
}
