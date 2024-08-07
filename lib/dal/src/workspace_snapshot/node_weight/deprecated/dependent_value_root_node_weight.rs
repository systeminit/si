use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::workspace_snapshot::vector_clock::deprecated::DeprecatedVectorClock;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedDependentValueRootNodeWeightLegacy {
    pub id: Ulid,
    pub lineage_id: Ulid,
    pub value_id: Ulid,
    pub touch_count: u16, // unused
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
}
