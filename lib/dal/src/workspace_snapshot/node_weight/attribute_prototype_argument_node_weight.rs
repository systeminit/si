use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    workspace_snapshot::graph::{
        deprecated::v1::DeprecatedAttributePrototypeArgumentNodeWeightV1, LineageId,
    },
    workspace_snapshot::node_weight::traits::CorrectTransforms,
    ComponentId, EdgeWeightKindDiscriminants, Timestamp,
};

use super::NodeHash;

/// When this `AttributePrototypeArgument` represents a connection between two
/// components, we need to know which components are being connected.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ArgumentTargets {
    pub source_component_id: ComponentId,
    pub destination_component_id: ComponentId,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributePrototypeArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub(super) merkle_tree_hash: MerkleTreeHash,
    targets: Option<ArgumentTargets>,
    timestamp: Timestamp,
}

impl AttributePrototypeArgumentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, targets: Option<ArgumentTargets>) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            targets,
            timestamp: Timestamp::now(),
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
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

    pub fn targets(&self) -> Option<ArgumentTargets> {
        self.targets
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Use,
            EdgeWeightKindDiscriminants::PrototypeArgumentValue,
        ]
    }
}

impl NodeHash for AttributePrototypeArgumentNodeWeight {
    fn node_hash(&self) -> ContentHash {
        self.content_hash()
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

impl From<DeprecatedAttributePrototypeArgumentNodeWeightV1>
    for AttributePrototypeArgumentNodeWeight
{
    fn from(value: DeprecatedAttributePrototypeArgumentNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            merkle_tree_hash: value.merkle_tree_hash,
            targets: value.targets,
            timestamp: value.timestamp,
        }
    }
}

impl CorrectTransforms for AttributePrototypeArgumentNodeWeight {}
