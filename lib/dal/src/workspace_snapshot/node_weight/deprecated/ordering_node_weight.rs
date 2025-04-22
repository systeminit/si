use serde::{Deserialize, Serialize};
use si_events::{ContentHash, merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::workspace_snapshot::vector_clock::deprecated::DeprecatedVectorClock;

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct DeprecatedOrderingNodeWeightLegacy {
    pub id: Ulid,
    pub lineage_id: Ulid,
    pub order: Vec<Ulid>,
    pub content_hash: ContentHash,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
}
