use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    ComponentId,
    EdgeWeightKindDiscriminants,
    workspace_snapshot::{
        graph::LineageId,
        node_weight::traits::CorrectTransforms,
    },
};

/// When this `AttributePrototypeArgument` represents a connection between two
/// components, we need to know which components are being connected.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct ArgumentTargets {
    pub source_component_id: ComponentId,
    pub destination_component_id: ComponentId,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AttributePrototypeArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    pub targets: Option<ArgumentTargets>,
    pub timestamp: Timestamp,
}

impl AttributePrototypeArgumentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid) -> Self {
        Self::new_with_targets_for_tests(id, lineage_id, None)
    }

    // Only for use in tests (and called by other test-only functions)
    pub(crate) fn new_with_targets_for_tests(
        id: Ulid,
        lineage_id: Ulid,
        targets: Option<ArgumentTargets>,
    ) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            targets,
            timestamp: Timestamp::now(),
        }
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

impl std::fmt::Debug for AttributePrototypeArgumentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AttributePrototypeArgumentNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("targets", &self.targets)
            .field("node_hash", &self.node_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl CorrectTransforms for AttributePrototypeArgumentNodeWeight {}
