use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    action::prototype::ActionKind,
    workspace_snapshot::graph::{deprecated::v1::DeprecatedActionPrototypeNodeWeightV1, LineageId},
    workspace_snapshot::node_weight::traits::CorrectTransforms,
    EdgeWeightKindDiscriminants,
};

use super::NodeHash;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionPrototypeNodeWeight {
    pub id: Ulid,
    kind: ActionKind,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    name: String,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    description: Option<String>,
    pub lineage_id: LineageId,
    pub(super) merkle_tree_hash: MerkleTreeHash,
}

impl ActionPrototypeNodeWeight {
    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        kind: ActionKind,
        name: impl AsRef<str>,
        description: Option<impl AsRef<str>>,
    ) -> Self {
        let name = name.as_ref().to_string();
        let description = description.map(|d| d.as_ref().to_string());

        Self {
            id,
            kind,
            name,
            description,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
        }
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

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Use]
    }
}

impl NodeHash for ActionPrototypeNodeWeight {
    fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "id": self.id,
            "kind": self.kind,
            "name": self.name,
            "description": self.description,
        }])
    }
}

impl From<DeprecatedActionPrototypeNodeWeightV1> for ActionPrototypeNodeWeight {
    fn from(value: DeprecatedActionPrototypeNodeWeightV1) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            name: value.name,
            description: value.description,
            lineage_id: value.lineage_id,
            merkle_tree_hash: value.merkle_tree_hash,
        }
    }
}

impl CorrectTransforms for ActionPrototypeNodeWeight {}
