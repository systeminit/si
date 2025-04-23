use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    action::prototype::ActionKind,
    workspace_snapshot::{
        graph::LineageId,
        vector_clock::deprecated::DeprecatedVectorClock,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedActionPrototypeNodeWeightLegacy {
    pub id: Ulid,
    pub kind: ActionKind,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    pub name: String,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    pub description: Option<String>,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
}
