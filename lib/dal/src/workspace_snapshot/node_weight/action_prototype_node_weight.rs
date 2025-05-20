use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use crate::{
    EdgeWeightKindDiscriminants,
    action::prototype::ActionKind,
    workspace_snapshot::{
        graph::LineageId,
        node_weight::traits::CorrectTransforms,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ActionPrototypeNodeWeight {
    pub id: Ulid,
    kind: ActionKind,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    name: String,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    description: Option<String>,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
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
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(&self.id.inner().to_bytes());
        content_hasher.update(self.kind.to_string().as_bytes());
        content_hasher.update(self.name.as_bytes());
        content_hasher.update(
            &self
                .description
                .as_ref()
                .map(|d| d.as_bytes().to_owned())
                .unwrap_or_else(|| vec![0x00]),
        );

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Use]
    }
}

impl CorrectTransforms for ActionPrototypeNodeWeight {}
