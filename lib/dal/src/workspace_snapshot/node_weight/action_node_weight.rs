use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash, VectorClockId};

use crate::{
    action::ActionState,
    workspace_snapshot::{
        graph::LineageId,
        vector_clock::{HasVectorClocks, VectorClock},
    },
    ChangeSetId, EdgeWeightKindDiscriminants,
};

use super::{deprecated::DeprecatedActionNodeWeight, NodeWeightResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionNodeWeight {
    pub id: Ulid,
    state: ActionState,
    originating_change_set_id: ChangeSetId,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl ActionNodeWeight {
    pub fn new(
        vector_clock_id: VectorClockId,
        originating_change_set_id: ChangeSetId,
        id: Ulid,
        lineage_id: Ulid,
    ) -> NodeWeightResult<Self> {
        let new_vector_clock = VectorClock::new(vector_clock_id);

        Ok(Self {
            id,
            state: ActionState::Queued,
            originating_change_set_id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            vector_clock_first_seen: new_vector_clock.clone(),
            vector_clock_recently_seen: new_vector_clock.clone(),
            vector_clock_write: new_vector_clock,
        })
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

impl HasVectorClocks for ActionNodeWeight {
    fn vector_clock_first_seen(&self) -> &VectorClock {
        &self.vector_clock_first_seen
    }

    fn vector_clock_recently_seen(&self) -> &VectorClock {
        &self.vector_clock_recently_seen
    }

    fn vector_clock_write(&self) -> &VectorClock {
        &self.vector_clock_write
    }

    fn vector_clock_first_seen_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_first_seen
    }

    fn vector_clock_recently_seen_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_recently_seen
    }

    fn vector_clock_write_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock_write
    }
}

impl From<DeprecatedActionNodeWeight> for ActionNodeWeight {
    fn from(value: DeprecatedActionNodeWeight) -> Self {
        Self {
            id: value.id,
            state: value.state,
            originating_change_set_id: value.originating_changeset_id,
            lineage_id: value.lineage_id,
            merkle_tree_hash: value.merkle_tree_hash,
            vector_clock_first_seen: VectorClock::empty(),
            vector_clock_recently_seen: VectorClock::empty(),
            vector_clock_write: VectorClock::empty(),
        }
    }
}
