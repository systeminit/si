use serde::{Deserialize, Serialize};
use si_events::{ContentHash, merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::workspace_snapshot::graph::LineageId;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::vector_clock::deprecated::DeprecatedVectorClock;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedCategoryNodeWeightLegacy {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub kind: CategoryNodeKind,
    // TODO This should not be a content hash, since it does not point to a value in cas
    pub content_hash: ContentHash,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
}
