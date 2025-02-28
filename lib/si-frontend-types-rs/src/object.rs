use serde::{Deserialize, Serialize};
use si_events::workspace_snapshot::Checksum;

pub mod patch;

pub const KIND_INDEX: &str = "index";

// Payload wrapper for sending data views to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontendObject {
    pub kind: String,
    pub id: String,
    pub checksum: Checksum,
    pub data: serde_json::Value,
}

pub trait FrontendObjectificate {}

pub mod index {
    use serde::{Deserialize, Serialize};

    use crate::reference::IndexReference;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct FrontendObjectIndex(Vec<IndexReference>);
}
