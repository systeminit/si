use serde::{
    Deserialize,
    Serialize,
};

pub mod patch;

// Payload wrapper for sending data views to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendObject {
    pub kind: String,
    pub id: String,
    pub checksum: String,
    pub data: serde_json::Value,
}

impl std::cmp::Ord for FrontendObject {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.kind.cmp(&other.kind) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.id.cmp(&other.id) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        // We can stop with the checksum since if the checksums are
        // equal, `data` will (better) also be euqal. So, no need to
        // fall back in the case of equality here.
        self.checksum.cmp(&other.checksum)
    }
}

impl std::cmp::PartialOrd for FrontendObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::PartialEq for FrontendObject {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.id == other.id
            && self.checksum == other.checksum
            && self.data == other.data
    }
}

impl Eq for FrontendObject {}
