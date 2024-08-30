use si_events::ContentHash;

use crate::workspace_snapshot::node_weight::AttributeValueNodeWeight;

pub trait StoresContent {
    fn content_store_hashes(&self) -> Vec<ContentHash>;
}

// AttributeValueNodeWeight stores multiple content values
impl StoresContent for AttributeValueNodeWeight {
    fn content_store_hashes(&self) -> Vec<ContentHash> {
        self.content_store_hashes()
    }
}
