use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{workspace_snapshot::{content_address::ContentAddress, graph::{deprecated::v1::DeprecatedPropNodeWeightV1, LineageId}, node_weight::{impl_has_discriminated_content_address, traits::CorrectTransforms, NodeHash}}, EdgeWeightKindDiscriminants, PropKind};

use super::HasContent as _;


#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    pub(super) merkle_tree_hash: MerkleTreeHash,
    kind: PropKind,
    name: String,
    can_be_used_as_prototype_arg: bool,
}

impl PropNodeWeight {
    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        content_address: ContentAddress,
        kind: PropKind,
        name: String,
    ) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            kind,
            name,
            can_be_used_as_prototype_arg: false,
        }
    }

    pub fn kind(&self) -> PropKind {
        self.kind
    }

    pub fn can_be_used_as_prototype_arg(&self) -> bool {
        self.can_be_used_as_prototype_arg
    }

    pub fn set_can_be_used_as_prototype_arg(&mut self, can_be_used: bool) {
        self.can_be_used_as_prototype_arg = can_be_used;
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl NodeHash for PropNodeWeight {
    fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "kind": self.kind,
            "name": self.name,
        }])
    }
}

impl_has_discriminated_content_address! { PropNodeWeight: Prop }

impl std::fmt::Debug for PropNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PropNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("kind", &self.kind)
            .field("name", &self.name)
            .field("content_hash", &self.content_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl From<DeprecatedPropNodeWeightV1> for PropNodeWeight {
    fn from(value: DeprecatedPropNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            kind: value.kind,
            name: value.name,
            can_be_used_as_prototype_arg: value.can_be_used_as_prototype_arg,
        }
    }
}

impl CorrectTransforms for PropNodeWeight {}
