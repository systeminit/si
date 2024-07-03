use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    change_set::ChangeSet,
    workspace_snapshot::{
        graph::LineageId,
        node_weight::NodeWeightResult,
        vector_clock::{HasVectorClocks, VectorClock},
    },
    ComponentId, EdgeWeightKindDiscriminants, Timestamp,
};

/// When this `AttributePrototypeArgument` represents a connection between two
/// components, we need to know which components are being connected.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ArgumentTargets {
    pub source_component_id: ComponentId,
    pub destination_component_id: ComponentId,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AttributePrototypeArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    vector_clock_first_seen: VectorClock,
    vector_clock_recently_seen: VectorClock,
    vector_clock_write: VectorClock,
    targets: Option<ArgumentTargets>,
    timestamp: Timestamp,
}

impl AttributePrototypeArgumentNodeWeight {
    pub fn new(
        change_set: &ChangeSet,
        id: Ulid,
        targets: Option<ArgumentTargets>,
    ) -> NodeWeightResult<Self> {
        Ok(Self {
            id,
            lineage_id: change_set.generate_ulid()?,
            merkle_tree_hash: MerkleTreeHash::default(),
            targets,
            vector_clock_first_seen: VectorClock::new(change_set.vector_clock_id()),
            vector_clock_recently_seen: VectorClock::new(change_set.vector_clock_id()),
            vector_clock_write: VectorClock::new(change_set.vector_clock_id()),
            timestamp: Timestamp::now(),
        })
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn content_hash(&self) -> ContentHash {
        let target_string = self
            .targets
            .map(|targets| {
                format!(
                    "{}{}",
                    targets.source_component_id, targets.destination_component_id
                )
            })
            .unwrap_or("".into());

        ContentHash::new(target_string.as_bytes())
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn node_hash(&self) -> ContentHash {
        self.content_hash()
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn targets(&self) -> Option<ArgumentTargets> {
        self.targets
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Use,
            EdgeWeightKindDiscriminants::PrototypeArgumentValue,
        ]
    }
}

impl HasVectorClocks for AttributePrototypeArgumentNodeWeight {
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

impl std::fmt::Debug for AttributePrototypeArgumentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AttributePrototypeArgumentNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("targets", &self.targets)
            .field("node_hash", &self.node_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .field("vector_clock_first_seen", &self.vector_clock_first_seen)
            .field(
                "vector_clock_recently_seen",
                &self.vector_clock_recently_seen,
            )
            .field("vector_clock_write", &self.vector_clock_write)
            .finish()
    }
}
