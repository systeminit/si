use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    PropKind,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        vector_clock::deprecated::DeprecatedVectorClock,
    },
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedPropNodeWeightLegacy {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub content_address: ContentAddress,
    pub merkle_tree_hash: MerkleTreeHash,
    pub kind: PropKind,
    pub name: String,
    pub can_be_used_as_prototype_arg: bool,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
}
