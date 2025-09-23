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
use si_id::ComponentId;

use crate::{
    EdgeWeightKindDiscriminants,
    workspace_snapshot::{
        graph::LineageId,
        node_weight::traits::CorrectTransforms,
    },
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AttributePrototypeArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    targets: Option<ArgumentTargets>,
    // NOTE (jkeiser) this is the only node that has timestamp; should we even have it?
    timestamp: Timestamp,
}

/// When this `AttributePrototypeArgument` represents a connection between two
/// components, we need to know which components are being connected.
///
/// TODO (jkeiser) this currently exists solely to allow old graphs to be deserialized.
/// Remove when we move to a new snapshot version.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
struct ArgumentTargets {
    source_component_id: ComponentId,
    destination_component_id: ComponentId,
}

impl AttributePrototypeArgumentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            targets: None,
            timestamp: Timestamp::now(),
        }
    }

    // Used only for the deserialization test, to ensure we can read old graphs.
    pub fn new_for_deserialization_test() -> Self {
        Self {
            id: Ulid::new(),
            lineage_id: Ulid::new(),
            merkle_tree_hash: MerkleTreeHash::default(),
            targets: Some(ArgumentTargets {
                source_component_id: ComponentId::new(),
                destination_component_id: ComponentId::new(),
            }),
            timestamp: Timestamp::now(),
        }
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        // This makes the node_hash() the same as before (we used "" when we had targets = None)
        content_hasher.update("".as_bytes());
        content_hasher.finalize()
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
            .field("node_hash", &self.node_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl CorrectTransforms for AttributePrototypeArgumentNodeWeight {}
