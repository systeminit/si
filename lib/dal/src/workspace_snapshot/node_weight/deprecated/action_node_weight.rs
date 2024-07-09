use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid};

use crate::{
    action::ActionState,
    func::FuncExecutionPk,
    workspace_snapshot::{graph::LineageId, vector_clock::deprecated::DeprecatedVectorClock},
    ChangeSetId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecatedActionNodeWeight {
    pub id: Ulid,
    state: ActionState,
    originating_changeset_id: ChangeSetId,
    // DEPRECATED
    func_execution_pk: Option<FuncExecutionPk>,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: DeprecatedVectorClock,
    vector_clock_recently_seen: DeprecatedVectorClock,
    vector_clock_write: DeprecatedVectorClock,
}
