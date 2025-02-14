use serde::Serialize;
use si_events::workspace_snapshot::Checksum;

pub mod patch;

// Payload wrapper for sending data views to the frontend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FrontendObject {
    pub kind: String,
    pub id: String,
    pub checksum: Checksum,
    pub data: serde_json::Value,
}

pub trait FrontendObjectificate {}
