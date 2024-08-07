use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    action::ActionState,
    workspace_snapshot::graph::{deprecated::v1::DeprecatedActionNodeWeightV1, LineageId},
    ChangeSetId, EdgeWeightKindDiscriminants,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionNodeWeight {
    pub id: Ulid,
    state: ActionState,
    originating_change_set_id: ChangeSetId,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
}

impl ActionNodeWeight {
    pub fn new(originating_change_set_id: ChangeSetId, id: Ulid, lineage_id: Ulid) -> Self {
        Self {
            id,
            state: ActionState::Queued,
            originating_change_set_id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
        }
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn set_state(&mut self, state: ActionState) {
        self.state = state;
    }

    pub fn state(&self) -> ActionState {
        self.state
    }

    pub fn originating_change_set_id(&self) -> ChangeSetId {
        self.originating_change_set_id
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "id": self.id,
            "lineage_id": self.lineage_id,
            "state": self.state,
            "originating_changeset_id": self.originating_change_set_id,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Use]
    }
}

impl From<DeprecatedActionNodeWeightV1> for ActionNodeWeight {
    fn from(value: DeprecatedActionNodeWeightV1) -> Self {
        Self {
            id: value.id,
            state: value.state,
            originating_change_set_id: value.originating_change_set_id,
            lineage_id: value.lineage_id,
            merkle_tree_hash: value.merkle_tree_hash,
        }
    }
}
