use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::{
    action::prototype::ActionKind,
    workspace_snapshot::{graph::LineageId, vector_clock::deprecated::DeprecatedVectorClock},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedActionPrototypeNodeWeight {
    pub id: Ulid,
    kind: ActionKind,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    name: String,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    description: Option<String>,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: DeprecatedVectorClock,
    vector_clock_recently_seen: DeprecatedVectorClock,
    vector_clock_write: DeprecatedVectorClock,
}
