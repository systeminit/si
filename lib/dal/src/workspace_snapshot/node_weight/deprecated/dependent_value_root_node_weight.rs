use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::workspace_snapshot::vector_clock::deprecated::DeprecatedVectorClock;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedDependentValueRootNodeWeight {
    pub id: Ulid,
    pub lineage_id: Ulid,
    value_id: Ulid,
    pub touch_count: u16, // unused
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: DeprecatedVectorClock,
    vector_clock_recently_seen: DeprecatedVectorClock,
    vector_clock_write: DeprecatedVectorClock,
}
