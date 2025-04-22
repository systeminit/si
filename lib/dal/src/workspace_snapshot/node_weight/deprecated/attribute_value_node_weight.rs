use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::{
    func::FuncExecutionPk,
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        vector_clock::{VectorClock, deprecated::DeprecatedVectorClock},
    },
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeprecatedAttributeValueNodeWeightLegacy {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: DeprecatedVectorClock,
    pub vector_clock_recently_seen: DeprecatedVectorClock,
    pub vector_clock_write: DeprecatedVectorClock,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    pub unprocessed_value: Option<ContentAddress>,
    /// The processed return value.
    /// Example: empty array.
    pub value: Option<ContentAddress>,
    // DEPRECATED - this was the old function execution system
    pub func_execution_pk: Option<FuncExecutionPk>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DeprecatedAttributeValueNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub vector_clock_first_seen: VectorClock,
    pub vector_clock_recently_seen: VectorClock,
    pub vector_clock_write: VectorClock,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    pub unprocessed_value: Option<ContentAddress>,
    /// The processed return value.
    /// Example: empty array.
    pub value: Option<ContentAddress>,
}
