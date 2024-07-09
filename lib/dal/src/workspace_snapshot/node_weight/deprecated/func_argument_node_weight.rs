use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::workspace_snapshot::{
    content_address::ContentAddress, graph::LineageId,
    vector_clock::deprecated::DeprecatedVectorClock,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedFuncArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: DeprecatedVectorClock,
    vector_clock_recently_seen: DeprecatedVectorClock,
    vector_clock_write: DeprecatedVectorClock,
    name: String,
}
