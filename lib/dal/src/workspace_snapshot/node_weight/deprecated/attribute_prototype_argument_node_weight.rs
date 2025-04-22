use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    Timestamp,
    workspace_snapshot::{
        graph::LineageId,
        node_weight::ArgumentTargets,
        vector_clock::deprecated::DeprecatedVectorClock,
    },
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedAttributePrototypeArgumentNodeWeightLegacy {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
    pub targets: Option<ArgumentTargets>,
    pub timestamp: Timestamp,
}
