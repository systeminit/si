use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash, VectorClockId};

use crate::{
    action::prototype::ActionKind,
    workspace_snapshot::{
        graph::LineageId,
        vector_clock::{HasVectorClocks, VectorClock},
    },
    EdgeWeightKindDiscriminants,
};

use super::{
    deprecated::DeprecatedActionPrototypeNodeWeight, traits::UpdateConflictsAndUpdates,
    NodeWeightResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPrototypeNodeWeight {
    pub id: Ulid,
    kind: ActionKind,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    name: String,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    description: Option<String>,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
}

impl ActionPrototypeNodeWeight {
    pub fn new(
        vector_clock_id: VectorClockId,
        id: Ulid,
        lineage_id: Ulid,
        kind: ActionKind,
        name: impl AsRef<str>,
        description: Option<impl AsRef<str>>,
    ) -> NodeWeightResult<Self> {
        let new_vector_clock = VectorClock::new(vector_clock_id);
        let name = name.as_ref().to_string();
        let description = description.map(|d| d.as_ref().to_string());

        Ok(Self {
            id,
            kind,
            name,
            description,
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

    pub fn kind(&self) -> ActionKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
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
            "kind": self.kind,
            "name": self.name,
            "description": self.description,
        }])
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Use]
    }
}

impl HasVectorClocks for ActionPrototypeNodeWeight {
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

impl From<DeprecatedActionPrototypeNodeWeight> for ActionPrototypeNodeWeight {
    fn from(value: DeprecatedActionPrototypeNodeWeight) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            name: value.name,
            description: value.description,
            lineage_id: value.lineage_id,
            merkle_tree_hash: value.merkle_tree_hash,
            vector_clock_first_seen: VectorClock::empty(),
            vector_clock_recently_seen: VectorClock::empty(),
            vector_clock_write: VectorClock::empty(),
        }
    }
}

impl UpdateConflictsAndUpdates for ActionPrototypeNodeWeight {}
