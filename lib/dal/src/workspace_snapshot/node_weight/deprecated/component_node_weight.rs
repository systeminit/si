use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::workspace_snapshot::{
    content_address::ContentAddress,
    graph::LineageId,
    vector_clock::deprecated::DeprecatedVectorClock,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedComponentNodeWeightLegacy {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub content_address: ContentAddress,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
    pub to_delete: bool,
}
