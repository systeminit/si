use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::{
    workspace_snapshot::{
        graph::LineageId, node_weight::ArgumentTargets,
        vector_clock::deprecated::DeprecatedVectorClock,
    },
    Timestamp,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedAttributePrototypeArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: DeprecatedVectorClock,
    vector_clock_recently_seen: DeprecatedVectorClock,
    vector_clock_write: DeprecatedVectorClock,
    targets: Option<ArgumentTargets>,
    timestamp: Timestamp,
}
